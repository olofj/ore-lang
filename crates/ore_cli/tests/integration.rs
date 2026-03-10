use std::path::PathBuf;
use std::process::Command;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()  // crates/
        .parent().unwrap()  // ore/
        .join("tests/fixtures")
}

fn run_ore(fixture: &str) -> String {
    let path = fixtures_dir().join(fixture);
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["run", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("ore run failed for {}:\n{}", fixture, stderr);
    }

    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn phase0_hello() {
    assert_eq!(run_ore("phase_0/hello.ore").trim(), "42");
}

#[test]
fn phase1_arithmetic() {
    assert_eq!(run_ore("phase_1/arithmetic.ore").trim(), "50");
}

#[test]
fn phase1_expressions() {
    let out = run_ore("phase_1/expressions.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["35", "75", "2"]);
}

#[test]
fn phase2_functions() {
    let out = run_ore("phase_2/functions.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["42", "42"]);
}

#[test]
fn phase2_fib() {
    let out = run_ore("phase_2/fib.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["42", "55"]);
}

#[test]
fn phase3_strings() {
    let out = run_ore("phase_3/strings.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["Hello, world!", "The answer is 42", "plain string"]);
}

#[test]
fn phase4_pipeline() {
    assert_eq!(run_ore("phase_4/pipeline.ore").trim(), "-12");
}

#[test]
fn phase4_lambda() {
    let out = run_ore("phase_4/lambda.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["-12", "26", "30"]);
}

#[test]
fn phase5_records() {
    assert_eq!(run_ore("phase_5/records.ore").trim(), "25.0");
}

#[test]
fn phase6_enums() {
    let out = run_ore("phase_6/enums.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["78.53975", "12.0"]);
}

#[test]
fn phase7_loops() {
    let out = run_ore("phase_7/loops.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["45", "5", "3"]);
}

#[test]
fn phase8_option() {
    let out = run_ore("phase_8/option.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["3", "-1"]);
}

#[test]
fn phase9_closures() {
    let out = run_ore("phase_9/closures.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["15", "21"]);
}

#[test]
fn phase10_methods() {
    let out = run_ore("phase_10/methods.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["25.0", "4.0", "5.0"]);
}

#[test]
fn phase11_modules() {
    let out = run_ore("phase_11/modules.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["7", "25"]);
}

#[test]
fn phase12_mut() {
    let out = run_ore("phase_12/mut_check.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["20"]);
}

#[test]
fn phase13_result() {
    let out = run_ore("phase_13/result.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["5", "-1"]);
}

#[test]
fn phase15_concurrency() {
    let out = run_ore("phase_15/concurrency.ore");
    assert!(out.contains("42"));
    assert!(out.contains("1"));
}

#[test]
fn lists_basic() {
    let out = run_ore("lists/basic.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["[1, 2, 3, 4, 5]", "3", "5"]);
}

#[test]
fn lists_methods() {
    let out = run_ore("lists/methods.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["[2, 4, 6, 8, 10]", "[3, 4, 5]", "1", "2", "3", "4", "5"]);
}

#[test]
fn generics_basic() {
    let out = run_ore("generics/basic.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["42", "10", "14"]);
}

#[test]
fn traits_basic() {
    let out = run_ore("traits/basic.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["5", "9"]);
}

#[test]
fn stdlib_file_io() {
    let out = run_ore("stdlib/file_io.ore");
    assert_eq!(out.trim(), "hello from ore");
}

#[test]
fn stdlib_float_interp() {
    let out = run_ore("stdlib/interp.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["pi is 3.14", "42 is cool: true"]);
}

#[test]
fn stdlib_strings() {
    let out = run_ore("stdlib/strings.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["11", "true", "false", "trim me", "50"]);
}

#[test]
fn stdlib_math() {
    let out = run_ore("stdlib/math.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["42", "7", "3", "8", "-5"]);
}
