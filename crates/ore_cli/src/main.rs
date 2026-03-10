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

    let tokens = ore_lexer::lex(&source).map_err(|e| e.to_string())?;
    let program = ore_parser::parse(tokens).map_err(|e| e.to_string())?;

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
