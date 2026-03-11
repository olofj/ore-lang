use clap::{Parser, Subcommand};
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "ore", about = "The Ore programming language compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile and run an Ore source file
    Run {
        /// Path to the .ore file
        file: PathBuf,
    },
    /// Compile an Ore source file to a native binary
    Build {
        /// Path to the .ore file
        file: PathBuf,
        /// Output binary path
        #[arg(short, long, default_value = "a.out")]
        output: PathBuf,
    },
    /// Check an Ore source file for errors (parse only, no codegen)
    Check {
        /// Path to the .ore file
        file: PathBuf,
    },
    /// Format an Ore source file
    Fmt {
        /// Path to the .ore file
        file: PathBuf,
        /// Write formatted output back to the file
        #[arg(short, long)]
        write: bool,
    },
    /// Start an interactive REPL
    Repl,
}

type MainFunc = unsafe extern "C" fn() -> i32;

fn print_error_with_context(error: &str, file: &Path) {
    // Try to extract line number from error message (format: "line N: ..." or "N:M: ...")
    let line_num = if error.starts_with("line ") {
        error.strip_prefix("line ")
            .and_then(|s| s.split(':').next())
            .and_then(|s| s.trim().parse::<usize>().ok())
    } else if error.contains("at ") {
        // Parse "parse error at N:M:" or "lex error at N:M:" format
        error.split("at ")
            .nth(1)
            .and_then(|s| s.split(':').next())
            .and_then(|s| s.trim().parse::<usize>().ok())
    } else {
        // Try "N:M:" format
        error.split(':').next()
            .and_then(|s| s.trim().parse::<usize>().ok())
    };

    eprintln!("error: {}", error);

    if let Some(line) = line_num {
        if let Ok(source) = std::fs::read_to_string(file) {
            let lines: Vec<&str> = source.lines().collect();
            let start = if line > 2 { line - 2 } else { 0 };
            let end = (line + 1).min(lines.len());
            eprintln!();
            for i in start..end {
                let marker = if i + 1 == line { " --> " } else { "     " };
                eprintln!("{}{:>4} | {}", marker, i + 1, lines[i]);
            }
            eprintln!();
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            if let Err(e) = run_file(&file) {
                print_error_with_context(&e, &file);
                std::process::exit(1);
            }
        }
        Commands::Build { file, output } => {
            if let Err(e) = build_file(&file, &output) {
                print_error_with_context(&e, &file);
                std::process::exit(1);
            }
        }
        Commands::Check { file } => {
            if let Err(e) = check_file(&file) {
                print_error_with_context(&e, &file);
                std::process::exit(1);
            }
            eprintln!("ok: {}", file.display());
        }
        Commands::Fmt { file, write } => {
            match fmt_file(&file) {
                Ok(formatted) => {
                    if write {
                        if let Err(e) = std::fs::write(&file, &formatted) {
                            eprintln!("error writing {}: {}", file.display(), e);
                            std::process::exit(1);
                        }
                    } else {
                        print!("{}", formatted);
                    }
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Repl => {
            run_repl();
        }
    }
}

/// Parse a single file and return its program.
fn parse_file(path: &Path) -> Result<ore_parser::ast::Program, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {}", path.display(), e))?;
    let tokens = ore_lexer::lex(&source).map_err(|e| e.to_string())?;
    ore_parser::parse(tokens).map_err(|e| e.to_string())
}

/// Recursively resolve `use` items, collecting all imported items.
/// `already_loaded` tracks canonical paths to avoid duplicate loading.
fn resolve_imports(
    program: &ore_parser::ast::Program,
    base_dir: &Path,
    already_loaded: &mut HashSet<PathBuf>,
) -> Result<Vec<ore_parser::ast::Item>, String> {
    let mut imported_items = Vec::new();

    for item in &program.items {
        if let ore_parser::ast::Item::Use { path } = item {
            let resolved = base_dir.join(path);
            let canonical = resolved.canonicalize()
                .map_err(|e| format!("cannot resolve '{}': {}", resolved.display(), e))?;

            if already_loaded.contains(&canonical) {
                continue; // skip duplicates
            }
            already_loaded.insert(canonical.clone());

            let imported_program = parse_file(&canonical)?;
            let dep_dir = canonical.parent().unwrap();

            // Recursively resolve imports from the imported file
            let transitive = resolve_imports(&imported_program, dep_dir, already_loaded)?;
            imported_items.extend(transitive);

            // Add non-Use items from this imported file
            for dep_item in imported_program.items {
                if !matches!(dep_item, ore_parser::ast::Item::Use { .. }) {
                    imported_items.push(dep_item);
                }
            }
        }
    }

    Ok(imported_items)
}

/// Parse, resolve imports, and compile a source file, returning the codegen context.
fn compile_source<'ctx>(
    path: &Path,
    context: &'ctx Context,
) -> Result<ore_codegen::CodeGen<'ctx>, String> {
    let canonical_path = path.canonicalize()
        .map_err(|e| format!("cannot resolve '{}': {}", path.display(), e))?;
    let base_dir = canonical_path.parent().unwrap();

    let program = parse_file(&canonical_path)?;

    // Resolve all use imports
    let mut already_loaded = HashSet::new();
    already_loaded.insert(canonical_path.clone());
    let imported_items = resolve_imports(&program, base_dir, &mut already_loaded)?;

    // Merge: imported items first, then main file items (excluding Use items)
    let mut merged_items = imported_items;
    for item in program.items {
        if !matches!(item, ore_parser::ast::Item::Use { .. }) {
            merged_items.push(item);
        }
    }
    let program = ore_parser::ast::Program { items: merged_items };

    // Type check
    if let Err(errors) = ore_typecheck::typecheck(&program) {
        let msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        return Err(msgs.join("\n"));
    }

    let mut codegen = ore_codegen::CodeGen::new(context, "ore_main");
    codegen.compile_program(&program).map_err(|e| e.to_string())?;

    Ok(codegen)
}

