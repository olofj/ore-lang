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

fn run_file(path: &std::path::Path) -> Result<(), String> {
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

    let context = Context::create();
    let mut codegen = ore_codegen::CodeGen::new(&context, "ore_main");
    codegen.compile_program(&program).map_err(|e| e.to_string())?;

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

    unsafe {
        let main_fn: JitFunction<MainFunc> = ee
            .get_function("main")
            .map_err(|e| format!("no main function: {}", e))?;
        main_fn.call();
    }

    Ok(())
}
