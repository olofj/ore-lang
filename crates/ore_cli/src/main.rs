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
}

type MainFunc = unsafe extern "C" fn() -> i32;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            if let Err(e) = run_file(&file) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Build { file, output } => {
            if let Err(e) = build_file(&file, &output) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
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

    let mut codegen = ore_codegen::CodeGen::new(context, "ore_main");
    codegen.compile_program(&program).map_err(|e| e.to_string())?;

    Ok(codegen)
}

fn run_file(path: &std::path::Path) -> Result<(), String> {
    let context = Context::create();
    let codegen = compile_source(path, &context)?;

    let ee = codegen.module
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .map_err(|e| format!("JIT error: {}", e))?;

    // Map runtime functions
    macro_rules! map_fn {
        ($name:expr, $func:expr) => {
            ee.add_global_mapping(&codegen.module.get_function($name).unwrap(), $func as usize);
        };
    }
    map_fn!("ore_print_int", ore_runtime::ore_print_int);
    map_fn!("ore_print_bool", ore_runtime::ore_print_bool);
    map_fn!("ore_print_float", ore_runtime::ore_print_float);
    map_fn!("ore_str_new", ore_runtime::ore_str_new);
    map_fn!("ore_str_concat", ore_runtime::ore_str_concat);
    map_fn!("ore_str_print", ore_runtime::ore_str_print);
    map_fn!("ore_str_retain", ore_runtime::ore_str_retain);
    map_fn!("ore_str_release", ore_runtime::ore_str_release);
    map_fn!("ore_int_to_str", ore_runtime::ore_int_to_str);
    map_fn!("ore_bool_to_str", ore_runtime::ore_bool_to_str);
    map_fn!("ore_spawn", ore_runtime::ore_spawn);
    map_fn!("ore_thread_join_all", ore_runtime::ore_thread_join_all);
    map_fn!("ore_sleep", ore_runtime::ore_sleep);

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