fn check_file(path: &Path) -> Result<(), String> {
    let canonical_path = path.canonicalize()
        .map_err(|e| format!("cannot resolve '{}': {}", path.display(), e))?;
    let base_dir = canonical_path.parent().unwrap();

    let program = parse_file(&canonical_path)?;

    // Also resolve imports to check them
    let mut already_loaded = HashSet::new();
    already_loaded.insert(canonical_path.clone());
    let imported_items = resolve_imports(&program, base_dir, &mut already_loaded)?;

    // Merge for type checking
    let mut merged_items = imported_items;
    for item in program.items {
        if !matches!(item, ore_parser::ast::Item::Use { .. }) {
            merged_items.push(item);
        }
    }
    let merged = ore_parser::ast::Program { items: merged_items };

    // Type check
    if let Err(errors) = ore_typecheck::typecheck(&merged) {
        let msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        return Err(msgs.join("\n"));
    }

    Ok(())
}

fn fmt_file(path: &Path) -> Result<String, String> {
    let program = parse_file(path)?;
    Ok(ore_parser::fmt::format_program(&program))
}

fn run_repl() {
    use std::io::{self, BufRead, Write};

    eprintln!("Ore REPL (type :quit to exit)");

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let mut line_num = 0u64;

    loop {
        eprint!("ore> ");
        io::stderr().flush().unwrap();

        let line = match lines.next() {
            Some(Ok(line)) => line,
            _ => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == ":quit" || trimmed == ":q" {
            break;
        }

        line_num += 1;

        // Wrap the line in fn main
        let source = format!("fn main\n  {}\n", trimmed);

        let result = (|| -> Result<(), String> {
            let tokens = ore_lexer::lex(&source).map_err(|e| e.to_string())?;
            let program = ore_parser::parse(tokens).map_err(|e| e.to_string())?;

            let context = Context::create();
            let mut codegen = ore_codegen::CodeGen::new(&context, &format!("repl_{}", line_num));
            codegen.compile_program(&program).map_err(|e| e.to_string())?;

            let ee = codegen.module
                .create_jit_execution_engine(inkwell::OptimizationLevel::None)
                .map_err(|e| format!("JIT error: {}", e))?;

            map_runtime_functions(&ee, &codegen.module);

            unsafe {
                let main_fn: JitFunction<MainFunc> = ee
                    .get_function("main")
                    .map_err(|e| format!("no main function: {}", e))?;
                main_fn.call();
            }

            Ok(())
        })();

        if let Err(e) = result {
            eprintln!("error: {}", e);
        }
    }
}

/// Map all ore_runtime functions to the JIT execution engine.
/// This is the single source of truth for JIT function mappings.
fn map_runtime_functions(
    ee: &inkwell::execution_engine::ExecutionEngine,
    module: &inkwell::module::Module,
) {
    macro_rules! map_fn {
        ($name:expr, $func:expr) => {
            if let Some(f) = module.get_function($name) {
                ee.add_global_mapping(&f, $func as usize);
            }
        };
    }
    // Print primitives
    map_fn!("ore_print_int", ore_runtime::ore_print_int);
    map_fn!("ore_print_bool", ore_runtime::ore_print_bool);
    map_fn!("ore_print_float", ore_runtime::ore_print_float);
    map_fn!("ore_print_int_no_newline", ore_runtime::ore_print_int_no_newline);
    map_fn!("ore_print_float_no_newline", ore_runtime::ore_print_float_no_newline);
    map_fn!("ore_print_bool_no_newline", ore_runtime::ore_print_bool_no_newline);
    // Strings
    map_fn!("ore_str_new", ore_runtime::ore_str_new);
    map_fn!("ore_str_concat", ore_runtime::ore_str_concat);
    map_fn!("ore_str_print", ore_runtime::ore_str_print);
    map_fn!("ore_str_print_no_newline", ore_runtime::ore_str_print_no_newline);
    map_fn!("ore_str_retain", ore_runtime::ore_str_retain);
    map_fn!("ore_str_release", ore_runtime::ore_str_release);
    map_fn!("ore_int_to_str", ore_runtime::ore_int_to_str);
    map_fn!("ore_float_to_str", ore_runtime::ore_float_to_str);
    map_fn!("ore_bool_to_str", ore_runtime::ore_bool_to_str);
    map_fn!("ore_dynamic_to_str", ore_runtime::ore_dynamic_to_str);
    map_fn!("ore_str_len", ore_runtime::ore_str_len);
    map_fn!("ore_str_eq", ore_runtime::ore_str_eq);
    map_fn!("ore_str_cmp", ore_runtime::ore_str_cmp);
    map_fn!("ore_str_contains", ore_runtime::ore_str_contains);
    map_fn!("ore_str_trim", ore_runtime::ore_str_trim);
    map_fn!("ore_str_split", ore_runtime::ore_str_split);
    map_fn!("ore_str_to_int", ore_runtime::ore_str_to_int);
    map_fn!("ore_str_to_float", ore_runtime::ore_str_to_float);
    map_fn!("ore_str_replace", ore_runtime::ore_str_replace);
    map_fn!("ore_str_starts_with", ore_runtime::ore_str_starts_with);
    map_fn!("ore_str_ends_with", ore_runtime::ore_str_ends_with);
    map_fn!("ore_str_to_upper", ore_runtime::ore_str_to_upper);
    map_fn!("ore_str_to_lower", ore_runtime::ore_str_to_lower);
    map_fn!("ore_str_substr", ore_runtime::ore_str_substr);
    map_fn!("ore_str_chars", ore_runtime::ore_str_chars);
    map_fn!("ore_str_repeat", ore_runtime::ore_str_repeat);
    map_fn!("ore_str_index_of", ore_runtime::ore_str_index_of);
    map_fn!("ore_str_slice", ore_runtime::ore_str_slice);
    map_fn!("ore_assert_fail", ore_runtime::ore_assert_fail);
    map_fn!("ore_str_reverse", ore_runtime::ore_str_reverse);
    map_fn!("ore_list_reverse_new", ore_runtime::ore_list_reverse_new);
    map_fn!("ore_str_split_whitespace", ore_runtime::ore_str_split_whitespace);
    map_fn!("ore_list_min", ore_runtime::ore_list_min);
    map_fn!("ore_list_max", ore_runtime::ore_list_max);
    map_fn!("ore_list_count", ore_runtime::ore_list_count);
    map_fn!("ore_list_sort_by", ore_runtime::ore_list_sort_by);
    map_fn!("ore_list_index_of", ore_runtime::ore_list_index_of);
    map_fn!("ore_list_unique", ore_runtime::ore_list_unique);
    map_fn!("ore_list_flatten", ore_runtime::ore_list_flatten);
    // Lists
    map_fn!("ore_list_new", ore_runtime::ore_list_new);
    map_fn!("ore_list_push", ore_runtime::ore_list_push);
    map_fn!("ore_list_set", ore_runtime::ore_list_set);
    map_fn!("ore_list_get", ore_runtime::ore_list_get);
    map_fn!("ore_list_len", ore_runtime::ore_list_len);
    map_fn!("ore_list_print", ore_runtime::ore_list_print);
    map_fn!("ore_list_print_typed", ore_runtime::ore_list_print_typed);
    map_fn!("ore_list_print_str", ore_runtime::ore_list_print_str);
    map_fn!("ore_list_print_float", ore_runtime::ore_list_print_float);
    map_fn!("ore_list_print_bool", ore_runtime::ore_list_print_bool);
    map_fn!("ore_list_map", ore_runtime::ore_list_map);
    map_fn!("ore_list_filter", ore_runtime::ore_list_filter);
    map_fn!("ore_list_each", ore_runtime::ore_list_each);
    map_fn!("ore_list_sort", ore_runtime::ore_list_sort);
    map_fn!("ore_list_reverse", ore_runtime::ore_list_reverse);
    map_fn!("ore_list_contains", ore_runtime::ore_list_contains);
    map_fn!("ore_list_concat", ore_runtime::ore_list_concat);
    map_fn!("ore_list_par_map", ore_runtime::ore_list_par_map);
    map_fn!("ore_list_par_each", ore_runtime::ore_list_par_each);
    map_fn!("ore_list_reduce", ore_runtime::ore_list_reduce);
    map_fn!("ore_list_find", ore_runtime::ore_list_find);
    map_fn!("ore_list_join", ore_runtime::ore_list_join);
    map_fn!("ore_list_join_str", ore_runtime::ore_list_join_str);
    map_fn!("ore_list_slice", ore_runtime::ore_list_slice);
    map_fn!("ore_list_any", ore_runtime::ore_list_any);
    map_fn!("ore_list_all", ore_runtime::ore_list_all);
    map_fn!("ore_list_zip", ore_runtime::ore_list_zip);
    map_fn!("ore_list_enumerate", ore_runtime::ore_list_enumerate);
    map_fn!("ore_list_flat_map", ore_runtime::ore_list_flat_map);
    map_fn!("ore_range", ore_runtime::ore_range);
    map_fn!("ore_list_take", ore_runtime::ore_list_take);
    map_fn!("ore_list_skip", ore_runtime::ore_list_skip);
    map_fn!("ore_list_sum", ore_runtime::ore_list_sum);
    // Maps
    map_fn!("ore_map_new", ore_runtime::ore_map_new);
    map_fn!("ore_map_set", ore_runtime::ore_map_set);
    map_fn!("ore_map_get", ore_runtime::ore_map_get);
    map_fn!("ore_map_contains", ore_runtime::ore_map_contains);
    map_fn!("ore_map_len", ore_runtime::ore_map_len);
    map_fn!("ore_map_remove", ore_runtime::ore_map_remove);
    map_fn!("ore_map_keys", ore_runtime::ore_map_keys);
    map_fn!("ore_map_values", ore_runtime::ore_map_values);
    map_fn!("ore_map_print", ore_runtime::ore_map_print);
    map_fn!("ore_map_print_str", ore_runtime::ore_map_print_str);
    // Concurrency
    map_fn!("ore_spawn", ore_runtime::ore_spawn);
    map_fn!("ore_spawn_with_arg", ore_runtime::ore_spawn_with_arg);
    map_fn!("ore_thread_join_all", ore_runtime::ore_thread_join_all);
    map_fn!("ore_sleep", ore_runtime::ore_sleep);
    map_fn!("ore_channel_new", ore_runtime::ore_channel_new);
    map_fn!("ore_channel_send", ore_runtime::ore_channel_send);
    map_fn!("ore_channel_recv", ore_runtime::ore_channel_recv);
    // Int math
    map_fn!("ore_int_pow", ore_runtime::ore_int_pow);
    // String parsing
    map_fn!("ore_str_parse_int", ore_runtime::ore_str_parse_int);
    map_fn!("ore_str_parse_float", ore_runtime::ore_str_parse_float);
    // I/O
    map_fn!("ore_readln", ore_runtime::ore_readln);
    map_fn!("ore_file_read", ore_runtime::ore_file_read);
    map_fn!("ore_file_write", ore_runtime::ore_file_write);
}

fn run_file(path: &std::path::Path) -> Result<(), String> {
    let context = Context::create();
    let codegen = compile_source(path, &context)?;

    let ee = codegen.module
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .map_err(|e| format!("JIT error: {}", e))?;

    map_runtime_functions(&ee, &codegen.module);

    unsafe {
        let main_fn: JitFunction<MainFunc> = ee
            .get_function("main")
            .map_err(|e| format!("no main function: {}", e))?;
        main_fn.call();
    }

    Ok(())
}

fn build_file(path: &Path, output: &Path) -> Result<(), String> {
    use inkwell::targets::{
        CodeModel, InitializationConfig, RelocMode, Target, TargetMachine,
    };
    use inkwell::OptimizationLevel;

    let context = Context::create();
    let codegen = compile_source(path, &context)?;

    // Initialize native target for object file emission
    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("failed to initialize native target: {}", e))?;

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple)
        .map_err(|e| format!("failed to get target from triple: {}", e))?;
    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| "failed to create target machine".to_string())?;

    // Write object file to a temp location
    let tmp_dir = std::env::temp_dir().join("ore_build");
    std::fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("failed to create temp dir: {}", e))?;
    let obj_path = tmp_dir.join("output.o");

    machine
        .write_to_file(
            &codegen.module,
            inkwell::targets::FileType::Object,
            &obj_path,
        )
        .map_err(|e| format!("failed to write object file: {}", e))?;

    // Find the ore_runtime staticlib
    let runtime_lib = find_runtime_staticlib()?;

    // Link with cc
    let linker = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let status = std::process::Command::new(&linker)
        .arg(&obj_path)
        .arg(&runtime_lib)
        .arg("-o")
        .arg(output)
        .arg("-lm")     // math library
        .arg("-lpthread") // pthreads (needed by Rust runtime)
        .arg("-ldl")    // dynamic linking (needed by Rust runtime)
        .status()
        .map_err(|e| format!("failed to run linker '{}': {}", linker, e))?;

    if !status.success() {
        return Err(format!("linker '{}' failed with {}", linker, status));
    }

    // Clean up temp object file
    let _ = std::fs::remove_file(&obj_path);

    eprintln!("compiled to {}", output.display());
    Ok(())
}

/// Locate the ore_runtime static library (libore_runtime.a).
///
/// Strategy:
/// 1. Check ORE_RUNTIME_LIB env var
/// 2. Look in the cargo target directory relative to this binary
fn find_runtime_staticlib() -> Result<PathBuf, String> {
    // 1. Explicit env var
    if let Ok(path) = std::env::var("ORE_RUNTIME_LIB") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 2. Walk up from the current exe to find target/*/libore_runtime.a
    if let Ok(exe) = std::env::current_exe() {
        // The exe is typically in target/{debug,release}/ore
        // The staticlib is in target/{debug,release}/libore_runtime.a
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("libore_runtime.a");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    // 3. Try common cargo target paths relative to CWD
    for profile in &["debug", "release"] {
        let candidate = PathBuf::from(format!("target/{}/libore_runtime.a", profile));
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(
        "cannot find libore_runtime.a — build the workspace first with `cargo build`, \
         or set ORE_RUNTIME_LIB to the path of the static library"
            .to_string(),
    )
}
