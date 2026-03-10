use clap::{Parser, Subcommand};
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use std::path::PathBuf;

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

fn run_file(path: &std::path::Path) -> Result<(), String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {}", path.display(), e))?;

    // Lex
    let tokens = ore_lexer::lex(&source)
        .map_err(|e| e.to_string())?;

    // Parse
    let program = ore_parser::parse(tokens)
        .map_err(|e| e.to_string())?;

    // Codegen
    let context = Context::create();
    let mut codegen = ore_codegen::CodeGen::new(&context, "ore_main");
    codegen.compile_program(&program)
        .map_err(|e| e.to_string())?;

    // JIT execute
    let ee = codegen.module
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .map_err(|e| format!("JIT error: {}", e))?;

    // Map runtime functions
    ee.add_global_mapping(
        &codegen.module.get_function("ore_print_int").unwrap(),
        ore_runtime::ore_print_int as usize,
    );
    ee.add_global_mapping(
        &codegen.module.get_function("ore_print_bool").unwrap(),
        ore_runtime::ore_print_bool as usize,
    );

    unsafe {
        let main_fn: JitFunction<MainFunc> = ee
            .get_function("main")
            .map_err(|e| format!("no main function: {}", e))?;
        main_fn.call();
    }

    Ok(())
}
