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

/// Run ore and expect a type error. Returns stderr.
fn run_ore_expect_error(fixture: &str) -> String {
    let path = fixtures_dir().join(fixture);
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["run", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore");

    assert!(!output.status.success(), "expected ore to fail for {}", fixture);
    String::from_utf8(output.stderr).unwrap()
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
fn phase8_result_str() {
    let out = run_ore("phase_8/result_str.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["Error: division by zero", "Ok: 5"]);
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
fn phase10_implicit_self() {
    let out = run_ore("phase_10/implicit_self.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["25.0", "4.0", "6.0"]);
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
fn traits_missing_method() {
    let stderr = run_ore_expect_error("traits/missing_method.ore");
    assert!(stderr.contains("missing method 'greet'"), "expected missing method error, got: {}", stderr);
    assert!(stderr.contains("not defined in trait"), "expected extra method error, got: {}", stderr);
}

#[test]
fn traits_wrong_signature() {
    let stderr = run_ore_expect_error("traits/wrong_signature.ore");
    assert!(stderr.contains("return type mismatch"), "expected return type mismatch, got: {}", stderr);
}

#[test]
fn traits_return_type_mismatch() {
    let stderr = run_ore_expect_error("traits/return_type_mismatch.ore");
    assert!(stderr.contains("declared to return Int, but body returns Str"), "expected return type mismatch, got: {}", stderr);
}

#[test]
fn traits_arg_type_mismatch() {
    let stderr = run_ore_expect_error("traits/arg_type_mismatch.ore");
    assert!(stderr.contains("argument 2 of 'add' expects Int, got Str"), "expected type mismatch, got: {}", stderr);
}

#[test]
fn traits_exhaustive_match() {
    let stderr = run_ore_expect_error("traits/exhaustive_match.ore");
    assert!(stderr.contains("non-exhaustive match"), "expected non-exhaustive match error, got: {}", stderr);
    assert!(stderr.contains("East"), "expected missing East, got: {}", stderr);
    assert!(stderr.contains("West"), "expected missing West, got: {}", stderr);
}

#[test]
fn testing_assert_in_main() {
    let out = run_ore("testing/assert_in_main.ore");
    assert_eq!(out.trim(), "all assertions passed");
}

#[test]
fn testing_ore_test() {
    // Test `ore test` subcommand
    let path = fixtures_dir().join("testing/basic.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["test", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore");
    assert!(output.status.success(), "ore test should pass");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("3 passed"), "expected 3 passed, got: {}", stderr);
}

#[test]
fn testing_ore_test_failure() {
    let path = fixtures_dir().join("testing/failing.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["test", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore");
    assert!(!output.status.success(), "ore test should fail");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("1 failed"), "expected 1 failed, got: {}", stderr);
}

#[test]
fn testing_assert_eq() {
    let path = fixtures_dir().join("testing/assert_eq.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["test", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore");
    assert!(output.status.success(), "ore test should pass for assert_eq");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("3 passed"), "expected 3 passed, got: {}", stderr);
}

#[test]
fn testing_comprehensive() {
    let path = fixtures_dir().join("testing/comprehensive.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["test", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore");
    assert!(output.status.success(), "comprehensive tests should pass");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("11 passed"), "expected 11 passed, got: {}", stderr);
}

#[test]
fn stdlib_file_io() {
    let out = run_ore("stdlib/file_io.ore");
    assert_eq!(out.trim(), "hello from ore");
}

#[test]
fn stdlib_json() {
    let out = run_ore("stdlib/json.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "10");
    assert_eq!(lines[1], "20");
    // JSON stringify output (key order may vary)
    assert!(lines[2].contains("\"x\":10"));
    assert!(lines[2].contains("\"y\":20"));
    assert_eq!(lines[3], "{}");
    // Mixed types round-trip
    assert!(lines[4].contains("\"name\":\"Alice\""));
    assert!(lines[4].contains("\"age\":30"));
    assert!(lines[4].contains("\"active\":true"));
    assert_eq!(lines[5], "json ok");
}

#[test]
fn stdlib_time() {
    let out = run_ore("stdlib/time.ore");
    assert_eq!(out.trim(), "time ok");
}

#[test]
fn stdlib_random() {
    let out = run_ore("stdlib/random.ore");
    assert_eq!(out.trim(), "random ok");
}

#[test]
fn stdlib_file_ops() {
    let out = run_ore("stdlib/file_ops.ore");
    assert!(out.contains("line1"));
    assert!(out.contains("line2"));
    assert!(out.contains("file_ops ok"));
}

#[test]
fn stdlib_env() {
    let out = run_ore("stdlib/env.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["hello_ore", "env ok"]);
}

#[test]
fn stdlib_typeof() {
    let out = run_ore("stdlib/typeof.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["Int", "Float", "Bool", "Str", "List", "typeof ok"]);
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

#[test]
fn lists_foreach() {
    let out = run_ore("lists/foreach.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["150", "10", "20", "30"]);
}

#[test]
fn collections_record_list() {
    let out = run_ore("collections/record_list.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["Charlie", "35", "90"]);
}

#[test]
fn collections_string_list() {
    let out = run_ore("collections/string_list.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["Hello, Charlie!", "Hello, Alice!", "Hello, Bob!"]);
}

#[test]
fn showcase() {
    let out = run_ore("showcase.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "11", "25.0", "[5, 3, 4]", "4, 16, 36, 64, 100",
        "60", "Hello, Ore!", "11", "55", "Hello, world!",
        "99", "7", "30", "2", "red", "blue", "the answer", "FizzBuzz",
    ]);
}

#[test]
fn lists_advanced() {
    let out = run_ore("lists/advanced.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "[1, 2, 3, 4, 5]",
        "[5, 4, 3, 2, 1]",
        "true",
        "false",
        "6",
    ]);
}

#[test]
fn lists_closures() {
    let out = run_ore("lists/closures.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "[10, 20, 30, 40, 50]",
        "[4, 5]",
        "101", "102", "103", "104", "105",
    ]);
}

#[test]
fn lists_reduce() {
    let out = run_ore("lists/reduce.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["15", "120", "4", "10-20-30"]);
}

#[test]
fn stdlib_strings_adv() {
    let out = run_ore("stdlib/strings_adv.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "Hello, Ore!", "true", "true",
        "HELLO, WORLD!", "hello, world!", "World",
    ]);
}

#[test]
fn control_elseif() {
    let out = run_ore("control/elseif.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["-1", "0", "1", "2"]);
}

#[test]
fn mutation_assign() {
    let out = run_ore("mutation/assign.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["[1, 99, 3]", "42", "2"]);
}

#[test]
fn maps_basic() {
    let out = run_ore("maps/basic.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["3", "true", "false", "2", "4", "3"]);
}

#[test]
fn control_literal_match() {
    let out = run_ore("control/literal_match.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["three", "Hey Bob!", "yes", "many"]);
}

#[test]
fn control_multiline() {
    let out = run_ore("control/multiline.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["10, 20, 30", "20, 40, 60", "20, 30"]);
}

#[test]
fn control_match_keyword() {
    let out = run_ore("control/match_keyword.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["red", "blue", "seven"]);
}

#[test]
fn control_multiline_pipe() {
    let out = run_ore("control/multiline_pipe.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["4, 16, 36, 64, 100", "60"]);
}

#[test]
fn records_display() {
    let out = run_ore("records/display.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "Point(x: 3, y: 4)",
        "Person(name: Bob, age: 25)",
        "Point(x: 3, y: 4)",
    ]);
}

#[test]
fn lists_typed_display() {
    let out = run_ore("lists/typed_display.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "[hello, world]",
        "[true, false, true]",
        "[1.5, 2.0, 3.14]",
        "[10, 20, 30]",
    ]);
}

#[test]
fn higher_order_fn_params() {
    let out = run_ore("higher_order/fn_params.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["10", "10", "12", "12"]);
}

#[test]
fn enum_display() {
    let out = run_ore("records/enum_display.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "Circle(radius: 5.0)",
        "Rect(width: 3.0, height: 4.0)",
        "Red",
        "Blue",
    ]);
}

#[test]
fn stdlib_split_iter() {
    let out = run_ore("stdlib/split_iter.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["hello", "world", "ore", "[hello, world, ore]"]);
}

#[test]
fn mutation_compound() {
    let out = run_ore("mutation/compound.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["15", "12", "48", "8", "2"]);
}

#[test]
fn mutation_spawn_mut_error() {
    let err = run_ore_expect_error("mutation/spawn_mut.ore");
    assert!(err.contains("cannot send mutable variable 'counter' to spawned task"));
}

#[test]
fn mutation_channel_mut_error() {
    let err = run_ore_expect_error("mutation/channel_mut.ore");
    assert!(err.contains("cannot send mutable variable 'x' through channel"));
}

#[test]
fn stdlib_chars() {
    let out = run_ore("stdlib/chars.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["[h, e, l, l, o]", "h", "e", "l", "l", "o", "6", "-1"]);
}

#[test]
fn concurrency_each_pipe() {
    let out = run_ore("concurrency/each_pipe.ore");
    assert_eq!(out.trim(), "[2, 4, 6, 8, 10]");
}

#[test]
fn lists_any_all() {
    let out = run_ore("lists/any_all.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["true", "false", "true", "false"]);
}

#[test]
fn lists_zip_enum() {
    let out = run_ore("lists/zip_enum.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["true", "false", "true", "false", "3", "3"]);
}

#[test]
fn stdlib_numeric() {
    let out = run_ore("stdlib/numeric.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "42.0", "3", "4.0", "3.0", "3.0", "4.0", "5.0", "4.0", "42",
    ]);
}

#[test]
fn stdlib_methods() {
    let out = run_ore("stdlib/methods.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "7", "3", "5", "10", "0", "5", "1024",
        "3.0", "3.0", "4.0", "2.5", "1.5",
        "42", "3.14", "8",
    ]);
}

#[test]
fn stdlib_range() {
    let out = run_ore("stdlib/range.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "[1, 2, 3, 4, 5]", "55", "[1, 10, 2, 20, 3, 30]",
    ]);
}

#[test]
fn lists_take_skip() {
    let out = run_ore("lists/take_skip.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["[1, 2, 3]", "[3, 4, 5]", "15", "12"]);
}

#[test]
fn control_chain_cmp() {
    let out = run_ore("control/chain_cmp.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["true", "false", "false", "false"]);
}

#[test]
fn control_guard_patterns() {
    let out = run_ore("control/guard_patterns.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["positive", "zero", "negative"]);
}

#[test]
fn control_optional_chain() {
    let out = run_ore("control/optional_chain.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["10", "none", "10", "none"]);
}

#[test]
fn control_option_map() {
    let out = run_ore("control/option_map.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["10", "none", "105"]);
}

#[test]
fn stdlib_slicing() {
    let out = run_ore("stdlib/slicing.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["hello", "world", "[20, 30, 40]", "[10, 20]"]);
}

#[test]
fn control_option_methods() {
    let out = run_ore("control/option_methods.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["42", "0", "true", "false", "false", "true"]);
}

#[test]
fn stdlib_assert_typeof() {
    let out = run_ore("stdlib/assert_typeof.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["Int", "Str", "Bool", "Float", "List"]);
}

#[test]
fn control_or_patterns() {
    let out = run_ore("control/or_patterns.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["small", "medium", "large", "other", "true", "false"]);
}

#[test]
fn control_block_lambda() {
    let out = run_ore("control/block_lambda.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["10", "20", "30", "[3, 5, 7]", "[16, 25, 36]"]);
}

#[test]
fn control_ifelse_expr() {
    let out = run_ore("control/ifelse_expr.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["positive", "negative", "42", "true"]);
}

#[test]
fn control_range_patterns() {
    let out = run_ore("control/range_patterns.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["zero", "small", "medium", "large", "freezing", "cold", "nice", "hot"]);
}

#[test]
fn control_pipe_else() {
    let out = run_ore("control/pipe_else.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["50", "99", "60", "14"]);
}

#[test]
fn stdlib_reverse() {
    let out = run_ore("stdlib/reverse.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["olleh", "edcba", "[5, 4, 3, 2, 1]"]);
}

#[test]
fn lists_min_max_count() {
    let out = run_ore("lists/min_max_count.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["1", "9", "3", "35", "[hello, world, foo, bar]"]);
}

#[test]
fn lists_indexing() {
    let out = run_ore("lists/indexing.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["50", "40", "10", "10", "50"]);
}

#[test]
fn pipelines_comprehensive() {
    let out = run_ore("pipelines/comprehensive.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["10", "100", "10", "hello world ore", "55", "11, 12, 13"]);
}

#[test]
fn showcase2() {
    let out = run_ore("showcase2.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "1: low", "2: low", "3: mid", "4: high", "5: high",
        "[9, 36, 81, 144, 225, 324]",
        "min=1 max=9",
        "[5, 3, 8, 1, 9, 2, 7]",
        "count>5: 3",
        "first=5 last=3",
        "[1, 2, 4, 5, 7]",
        "20", "true", "-1",
        "!dlroW ,olleH", "Hello", "spaces",
        "[the, quick, brown, fox]",
        "Int", "List",
    ]);
}

#[test]
fn showcase3() {
    let out = run_ore("showcase3.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "baby");
    assert_eq!(lines[1], "child");
    assert_eq!(lines[2], "teen");
    assert_eq!(lines[3], "adult");
    assert_eq!(lines[4], "senior");
    assert_eq!(lines[5], "5");
    assert_eq!(lines[6], "-1");
    assert_eq!(lines[7], "275"); // sum of scores (order-independent)
    assert_eq!(lines[8], "0: first");
    assert_eq!(lines[9], "1: second");
    assert_eq!(lines[10], "2: third");
    assert_eq!(lines[11], "7");
    assert_eq!(lines[12], "8");
    assert_eq!(lines[13], "10");
    assert_eq!(lines[14], "256");
    assert_eq!(lines[15], "50");
    assert_eq!(lines[16], "2");
    assert_eq!(lines[17], "0");
}

#[test]
fn showcase4() {
    let out = run_ore("showcase4.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "Alice: active");
    assert_eq!(lines[1], "2");
    assert_eq!(lines[5], "10");
    assert!(lines.last().unwrap().contains("showcase4 ok"));
}

#[test]
fn showcase6() {
    let out = run_ore("showcase6.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "Even numbers: 5");
    assert_eq!(lines[1], "Sum of evens: 30");
    assert_eq!(lines[2], "Server: localhost:8080");
    assert_eq!(lines[3], "Environment: production");
    assert!(lines.last().unwrap().contains("showcase6 ok"));
}

#[test]
fn showcase5() {
    let out = run_ore("showcase5.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert!(lines[0].contains("\"language\":\"Ore\""));
    assert_eq!(lines[1], "Int");
    assert_eq!(lines[2], "Str");
    assert_eq!(lines[3], "List");
    assert_eq!(lines[4], "Float");
    assert_eq!(lines[5], "Bool");
    assert_eq!(lines[6], "showcase5 ok");
}

#[test]
fn maps_typed_values() {
    let out = run_ore("maps/typed_values.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "Alice");
    assert_eq!(lines[1], "Smith");
    assert_eq!(lines[2], "95");
    // keys() returns string keys (order may vary)
    assert!(lines[3].contains("first") && lines[3].contains("last"));
    assert_eq!(lines[4], "3");
}

#[test]
fn maps_iteration() {
    let out = run_ore("maps/iteration.ore");
    assert_eq!(out.trim(), "6");
}

#[test]
fn lists_enumerate() {
    let out = run_ore("lists/enumerate.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["0: Alice", "1: Bob", "2: Charlie", "60"]);
}

#[test]
fn lists_string_lambdas() {
    let out = run_ore("lists/string_lambdas.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "alice", "bob", "charlie", "dave",    // each
        "alice", "charlie", "dave",            // filter (len > 3)
        "alice", "bob", "charlie", "dave",     // map (trim)
        "3",                                   // count (len > 3)
        "string_lambdas ok",
    ]);
}

#[test]
fn maps_merge() {
    let out = run_ore("maps/merge.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["3", "1", "3", "4"]);
}

#[test]
fn lists_map_type_change() {
    let out = run_ore("lists/map_type_change.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["item_1", "item_2", "item_3", "item_100", "item_200", "map_type_change ok"]);
}

#[test]
fn maps_string_values() {
    let out = run_ore("maps/string_values.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["Alice", "admin", "alice", "charlie", "5", "string_values ok"]);
}

#[test]
fn maps_for_kv() {
    let out = run_ore("maps/for_kv.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    // Key order may vary, but we should have name=Alice, city=Paris, and total 60
    assert!(lines.contains(&"name=Alice"));
    assert!(lines.contains(&"city=Paris"));
    assert_eq!(lines.last().unwrap(), &"60");
}

#[test]
fn concurrency_channels() {
    let out = run_ore("concurrency/channels.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["42", "100"]);
}

#[test]
fn generics_monomorphize() {
    let out = run_ore("generics/monomorphize.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "42", "hello", "true",
        "10", "foo",
        "10", "abab",
    ]);
}

#[test]
fn lists_string_compare() {
    let out = run_ore("lists/string_compare.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "true",     // any: has_bob
        "true",     // all: all_short
        "1",        // filter: just_alice.len()
        "charlie",  // find
        "3",        // count: != bob
        "string_compare ok",
    ]);
}

#[test]
fn lists_comprehension() {
    let out = run_ore("lists/comprehension.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "5", "0", "16",       // squares: len, [0], [4]
        "5", "0", "8",        // evens: len, [0], [4]
        "5", "10", "90",      // big_odds: len, [0], [4]
        "5", "2", "10",       // doubled: len, [0], [4]
        "2", "4",             // big: len, [0]
        "comprehension ok",
    ]);
}

#[test]
fn lists_destructure() {
    let out = run_ore("lists/destructure.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "10", "20", "30",       // nums
        "alice", "bob",         // strings
        "100", "200",           // from literal
        "destructure ok",
    ]);
}

#[test]
fn showcase7() {
    let out = run_ore("showcase7.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "5", "0", "64",    // squares comprehension
        "6",                // label lengths
        "60",               // destructuring sum
        "4", "hello", "ore", // words
        "819",              // sum of squares of multiples of 3
        "showcase7 ok",
    ]);
}

#[test]
fn control_comparison_chains() {
    let out = run_ore("control/comparison_chains.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "true", "false",    // 0 < x < 100, 0 < x < 10
        "true", "false",    // a == b == c, a == b == d
        "true", "false",    // 1 <= 5 < 10, 1 <= 0 < 10
        "chains ok",
    ]);
}

#[test]
fn lists_count_by() {
    let out = run_ore("lists/count_by.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "3", "2", "1", "1", "1", // word counts
        "5",                      // unique words
        "2", "2", "1",           // by first letter
        "count_by ok",
    ]);
}

#[test]
fn showcase8() {
    let out = run_ore("showcase8.ore");
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec![
        "3", "2",                  // word counts
        "5",                       // long words
        "alpha", "beta", "gamma",  // destructuring
        "true", "false",           // comparison chains
        "220",                     // pipeline sum of squares
        "hello world",             // string pipeline
        "showcase8 ok",
    ]);
}

#[test]
fn types_bool_methods() {
    let out = run_ore("types/bool_methods.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "true", "false",
        "1", "0",
        "2",
    ]);
}

#[test]
fn showcase11() {
    let out = run_ore("showcase11.ore");
    assert!(out.contains("cumsum: 5 8 16 17 26 28 35"));
    assert!(out.contains("palindromes: racecar, level, madam"));
    assert!(out.contains("quarters: 6 15 24 33"));
    assert!(out.contains("changes: 3 -2 4 3"));
    assert!(out.contains("showcase11 ok"));
}

#[test]
fn lists_scan() {
    let out = run_ore("lists/scan.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "0 1 3 6 10 15",   // running sum
        "1 2 6 24",         // running product
        "0 3 3 4 4 5 9",   // running max
    ]);
}

#[test]
fn lists_take_drop_while() {
    let out = run_ore("lists/take_drop_while.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["1 2 3 4", "5 6 7 8", "1 2 0 3"]);
}

#[test]
fn showcase10() {
    let out = run_ore("showcase10.ore");
    assert!(out.contains("dot: 35"));
    assert!(out.contains("even sum: 110"));
    assert!(out.contains("odd sum: 100"));
    assert!(out.contains("odd squares sum: 1330"));
    assert!(out.contains("showcase10 ok"));
}

#[test]
fn lists_zip_with() {
    let out = run_ore("lists/zip_with.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["11 22 33", "10 40 90", "140"]);
}

#[test]
fn strings_methods2() {
    let out = run_ore("strings/str_methods2.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "2", "3", "0",       // count
        "file", "file.ore",  // strip_suffix
        "example.com",       // strip_prefix
    ]);
}

#[test]
fn lists_partition() {
    let out = run_ore("lists/partition.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["2 4 6 8", "1 3 5 7"]);
}

#[test]
fn lists_range_step() {
    let out = run_ore("lists/range_step.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "0 2 4 6 8",
        "10 8 6 4 2",
        "1 4 7 10 13 16 19",
    ]);
}

#[test]
fn lists_min_max_by() {
    let out = run_ore("lists/min_max_by.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["1", "9", "4", "9"]);
}

#[test]
fn lists_window_chunks() {
    let out = run_ore("lists/window_chunks.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "6", "9", "12",   // window(3) sums
        "2", "2", "1",    // chunks(2) lengths
    ]);
}

#[test]
fn showcase9() {
    let out = run_ore("showcase9.ore");
    assert!(out.contains("Alice"));
    assert!(out.contains("avg: 88"));
    assert!(out.contains("showcase9 ok"));
}

#[test]
fn strings_padding() {
    let out = run_ore("strings/padding.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "   42",
        "hi   ",
        "007",
        "x...",
        "hello",
    ]);
}

#[test]
fn maps_functional() {
    let out = run_ore("maps/functional.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "a: 1", "b: 2", "c: 3",  // each
        "{b: 2, c: 3}",           // filter
        "{a: 2, b: 4, c: 6}",    // map
    ]);
}

#[test]
fn lists_product_is_empty() {
    let out = run_ore("lists/product_is_empty.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "120", "false", "true",  // list product, is_empty
        "false", "true",         // string is_empty
    ]);
}

#[test]
fn lists_sort_by() {
    let out = run_ore("lists/sort_by.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "9 6 5 4 3 2 1 1",
        "1 3 5 8 9",
    ]);
}

#[test]
fn types_numeric_methods() {
    let out = run_ore("types/numeric_methods.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "3.0", "4.0", "4.0",  // floor, ceil, round
        "2.5",                 // abs
        "3.0",                 // sqrt
        "1024.0",              // pow
        "42",                  // int abs
        "3.0",                 // to_float
    ]);
}

#[test]
fn lists_frequencies() {
    let out = run_ore("lists/frequencies.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["3", "2", "1", "1", "2", "3"]);
}

#[test]
fn lists_intersperse() {
    let out = run_ore("lists/intersperse.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["1", "0", "2", "0", "3", "0", "4"]);
}

#[test]
fn maps_entries() {
    let out = run_ore("maps/entries.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["3", "60"]);
}

#[test]
fn maps_get_or() {
    let out = run_ore("maps/get_or.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["10", "99", "20"]);
}

#[test]
fn math_functions() {
    let out = run_ore("math/functions.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "4.0", "1.4142135623730951",   // sqrt
        "3.141592653589793",            // pi
        "true",                         // sin(pi) < 0.0001
        "1.0",                          // cos(0)
        "0.0",                          // log(1)
        "1.0",                          // exp(0)
        "1024.0",                       // pow(2, 10)
        "3.0",                          // floor(3.7)
        "4.0",                          // ceil(3.2)
        "4.0",                          // round(3.5)
        "true", "true",                // euler bounds
    ]);
}

#[test]
fn lists_string_ops() {
    let out = run_ore("lists/string_ops.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "apple, banana, cherry",  // unique
        "true", "false",          // contains
        "2", "-1",                // index_of
        "apple, banana, cherry",  // sort
        "2",                      // frequencies
        "apple",                  // min
        "cherry",                 // max
        "cat, dog",               // unique_by
    ]);
}

#[test]
fn stdlib_comprehensive() {
    let out = run_ore("stdlib/comprehensive.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "Hello world", "h", "o",           // capitalize, char_at
        "20.0", "2.5",                      // average
        "12",                               // filter + sum
        "3", "2",                           // frequencies
        "1 0 2 0 3",                        // intersperse
        "1 2 3 1",                          // dedup
        "10", "99",                         // get_or
        "apple, banana, cherry",            // string sort
        "1.0", "3.14",                      // float sort min/max
        "12.0",                             // sqrt
        "stdlib ok",
    ]);
}

#[test]
fn showcase14() {
    let out = run_ore("showcase14.ore");
    assert!(out.contains("A: 3, B: 3, C: 2, F: 2"));
    assert!(out.contains("median: 84"));
    assert!(out.contains("range: 40"));
    assert!(out.contains("passing: 72, 76, 84, 87, 89, 91, 93, 95"));
    assert!(out.contains("sqrt(9.0) = 3.0"));
    assert!(out.contains("sorted: apple, banana, cherry, date"));
    assert!(out.contains("first: h, last: o"));
    assert!(out.contains("showcase14 ok"));
}

#[test]
fn lists_to_map() {
    let out = run_ore("lists/to_map.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["10", "20", "30", "3"]);
}

#[test]
fn lists_each_with_index() {
    let out = run_ore("lists/each_with_index.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["0: Alice", "1: Bob", "2: Charlie"]);
}

#[test]
fn lists_float_ops() {
    let out = run_ore("lists/float_ops.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["11.0", "45.0", "1.5", "4.0", "55.0"]);
}

#[test]
fn lists_sort_strings() {
    let out = run_ore("lists/sort_strings.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "apple, banana, cherry, date",
        "0.5", "1.0", "2.71", "3.14",
        "1 2 3 1",
    ]);
}

#[test]
fn showcase12() {
    let out = run_ore("showcase12.ore");
    assert!(out.contains("distance: 5.0"));
    assert!(out.contains("yes: 3, no: 2, abstain: 1"));
    assert!(out.contains("date: 2025-03-10"));
    assert!(out.contains("triangle area: 6.0"));
    assert!(out.contains("showcase12 ok"));
}

#[test]
fn showcase13() {
    let out = run_ore("showcase13.ore");
    assert!(out.contains("sum: 219.3"));
    assert!(out.contains("min: 19.8"));
    assert!(out.contains("max: 24.3"));
    assert!(out.contains("sorted: 19.8, 19.8, 20.7, 21.0, 22.5, 22.5, 22.5, 23.1, 23.1, 24.3"));
    assert!(out.contains("unique: 19.8, 20.7, 21.0, 22.5, 23.1, 24.3"));
    assert!(out.contains("hot: 4, warm: 4, cold: 2"));
    assert!(out.contains("kelvin min: 292.95"));
    assert!(out.contains("kelvin max: 297.45"));
    assert!(out.contains("sensors: S1|S2|S3|S4"));
    assert!(out.contains("log(e^2): 2.0"));
    assert!(out.contains("showcase13 ok"));
}

#[test]
fn showcase15() {
    let out = run_ore("showcase15.ore");
    assert!(out.contains("Words: 15"));
    assert!(out.contains("Unique words: 11"));
    assert!(out.contains("Occurrences of 'the': 3"));
    assert!(out.contains("Occurrences of 'fox': 2"));
    assert!(out.contains("Total chars: 59"));
    assert!(out.contains("Long words: brown, indeed, jumps, lazy, over, quick, very"));
    assert!(out.contains("(9) + (16) + (25)"));
    assert!(out.contains("Sum=295 Min=10 Max=70"));
}

#[test]
fn showcase16() {
    let out = run_ore("showcase16.ore");
    assert!(out.contains("Primes: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47"));
    assert!(out.contains("Count: 15"));
    assert!(out.contains("Sum: 328"));
    assert!(out.contains("Max steps: 20"));
    assert!(out.contains("1 2 Fizz 4 Buzz Fizz 7 8 Fizz Buzz 11 Fizz 13 14 FizzBuzz"));
    assert!(out.contains("Pi approx: 3.14"));
    assert!(out.contains("Pi actual: 3.14159"));
}

#[test]
fn function_defaults() {
    let out = run_ore("functions/defaults.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "Hi, Alice!",
        "Hello, Bob!",
        "15",
        "25",
        "0..10 step 1",
        "5..10 step 1",
        "5..20 step 1",
        "1..100 step 5",
    ]);
}

#[test]
fn lists_find_fold() {
    let out = run_ore("lists/find_fold.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec![
        "1",      // find_index > 4
        "-1",     // find_index not found
        "26",     // fold sum
        "120",    // fold product
        "3",      // fold count > 3
        "9",      // fold max
    ]);
}

#[test]
fn showcase17() {
    let out = run_ore("showcase17.ore");
    assert!(out.contains("Students: Alice, Bob, Charlie, Diana, Eve, Frank, Grace, Hank"));
    assert!(out.contains("Average grade: 78"));
    assert!(out.contains("Passing: 7 / 8"));
    assert!(out.contains("Highest: 95"));
    assert!(out.contains("Lowest: 54"));
    assert!(out.contains("A's: 2"));
    assert!(out.contains("B's: 2"));
    assert!(out.contains("Math students: 4"));
    assert!(out.contains("Top students: Alice, Eve"));
    assert!(out.contains("Hello, Ore!"));
    assert!(out.contains("Bonjour, Ore!"));
}

#[test]
fn control_for_step() {
    let out = run_ore("control/for_step.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["0 2 4 6 8", "0 3 6 9 12", "5"]);
}

#[test]
fn strings_triple_quoted() {
    let out = run_ore("strings/triple_quoted.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["3", "Line one", "Line three", "hello", "world"]);
}

#[test]
fn lists_get_or() {
    let out = run_ore("lists/get_or.ore");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["10", "30", "-1", "30", "99"]);
}

#[test]
fn showcase18() {
    let out = run_ore("showcase18.ore");
    assert!(out.contains("*****"));
    assert!(out.contains("*-*-*-*-*"));
    assert!(out.contains("Even squares: 0, 4, 16, 36, 64"));
    assert!(out.contains("Fibs: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55"));
    assert!(out.contains("First fib > 20 at index: 8"));
    assert!(out.contains("5! = 120"));
    assert!(out.contains("Lines in poem: 4"));
    assert!(out.contains("First line: Roses are red,"));
    assert!(out.contains("Long words: Brown, Jumps, Lazy, Over, Quick"));
    assert!(out.contains("Twin prime pairs up to 30: 4"));
}

#[test]
fn showcase19() {
    let out = run_ore("showcase19.ore");
    assert!(out.contains("Dot product: 11"));
    assert!(out.contains("Magnitude: 5.0"));
    assert!(out.contains("Squares: 1, 4, 9, 16, 25"));
    assert!(out.contains("Safe access: -1"));
    assert!(out.contains("Sorted: 11, 12, 22, 25, 64"));
    assert!(out.contains("Mean: 86"));
    assert!(out.contains("Min: 76"));
    assert!(out.contains("Max: 95"));
    assert!(out.contains("Variance: 33"));
    assert!(out.contains("70s: 2"));
    assert!(out.contains("80s: 4"));
    assert!(out.contains("90s: 4"));
}

#[test]
fn showcase20() {
    let out = run_ore("showcase20.ore");
    assert!(out.contains("the: 3"));
    assert!(out.contains("cat: 2"));
    assert!(out.contains("sat: 1"));
    assert!(out.contains("Palindromes: racecar, madam, level"));
    assert!(out.contains("Title case: Hello World From Ore"));
    assert!(out.contains("Vowel count: 11"));
    assert!(out.contains("Longest word: quick"));
    assert!(out.contains("Pattern: *-*-*-*-*-"));
    assert!(out.contains("Cleaned: Hello   World"));
}

#[test]
fn showcase21() {
    let out = run_ore("showcase21.ore");
    assert!(out.contains("Running sum: 0, 1, 3, 6, 10, 15"));
    assert!(out.contains("Under 30 avg: 25"));
    assert!(out.contains("Over 30 avg: 34"));
    assert!(out.contains("Row 1: 4, 5, 6"));
    assert!(out.contains("Grand total: 510"));
    assert!(out.contains("Even sum: 30"));
    assert!(out.contains("Odd sum: 25"));
    assert!(out.contains("Chained result: 360"));
    assert!(out.contains("Unique values: 1, 2, 3, 4"));
    assert!(out.contains("Most common: 4 appears 4 times"));
}

#[test]
fn showcase23() {
    let out = run_ore("showcase23.ore");
    assert!(out.contains("10 / 3 = 3"));
    assert!(out.contains("10 / 0 = 0"));
    assert!(out.contains("Doubled: 10"));
    assert!(out.contains("None doubled: -1"));
    assert!(out.contains("Chained: 90"));
    assert!(out.contains("None chain: 0"));
    assert!(out.contains("Got value: 7"));
    assert!(out.contains("Index of 30: 2"));
    assert!(out.contains("Index of 99: -1"));
    assert!(out.contains("First > 25: 30"));
}

#[test]
fn showcase24() {
    let out = run_ore("showcase24.ore");
    // Thread order is non-deterministic, so check individual results exist
    assert!(out.contains("120"));  // 5!
    assert!(out.contains("40320"));  // 8!
    assert!(out.contains("3628800"));  // 10!
    assert!(out.contains("6820"));  // fib(10) + fib(20) = 55 + 6765
    assert!(out.contains("Sum of squares 1..5: 55"));
}

#[test]
fn strings_ord_chr() {
    let out = run_ore("strings/ord_chr.ore");
    assert!(out.contains("65\n"));
    assert!(out.contains("97\n"));
    assert!(out.contains("A\n"));
    assert!(out.contains("a\n"));
    assert!(out.contains("HIJKL"));
}

#[test]
fn lists_reduce_1arg() {
    let out = run_ore("lists/reduce_1arg.ore");
    assert!(out.contains("Product: 120"));
    assert!(out.contains("Sum: 15"));
    assert!(out.contains("Total with init 100: 115"));
}

#[test]
fn showcase25() {
    let out = run_ore("showcase25.ore");
    assert!(out.contains("Encrypted: Khoor Zruog"));
    assert!(out.contains("Decrypted: Hello World"));
    assert!(out.contains("ROT13: The Quick Brown Fox"));
    assert!(out.contains("Letters in 'Hello, World! 123': 10"));
}

#[test]
fn showcase26() {
    let out = run_ore("showcase26.ore");
    assert!(out.contains("Color: red"));
    assert!(out.contains("Color: rgb(255,128,0)"));
    assert!(out.contains("42 = 42"));
    assert!(out.contains("10+20 = 30"));
    assert!(out.contains("-5: negative"));
    assert!(out.contains("8: positive even"));
    assert!(out.contains("FizzBuzz: 1, 2, Fizz, 4, Buzz"));
}

#[test]
fn showcase27() {
    let out = run_ore("showcase27.ore");
    assert!(out.contains("1\t0\t0"));
    assert!(out.contains("0\t1\t0"));
    assert!(out.contains("1\t2\t3"));
    assert!(out.contains("4\t5\t6"));
    assert!(out.contains("Trace: 15"));
}

#[test]
fn showcase28() {
    let out = run_ore("showcase28.ore");
    assert!(out.contains("3 + 5 = 8"));
    assert!(out.contains("10 - 3 = 7"));
    assert!(out.contains("4 * 7 = 28"));
    assert!(out.contains("2 + 3 * 4 = 20"));
    assert!(out.contains("Multiplication table:"));
}

#[test]
fn showcase29() {
    let out = run_ore("showcase29.ore");
    assert!(out.contains("42 in binary: 101010"));
    assert!(out.contains("255 in hex: FF"));
    assert!(out.contains("Binary 101010 = 42"));
    assert!(out.contains("Roundtrip: 12345 -> 3039 -> 12345"));
}

#[test]
fn showcase30() {
    let out = run_ore("showcase30.ore");
    assert!(out.contains("Primes < 50: 2, 3, 5, 7, 11"));
    assert!(out.contains("Count: 15"));
    assert!(out.contains("Sum of primes < 50: 328"));
    assert!(out.contains("Max Collatz length: 20"));
    assert!(out.contains("Frequency of 1: 2"));
}

#[test]
fn showcase31() {
    let out = run_ore("showcase31.ore");
    assert!(out.contains("3 4 + = 7"));
    assert!(out.contains("6 7 * = 42"));
    assert!(out.contains("3 4 + 2 * = 14"));
    assert!(out.contains("5 1 2 + 4 * + 3 - = 14"));
    assert!(out.contains("2 3 + 4 5 + * = 45"));
}

#[test]
fn showcase32() {
    let out = run_ore("showcase32.ore");
    assert!(out.contains("Bubble sorted: 11, 12, 22, 25, 34, 64, 90"));
    assert!(out.contains("Found 25 at index: 3"));
}

#[test]
fn showcase33() {
    let out = run_ore("showcase33.ore");
    assert!(out.contains("GCD(12, 8) = 4"));
    assert!(out.contains("LCM(12, 8) = 24"));
    assert!(out.contains("2^10 = 1024"));
    assert!(out.contains("Coprime pairs 1-5:"));
}

#[test]
fn lists_pop() {
    let out = run_ore("lists/pop.ore");
    assert_eq!(out.trim(), "30\n2\n20\n1");
}

#[test]
fn showcase34() {
    let out = run_ore("showcase34.ore");
    assert!(out.contains("Double 3 twice: 12"));
    assert!(out.contains("Square 2 twice: 16"));
    assert!(out.contains("Double 1, 10 times: 1024"));
    assert!(out.contains("Sum of squares of multiples of 15 up to 100: 20475"));
    assert!(out.contains("Word count: 9"));
}

#[test]
fn showcase35() {
    let out = run_ore("showcase35.ore");
    assert!(out.contains("Vowels: 11"));
    assert!(out.contains("Consonants: 24"));
    assert!(out.contains("Freq of 'the': 2"));
    assert!(out.contains("racecar is a palindrome"));
    assert!(out.contains("hello is not a palindrome"));
    assert!(out.contains("Capitalized: Hello World"));
}

#[test]
fn showcase36() {
    let out = run_ore("showcase36.ore");
    assert!(out.contains("Perfect numbers < 500: 6, 28, 496"));
    assert!(out.contains("Armstrong numbers (100-999): 153, 370, 371, 407"));
    assert!(out.contains("10! = 3628800"));
    assert!(out.contains("Sum of first 10 Fibonacci: 88"));
}

#[test]
fn showcase37() {
    let out = run_ore("showcase37.ore");
    assert!(out.contains("Rule 110 Cellular Automaton:"));
    assert!(out.contains(".......................................#"));
    assert!(out.contains("......................................##"));
}

#[test]
fn showcase38() {
    let out = run_ore("showcase38.ore");
    assert!(out.contains("Conway's Game of Life - Glider"));
    assert!(out.contains("Generation 0 (5 alive):"));
    assert!(out.contains("Generation 4 (5 alive):"));
}

#[test]
fn showcase39() {
    let out = run_ore("showcase39.ore");
    assert!(out.contains("BF output: Hello World!"));
    assert!(out.contains("A program: A"));
    assert!(out.contains("3+5=8"));
}

#[test]
fn showcase40() {
    let out = run_ore("showcase40.ore");
    assert!(out.contains("Count: 25"));
    assert!(out.contains("Sum: 1060"));
    assert!(out.contains("168 primes"));
    assert!(out.contains("Twin prime pairs up to 1000: 35"));
    assert!(out.contains("Goldbach verified"));
}

#[test]
fn showcase41() {
    let out = run_ore("showcase41.ore");
    assert!(out.contains("NYC: 3, LA: 2, Chicago: 2"));
    assert!(out.contains("Oldest: Frank (40)"));
    assert!(out.contains("Over 30: Charlie, Frank, Grace"));
    assert!(out.contains("NYC residents: Alice, Charlie, Frank"));
}

#[test]
fn showcase42() {
    let out = run_ore("showcase42.ore");
    assert!(out.contains("ROT13:    Uryyb Jbeyq"));
    assert!(out.contains("Round-trip OK: true"));
    assert!(out.contains("Found at shift 7: ATTACK AT DAWN"));
}

#[test]
fn showcase43() {
    let out = run_ore("showcase43.ore");
    assert!(out.contains("Mandelbrot Set:"));
    assert!(out.contains("Center (0,0): 100 iterations"));
    assert!(out.contains("Outside (2,2): 1 iterations"));
}

#[test]
fn showcase44() {
    let out = run_ore("showcase44.ore");
    assert!(out.contains("Maze:"));
    assert!(out.contains("Open cells: 97"));
}

#[test]
fn showcase45() {
    let out = run_ore("showcase45.ore");
    assert!(out.contains("Euler 1 (multiples of 3,5 < 1000): 233168"));
    assert!(out.contains("Euler 2 (even Fibonacci < 4M): 4613732"));
    assert!(out.contains("Euler 6 (sum square diff, n=100): 25164150"));
    assert!(out.contains("Euler 9 (Pythagorean triplet): 31875000"));
    assert!(out.contains("sum primes < 2000): 277050"));
}

#[test]
fn showcase46() {
    let out = run_ore("showcase46.ore");
    assert!(out.contains("Palindrome primes < 200: 2, 3, 5, 7, 11, 101, 131, 151, 181, 191"));
    assert!(out.contains("Sum of halved even squares (1-20): 770"));
    assert!(out.contains("Deduped: 1, 2, 3, 4, 5"));
    assert!(out.contains("Evens by step: 0, 2, 4, 6, 8"));
}

#[test]
fn showcase47() {
    let out = run_ore("showcase47.ore");
    assert!(out.contains("Rect 3x4: area=12"));
    assert!(out.contains("Suits: Hearts, Diamonds, Clubs, Spades"));
    assert!(out.contains("FizzBuzz: 1, 2, Fizz, 4, Buzz"));
}

#[test]
fn showcase48() {
    let out = run_ore("showcase48.ore");
    assert!(out.contains("Final position: (1, 2)"));
    assert!(out.contains("Sum of even squares 1-10: 220"));
    assert!(out.contains("Cubes: 1, 8, 27, 64, 125"));
    assert!(out.contains("10/3 = 3"));
    assert!(out.contains("10/0 = -1"));
    assert!(out.contains("Feature tour complete!"));
}

#[test]
fn showcase49() {
    let out = run_ore("showcase49.ore");
    assert!(out.contains("Lines: 5"));
    assert!(out.contains("Words: 12"));
    assert!(out.contains("Exists: true"));
    assert!(out.contains("Missing: false"));
    assert!(out.contains("First line: Hello World"));
}

#[test]
fn showcase50() {
    let out = run_ore("showcase50.ore");
    assert!(out.contains("'to': 2"));
    assert!(out.contains("'be': 2"));
    assert!(out.contains("Alice: 95"));
    assert!(out.contains("Has alice: true"));
    assert!(out.contains("cache item_5: 25"));
}

#[test]
fn showcase51() {
    let out = run_ore("showcase51.ore");
    assert!(out.contains("The Quick Brown Fox Jumps Over The Lazy Dog"));
    assert!(out.contains("30, 32, 46, 84"));
    assert!(out.contains("hello_world"));
    assert!(out.contains("1 2 3 4 5 6 9"));
}

#[test]
fn showcase52() {
    let out = run_ore("showcase52.ore");
    assert!(out.contains("72"));
    assert!(out.contains("-100"));
    assert!(out.contains("0, 1, 3, 6, 10, 15"));
    assert!(out.contains("Evens: 2, 4, 6, 8, 10"));
    assert!(out.contains("Red: 3"));
    assert!(out.contains("Blue: 4"));
}

#[test]
fn showcase53() {
    let out = run_ore("showcase53.ore");
    assert!(out.contains("Palindromes: racecar, level, madam"));
    assert!(out.contains("Encoded: Uryyb Jbeyq"));
    assert!(out.contains("Decoded: Hello World"));
    assert!(out.contains("Header: name,age,city"));
    assert!(out.contains("Letters: 10, Digits: 3"));
}

#[test]
fn showcase54() {
    let out = run_ore("showcase54.ore");
    assert!(out.contains("1: 2, 3: 4, 5: 3"));
    assert!(out.contains("fib(19) = 4181"));
    assert!(out.contains("Age 25: Alice, Charlie"));
    assert!(out.contains("Age 30: Bob, Diana"));
    assert!(out.contains("the: 3"));
    assert!(out.contains("Keys: a, b, c"));
}

#[test]
fn showcase55() {
    let out = run_ore("showcase55.ore");
    assert!(out.contains("Running sums: 0, 10, 30, 60, 100, 150"));
    assert!(out.contains("Names: Alice, Bob, Charlie, Diana"));
    assert!(out.contains("10! = 3628800"));
    assert!(out.contains("[**********]"));
    assert!(out.contains("100/3 = 33"));
}

#[test]
fn showcase56() {
    let out = run_ore("showcase56.ore");
    assert!(out.contains("a, b, x, c, d, e"));
    assert!(out.contains("Removed: x"));
    assert!(out.contains("Stack top: 30"));
    assert!(out.contains("Squares of first 5 multiples of 7: 49, 196, 441, 784, 1225"));
    assert!(out.contains("Min: 1"));
    assert!(out.contains("Average: 5.0"));
    assert!(out.contains("First > 6: 8"));
    assert!(out.contains("Last: 50"));
    assert!(out.contains("--------------------"));
}

#[test]
fn showcase57() {
    let out = run_ore("showcase57.ore");
    assert!(out.contains("42 found at index: 21"));
    assert!(out.contains("Verify: sorted[21] = 42"));
    assert!(out.contains("gcd(12, 8) = 4"));
    assert!(out.contains("lcm(12, 8) = 24"));
    assert!(out.contains("Primes < 50: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47"));
    assert!(out.contains("1, 2, Fizz, 4, Buzz, Fizz, 7, 8, Fizz, Buzz, 11, Fizz, 13, 14, FizzBuzz"));
    assert!(out.contains("Collatz(27) takes 111 steps"));
    assert!(out.contains("Matrix sum: 10, 10, 10, 10, 10, 10, 10, 10, 10"));
    assert!(out.contains("Trace: 15"));
}

#[test]
fn showcase58() {
    let out = run_ore("showcase58.ore");
    assert!(out.contains("Sum of squares of evens: 220"));
    assert!(out.contains("Product of odds: 945"));
    assert!(out.contains("Running max: 3, 3, 4, 4, 5, 9, 9, 9, 9, 9"));
    assert!(out.contains("apple: 3"));
    assert!(out.contains("banana: 2"));
    assert!(out.contains("7x table: 7, 14, 21, 28, 35, 42, 49, 56, 63, 70"));
    assert!(out.contains("Pascal row 6: 1, 6, 15, 20, 15, 6, 1"));
    assert!(out.contains("Primes <= 30: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29"));
}

#[test]
fn showcase59() {
    let out = run_ore("showcase59.ore");
    assert!(out.contains("Vowels in 'Hello World': 3"));
    assert!(out.contains("Hello World From Ore"));
    assert!(out.contains("ababababab"));
    assert!(out.contains("the: 3"));
    assert!(out.contains("cat: 2"));
    assert!(out.contains("A: ###"));
    assert!(out.contains("B: #######"));
    assert!(out.contains(">     1<"));
    assert!(out.contains("ROT13('Hello World'): Uryyb Jbeyq"));
    assert!(out.contains("ROT13 roundtrip: Hello World"));
}

#[test]
fn showcase60() {
    let out = run_ore("showcase60.ore");
    assert!(out.contains("Fib(15): 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377"));
    assert!(out.contains("Factorials: 1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800"));
    assert!(out.contains("Powers of 2: 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024"));
    assert!(out.contains("digit_sum(12345) = 15"));
    assert!(out.contains("Palindromes 100-200: 101, 111, 121, 131, 141, 151, 161, 171, 181, 191"));
    assert!(out.contains("Perfect numbers <= 500: 6, 28, 496"));
    assert!(out.contains("Triangle numbers: 1, 3, 6, 10, 15, 21, 28, 36, 45, 55"));
    assert!(out.contains("Equal: true"));
}

#[test]
fn showcase61() {
    let out = run_ore("showcase61.ore");
    assert!(out.contains("sqrt(144) = 12.0"));
    assert!(out.contains("abs(-42) = 42"));
    assert!(out.contains("abs(-3.5) = 3.5"));
    assert!(out.contains("floor(3.7) = 3.0"));
    assert!(out.contains("ceil(3.2) = 4.0"));
    assert!(out.contains("round(3.5) = 4.0"));
    assert!(out.contains("pow(2, 10) = 1024.0"));
    assert!(out.contains("sum: 150"));
    assert!(out.contains("product: 12000000"));
    assert!(out.contains("any > 5: true"));
    assert!(out.contains("all > 5: false"));
    assert!(out.contains("take 3: 1, 2, 3"));
    assert!(out.contains("fold sum: 55"));
    assert!(out.contains("flat_map: 1, 10, 2, 20, 3, 30"));
    assert!(out.contains("unique: 1, 2, 3, 4"));
    assert!(out.contains("dedup: 1, 2, 3, 4, 5"));
    assert!(out.contains("index_of 30: 2"));
    assert!(out.contains("contains 40: true"));
    assert!(out.contains("contains 99: false"));
}

#[test]
fn showcase62() {
    let out = run_ore("showcase62.ore");
    assert!(out.contains("host: localhost"));
    assert!(out.contains("new port: 3000"));
    assert!(out.contains("after remove: 2"));
    assert!(out.contains("merged color: red"));
    assert!(out.contains("merged shape: circle"));
    assert!(out.contains("has alice: true"));
    assert!(out.contains("has dave: false"));
    assert!(out.contains("dave score: 0"));
    assert!(out.contains("hello: 3"));
    assert!(out.contains("world: 2"));
    assert!(out.contains("cached 5^2: 25"));
}

#[test]
fn showcase63_run() {
    let out = run_ore("showcase63.ore");
    assert!(out.contains("Tests defined in this file should be run with"));
}

#[test]
fn showcase63_test() {
    let path = fixtures_dir().join("showcase63.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["test", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore");
    assert!(output.status.success(), "ore test failed: {}", String::from_utf8_lossy(&output.stderr));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("8 passed, 0 failed, 8 total"));
}

#[test]
fn showcase64() {
    let out = run_ore("showcase64.ore");
    assert!(out.contains("Distance: 5.0"));
    assert!(out.contains("Circle area: 78.5398"));
    assert!(out.contains("Rect area: 12.0"));
    assert!(out.contains("Triangle area: 24.0"));
    assert!(out.contains("Red: 1"));
    assert!(out.contains("1 = one"));
    assert!(out.contains("4 = other"));
    assert!(out.contains("Circle perimeter: 62.8318"));
    assert!(out.contains("Rect perimeter: 16.0"));
}

#[test]
fn showcase65() {
    let out = run_ore("showcase65.ore");
    assert!(out.contains("Count: 6"));
    assert!(out.contains("Sum: 108"));
    assert!(out.contains("Mean: 18.0"));
    assert!(out.contains("Median: 15.5"));
    assert!(out.contains("Min: 4"));
    assert!(out.contains("Max: 42"));
    assert!(out.contains("Range: 38"));
    assert!(out.contains("Word count: 11"));
    assert!(out.contains("Unique words: 8"));
    assert!(out.contains("Most common: 'the' (3 times)"));
    assert!(out.contains("3: ###### (6)"));
}

#[test]
fn showcase66() {
    let out = run_ore("showcase66.ore");
    assert!(out.contains("First 5 even squares: 4, 16, 36, 64, 100"));
    assert!(out.contains("Sum of multiples of 3 or 5 below 100: 2418"));
    assert!(out.contains("Processed: 'the quick brown cat'"));
    assert!(out.contains("A: 4, B: 3, C: 3"));
    assert!(out.contains("Top 3: 96, 95, 92"));
    assert!(out.contains("Pipeline: 25 > 36 > 49 > 64 > 81 > 100"));
    assert!(out.contains("All chars: helloworld"));
}

#[test]
fn showcase67() {
    let out = run_ore("showcase67.ore");
    assert!(out.contains("10 / 3 = 3"));
    assert!(out.contains("Cannot divide by zero"));
    assert!(out.contains("a = 5, b = -1"));
    assert!(out.contains("head of empty: 0"));
    assert!(out.contains("head of full: 42"));
    assert!(out.contains("doubled: 10"));
    assert!(out.contains("chained: 30"));
    assert!(out.contains("with else: 99"));
    assert!(out.contains("with value: 5"));
}

#[test]
fn showcase68() {
    let out = run_ore("showcase68.ore");
    assert!(out.contains("-10: freezing"));
    assert!(out.contains("20: comfortable"));
    assert!(out.contains("40: hot"));
    assert!(out.contains("200: OK"));
    assert!(out.contains("404: Not Found"));
    assert!(out.contains("418: Unknown (418)"));
    assert!(out.contains("3 is in [3, 7)"));
    assert!(out.contains("1. Ore"));
    assert!(out.contains("42 is medium"));
    assert!(out.contains("12: big positive"));
    assert!(out.contains("0: zero"));
}

#[test]
fn showcase69() {
    let out = run_ore("showcase69.ore");
    assert!(out.contains("length: 13"));
    assert!(out.contains("upper: HELLO, WORLD!"));
    assert!(out.contains("reverse: !dlroW ,olleH"));
    assert!(out.contains("capitalize: Hello"));
    assert!(out.contains("count 'l': 3"));
    assert!(out.contains("trim: 'hello'"));
    assert!(out.contains("replace: Hello, Ore!"));
    assert!(out.contains("split: apple | banana | cherry | date"));
    assert!(out.contains("pad_left: '000042'"));
    assert!(out.contains("repeat: hahaha"));
    assert!(out.contains("strip_suffix: document"));
    assert!(out.contains("ord('A'): 65"));
    assert!(out.contains("chr(65): A"));
    assert!(out.contains("parse_int: 123"));
    assert!(out.contains("empty: true"));
    assert!(out.contains("lines count: 3"));
}

#[test]
fn showcase70() {
    let out = run_ore("showcase70.ore");
    assert!(out.contains("Temperatures: 20.5, 22.3, 19.8, 25.1, 21.7"));
    assert!(out.contains("Average: 21.88"));
    assert!(out.contains("Min: 19.8"));
    assert!(out.contains("Max: 25.1"));
    assert!(out.contains("Days above 21: 3"));
    assert!(out.contains("Fahrenheit: 32.0, 68.0, 98.6, 212.0"));
    assert!(out.contains("Probabilities: 0.2, 0.3, 0.5"));
    assert!(out.contains("Sorted: 0.58, 1.41, 1.73, 2.72, 3.14"));
    assert!(out.contains("Product: 9.0"));
    assert!(out.contains("Steps: 0.0, 0.5, 1.0, 1.5, 2.0, 2.5"));
}

#[test]
fn showcase71() {
    let out = run_ore("showcase71.ore");
    assert!(out.contains("Alice: 95"));
    assert!(out.contains("Bob: 87"));
    assert!(out.contains("Merged keys: 5"));
    assert!(out.contains("c value: 30"));
    assert!(out.contains("a: 5"));
    assert!(out.contains("b: 2"));
    assert!(out.contains("Before clear: 2"));
    assert!(out.contains("After clear: 0"));
    assert!(out.contains("hello freq: 3"));
    assert!(out.contains("ore freq: 1"));
}

#[test]
fn showcase72() {
    let out = run_ore("showcase72.ore");
    assert!(out.contains("Squares: 1, 4, 9, 16, 25, 36, 49, 64, 81, 100"));
    assert!(out.contains("Even squares: 4, 16, 36, 64, 100"));
    assert!(out.contains("Cubes: 1, 8, 27, 64, 125"));
    assert!(out.contains("Take while <= 5: 1, 2, 3, 4, 5"));
    assert!(out.contains("Drop while <= 5: 6, 7, 8, 9, 10"));
    assert!(out.contains("Scan: 0, 1, 3, 6, 10, 15"));
    assert!(out.contains("Reversed: 5, 4, 3, 2, 1"));
    assert!(out.contains("Concat: 1, 2, 3, 4, 5, 6"));
    assert!(out.contains("First: 10"));
    assert!(out.contains("Last: 50"));
    assert!(out.contains("Slice(1,4): 20, 30, 40"));
    assert!(out.contains("Step 3: 0, 3, 6, 9, 12, 15, 18"));
}

#[test]
fn showcase73() {
    let path = fixtures_dir().join("showcase73/main.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["run", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore");
    assert!(output.status.success(), "ore run failed: {}", String::from_utf8_lossy(&output.stderr));
    let out = String::from_utf8(output.stdout).unwrap();
    assert!(out.contains("clamp(15, 0, 10) = 10"));
    assert!(out.contains("clamp(-5, 0, 10) = 0"));
    assert!(out.contains("1: ####### (7)"));
    assert!(out.contains("5 in [0,10]: true"));
    assert!(out.contains("-1 in [0,10]: false"));
}

#[test]
fn showcase74() {
    let out = run_ore("showcase74.ore");
    assert!(out.contains("Hello from thread!"));
    assert!(out.contains("Sum 0..100 = 4950"));
    assert!(out.contains("Par map: 2, 4, 6, 8, 10, 12, 14, 16, 18, 20"));
    assert!(out.contains("After sleep"));
}

#[test]
fn showcase75() {
    let out = run_ore("showcase75.ore");
    assert!(out.contains("Alice       30     95"));
    assert!(out.contains("Total students: 5"));
    assert!(out.contains("Average score: 89.6"));
    assert!(out.contains("Highest: 96"));
    assert!(out.contains("Passing: 4/5"));
    assert!(out.contains("Top scorer: Eve (96)"));
    assert!(out.contains("Students: Alice, Bob, Carol, Dave, Eve"));
}

#[test]
fn showcase76() {
    let out = run_ore("showcase76.ore");
    assert!(out.contains("v1 = (3.0, 4.0)"));
    assert!(out.contains("|v1| = 5.0"));
    assert!(out.contains("v1 + v2 = (4.0, 6.0)"));
    assert!(out.contains("v1 * 2 = (6.0, 8.0)"));
    assert!(out.contains("v1 . v2 = 11.0"));
    assert!(out.contains("Area: 15.0"));
    assert!(out.contains("Perimeter: 16.0"));
    assert!(out.contains("Is square: false"));
    assert!(out.contains("Square: true"));
}

#[test]
fn showcase77() {
    let out = run_ore("showcase77.ore");
    assert!(out.contains("42\n"));
    assert!(out.contains("hello\n"));
    assert!(out.contains("true\n"));
    assert!(out.contains("3.14\n"));
    assert!(out.contains("alpha\n"));
    assert!(out.contains("list identity: 1, 2, 3"));
    assert!(out.contains("composed: 50"));
    assert!(out.contains("Hello, world!"));
    assert!(out.contains("n * 2 = 200"));
}

#[test]
fn showcase78() {
    let out = run_ore("showcase78.ore");
    assert!(out.contains("abs: 42"));
    assert!(out.contains("to_float: -42.0"));
    assert!(out.contains("pow: 1024"));
    assert!(out.contains("clamp: 10\n"));
    assert!(out.contains("floor: 3.0"));
    assert!(out.contains("ceil: 4.0"));
    assert!(out.contains("sqrt: 4.0"));
    assert!(out.contains("true.to_int: 1"));
    assert!(out.contains("parse_int: 123"));
    assert!(out.contains("parsed * 2 = 84"));
    assert!(out.contains("pi round: 3.0"));
}

#[test]
fn showcase79() {
    let out = run_ore("showcase79.ore");
    assert!(out.contains("doubled: 2, 4, 6, 8, 10"));
    assert!(out.contains("tripled: 3, 6, 9, 12, 15"));
    assert!(out.contains("squared: 1, 4, 9, 16, 25"));
    assert!(out.contains("scaled by 7: 7, 14, 21, 28, 35"));
    assert!(out.contains("pipeline: 31, 41, 51"));
    assert!(out.contains("positives: 2, 5, 7"));
    assert!(out.contains("long upper: HELLO, WORLD"));
    assert!(out.contains("product: 120"));
    assert!(out.contains("fold sum: 15"));
}

#[test]
fn div_by_zero() {
    let path = fixtures_dir().join("errors/div_zero.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["run", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("division by zero"));
}

#[test]
fn showcase79_closures_lambdas() {
    let out = run_ore("showcase79.ore");
    assert!(out.contains("doubled: 2, 4, 6, 8, 10"));
    assert!(out.contains("tripled: 3, 6, 9, 12, 15"));
    assert!(out.contains("squared: 1, 4, 9, 16, 25"));
    assert!(out.contains("scaled by 7: 7, 14, 21, 28, 35"));
    assert!(out.contains("pipeline: 31, 41, 51"));
    assert!(out.contains("positives: 2, 5, 7"));
    assert!(out.contains("adjusted: 101, 102, 103, 104, 105"));
    assert!(out.contains("long upper: HELLO, WORLD"));
    assert!(out.contains("product: 120"));
    assert!(out.contains("fold sum: 15"));
}

#[test]
fn showcase80_comprehensive_tour() {
    let out = run_ore("showcase80.ore");
    assert!(out.contains("x = 42, y = 85"));
    assert!(out.contains("true and false = false"));
    assert!(out.contains("Hello, Ore!"));
    assert!(out.contains("upper: HELLO, ORE!"));
    assert!(out.contains("nums: 1, 2, 3, 4, 5"));
    assert!(out.contains("sum: 15, product: 120"));
    assert!(out.contains("evens: 2, 4, 6, 8, 10"));
    assert!(out.contains("#FF0000"));
    assert!(out.contains("#128,64,255"));
    assert!(out.contains("fib(9) = 34"));
    assert!(out.contains("9 -> 36 -> 81 -> 144"));
    assert!(out.contains("sqrt(2) = 1.4142135623730951"));
    assert!(out.contains("2^10 = 1024"));
    assert!(out.contains("Done! Ore is working."));
}

#[test]
fn showcase81_list_mutation_methods() {
    let out = run_ore("showcase81.ore");
    assert!(out.contains("after insert: 1, 2, 3, 4, 5"));
    assert!(out.contains("popped: 5"));
    assert!(out.contains("after pop: 1, 2, 3, 4"));
    assert!(out.contains("removed at 0: 1"));
    assert!(out.contains("after clear len: 0"));
    assert!(out.contains("slice(1,4): 20, 30, 40"));
    assert!(out.contains("concat: 1, 2, 3, 4, 5, 6"));
    assert!(out.contains("any > 7: true"));
    assert!(out.contains("all > 0: true"));
    assert!(out.contains("all > 5: false"));
    assert!(out.contains("find > 25: 30"));
    assert!(out.contains("find_index > 25: 2"));
    assert!(out.contains("average: 30.0"));
}

#[test]
fn showcase82_map_utility_methods() {
    let out = run_ore("showcase82.ore");
    assert!(out.contains("len: 3"));
    assert!(out.contains("has host: true"));
    assert!(out.contains("has timeout: false"));
    assert!(out.contains("host: localhost"));
    assert!(out.contains("timeout: 30"));
    assert!(out.contains("merged port: 8080"));
    assert!(out.contains("merged timeout: 30"));
}

#[test]
fn showcase83_result_type() {
    let out = run_ore("showcase83.ore");
    assert!(out.contains("10 / 3 = 3"));
    assert!(out.contains("error: division by zero"));
    assert!(out.contains("100 / 2 = 50"));
    assert!(out.contains("100 / 0: division by zero"));
    assert!(out.contains("100 / 10 = 10"));
}

#[test]
fn showcase84_pattern_matching() {
    let out = run_ore("showcase84.ore");
    assert!(out.contains("small circle"));
    assert!(out.contains("large circle"));
    assert!(out.contains("rectangle"));
    assert!(out.contains("square"));
    assert!(out.contains("equilateral"));
    assert!(out.contains("circle area: 78.53975"));
    assert!(out.contains("rect area: 12.0"));
    assert!(out.contains("x has 42"));
    assert!(out.contains("y is empty"));
    assert!(out.contains("other"));
}

#[test]
fn showcase85_string_operations() {
    let out = run_ore("showcase85.ore");
    assert!(out.contains("trim: 'Hello, World!'"));
    assert!(out.contains("starts_with: true"));
    assert!(out.contains("split: one | two | three | four"));
    assert!(out.contains("the dog sat on the mat"));
    assert!(out.contains("--------------------"));
    assert!(out.contains("sub: Hello"));
    assert!(out.contains("char_at(7): W"));
    assert!(out.contains("index_of: 7"));
    assert!(out.contains("empty len: 0"));
}

#[test]
fn showcase86_loops_mutation() {
    let out = run_ore("showcase86.ore");
    assert!(out.contains("sum 1..10 = 55"));
    assert!(out.contains("7! = 5040"));
    assert!(out.contains("loop count: 5"));
    assert!(out.contains("pairs where x < y: 6"));
    assert!(out.contains("squares: 1, 4, 9, 16, 25"));
    assert!(out.contains("5, 4, 3, 2, 1"));
}

#[test]
fn showcase87_multiline_match_arms() {
    let out = run_ore("showcase87.ore");
    assert!(out.contains("small circle (r=5.0)"));
    assert!(out.contains("large circle (r=15.0)"));
    assert!(out.contains("rectangle (3.0x4.0)"));
    assert!(out.contains("square (7.0x7.0)"));
    assert!(out.contains("zero"));
    assert!(out.contains("one"));
    assert!(out.contains("negative"));
    assert!(out.contains("big"));
    assert!(out.contains("other"));
}

#[test]
fn showcase88_impl_methods() {
    let out = run_ore("showcase88.ore");
    assert!(out.contains("a = (3.0, 4.0)"));
    assert!(out.contains("|a| = 5.0"));
    assert!(out.contains("a + b = (4.0, 6.0)"));
    assert!(out.contains("a * 2 = (6.0, 8.0)"));
    assert!(out.contains("a . b = 11.0"));
    assert!(out.contains("chain = (5.0, 3.0)"));
}

#[test]
fn showcase89_traits() {
    let out = run_ore("showcase89.ore");
    assert!(out.contains("Rex (dog, age 5)"));
    assert!(out.contains("Whiskers (cat, 9 lives)"));
    assert!(out.contains("circle area: 78.53975"));
    assert!(out.contains("square area: 16.0"));
}

#[test]
fn showcase90_stack_data_structure() {
    let out = run_ore("showcase90.ore");
    assert!(out.contains("size: 3"));
    assert!(out.contains("top: 30"));
    assert!(out.contains("result: 35"));
    assert!(out.contains("final size: 1"));
}

#[test]
fn showcase91_default_params() {
    let out = run_ore("showcase91.ore");
    assert!(out.contains("Hello, World!"));
    assert!(out.contains("Hi, World!"));
    assert!(out.contains("ababab"));
    assert!(out.contains("xyxyxyxyxy"));
    assert!(out.contains("clamp 50: 50"));
    assert!(out.contains("clamp -10: 0"));
    assert!(out.contains("clamp 150: 100"));
    assert!(out.contains("clamp 150 (10..80): 80"));
}

#[test]
fn showcase92_file_io() {
    let out = run_ore("showcase92.ore");
    assert!(out.contains("read back: Hello from Ore!"));
    assert!(out.contains("exists: true"));
    assert!(out.contains("missing: false"));
    assert!(out.contains("line count: 4"));
    assert!(out.contains("last line: Line 4"));
}

#[test]
fn showcase93_json() {
    let out = run_ore("showcase93.ore");
    assert!(out.contains("host: localhost"));
    assert!(out.contains("port: 8080"));
    // JSON output should have string values properly quoted
    assert!(out.contains("\"host\":\"localhost\""));
}

#[test]
fn showcase94_complex_expressions() {
    let out = run_ore("showcase94.ore");
    assert!(out.contains("complex: 15"));
    assert!(out.contains("label: small"));
    assert!(out.contains("grade: B"));
    assert!(out.contains("distance: 5.0"));
    assert!(out.contains("midpoint: (1.5, 2.0)"));
    assert!(out.contains("sum of even squares: 220"));
}

#[test]
fn showcase95_functional_patterns() {
    let out = run_ore("showcase95.ore");
    assert!(out.contains("double twice 3: 12"));
    assert!(out.contains("sum of odd squares: 165"));
    assert!(out.contains("product of evens: 3840"));
    assert!(out.contains("running total: 55"));
    assert!(out.contains("not div by 2,3,5: 7, 11, 13, 17, 19"));
}

#[test]
fn showcase96_time_env_random() {
    let out = run_ore("showcase96.ore");
    assert!(out.contains("time is positive: true"));
    assert!(out.contains("PATH starts with /: true"));
    assert!(out.contains("missing env empty: true"));
    assert!(out.contains("random in range: true"));
}

#[test]
fn showcase97_state_machine() {
    let out = run_ore("showcase97.ore");
    assert!(out.contains("start: idle"));
    assert!(out.contains("-> done(50)"));
    assert!(out.contains("finished in 7 iterations"));
    assert!(out.contains("Alice: 95 (excellent)"));
    assert!(out.contains("Dave: 78 (passing)"));
}

#[test]
fn showcase98_test_suite() {
    // Run as ore test
    let path = fixtures_dir().join("showcase98.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["test", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore test");
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("9 passed, 0 failed"));
}

#[test]
fn showcase99_mini_application() {
    let out = run_ore("showcase99.ore");
    assert!(out.contains("[done] Write tests (HIGH)"));
    assert!(out.contains("[todo] Fix bug (HIGH)"));
    assert!(out.contains("done: 1, todo: 4"));
    assert!(out.contains("! Fix bug"));
    assert!(out.contains("! Deploy"));
}

#[test]
fn showcase100_celebration() {
    let out = run_ore("showcase100.ore");
    assert!(out.contains("FizzBuzz 1-20:"));
    assert!(out.contains("Primes < 50: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47"));
    assert!(out.contains("27 -> 111 steps"));
    assert!(out.contains("Prime squares < 500: 4, 9, 25, 49, 121, 169, 289, 361"));
    assert!(out.contains("Primes under 1000: 168"));
    assert!(out.contains("100 showcases complete. Ore is ready."));
}

#[test]
fn showcase101_inline_if_then_else() {
    let out = run_ore("showcase101.ore");
    assert!(out.contains("x = 42"));
    assert!(out.contains("label: big"));
    assert!(out.contains("abs(-7) = 7"));
    assert!(out.contains("sign(5) = positive"));
    assert!(out.contains("sign(0) = zero"));
    assert!(out.contains("42 is even"));
    assert!(out.contains("labels: small, small, small, big, big"));
}

#[test]
fn showcase102_multiline_match_logic() {
    let out = run_ore("showcase102.ore");
    assert!(out.contains("zero"));
    assert!(out.contains("positive number 42"));
    assert!(out.contains("negative number -7"));
    assert!(out.contains("3 + 4 = 7"));
    assert!(out.contains("5 * 6 = 30"));
    assert!(out.contains("negation of 10 = -10"));
    assert!(out.contains("eval Add(3,4) = 7"));
}

#[test]
fn showcase103_pipelines_transformation() {
    let out = run_ore("showcase103.ore");
    assert!(out.contains("racecar: yes"));
    assert!(out.contains("hello: no"));
    assert!(out.contains("encrypted: Khoor Zruog"));
    assert!(out.contains("decrypted: Hello World"));
    assert!(out.contains("the: 4"));
    assert!(out.contains("word count: 11"));
    assert!(out.contains("unique words: 7"));
}

#[test]
fn showcase104_advanced_tests() {
    let path = fixtures_dir().join("showcase104.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["test", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore test");
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("6 passed, 0 failed"));
}

#[test]
fn showcase105_generics() {
    let out = run_ore("showcase105.ore");
    assert!(out.contains("swap(1,2): 2, 1"));
    assert!(out.contains("max(3, 7): 7"));
    assert!(out.contains("min(3, 7): 3"));
    assert!(out.contains("id(42): 42"));
    assert!(out.contains("id(hello): hello"));
    assert!(out.contains("apply double 21: 42"));
    assert!(out.contains("sorted: 1, 3, 5, 8, 9"));
}

#[test]
fn showcase106_real_world_patterns() {
    let out = run_ore("showcase106.ore");
    assert!(out.contains("port: 8080"));
    assert!(out.contains("debug: true"));
    assert!(out.contains("= Report ="));
    assert!(out.contains("- First item"));
    assert!(out.contains("| Alice | 95 | A |"));
    assert!(out.contains("values > 30: count=7, sum=490, avg=70"));
}

#[test]
fn showcase107_recursive_processing() {
    let out = run_ore("showcase107.ore");
    assert!(out.contains("sum_digits(12345) = 15"));
    assert!(out.contains("digital_root(493) = 7"));
    assert!(out.contains("reverse(12345) = 54321"));
    assert!(out.contains("Armstrong numbers < 1000: 1, 2, 3, 4, 5, 6, 7, 8, 9, 153, 370, 371, 407"));
    assert!(out.contains("2^10 = 1024"));
}

#[test]
fn showcase108_type_system() {
    let out = run_ore("showcase108.ore");
    assert!(out.contains("red"));
    assert!(out.contains("rgb(128,0,255)"));
    assert!(out.contains("Rex the Labrador says woof!"));
    assert!(out.contains("Whiskers the indoor cat says meow!"));
    assert!(out.contains("Clownfish says blub!"));
    assert!(out.contains("x = 42"));
    assert!(out.contains("y is none"));
    assert!(out.contains("ok: 100"));
    assert!(out.contains("err: something went wrong"));
}

#[test]
fn showcase109_string_builder() {
    let out = run_ore("showcase109.ore");
    assert!(out.contains("+-------------+"));
    assert!(out.contains("| Hello, Ore! |"));
    assert!(out.contains("|   Welcome   |"));
    assert!(out.contains("Status Report"));
    assert!(out.contains("compile"));
    assert!(out.contains("All systems go!"));
}

#[test]
fn showcase110_performance() {
    let out = run_ore("showcase110.ore");
    assert!(out.contains("Primes < 100: 25"));
    assert!(out.contains("Primes < 1000: 168"));
    assert!(out.contains("Primes < 10000: 1229"));
    assert!(out.contains("Matrix product: [19, 22, 43, 50]"));
    assert!(out.contains("fib(10) = 55"));
    assert!(out.contains("fib(40) = 102334155"));
    assert!(out.contains("sum of squares 1..1000 = 333833500"));
    assert!(out.contains("match: true"));
}

#[test]
fn showcase111_advanced_lists() {
    let out = run_ore("showcase111.ore");
    assert!(out.contains("1, 2, 3, 5, 8, 9"));
    assert!(out.contains("1, 2, 3, 4"));
    assert!(out.contains("10, 20, 30"));
    assert!(out.contains("60, 70, 80"));
    assert!(out.contains("1, 2, 3, 4, 5, 6"));
    assert!(out.contains("hello-world-ore"));
    assert!(out.contains("has_big: true"));
    assert!(out.contains("all_pos: true"));
}

#[test]
fn showcase112_math() {
    let out = run_ore("showcase112.ore");
    assert!(out.contains("12.0"));
    assert!(out.contains("1024.0"));
    assert!(out.contains("3.14159"));
    assert!(out.contains("65536"));
    assert!(out.contains("42.0"));
    assert!(out.contains("456"));
}

#[test]
fn showcase113_concurrency() {
    let out = run_ore("showcase113.ore");
    assert!(out.contains("10, 20, 30, 40, 50"));
    assert!(out.contains("sum: 100"));
}

#[test]
fn showcase114_pipelines() {
    let out = run_ore("showcase114.ore");
    assert!(out.contains("20"));
    assert!(out.contains("4, 8, 12, 16, 20"));
    assert!(out.contains("HELLO + WORLD + LANG"));
    assert!(out.contains("total: 15"));
    assert!(out.contains("HELLO, ORE WORLD!"));
}

#[test]
fn showcase115_maps() {
    let out = run_ore("showcase115.ore");
    assert!(out.contains("95"));
    assert!(out.contains("host = localhost"));
    assert!(out.contains("color: red"));
    assert!(out.contains("apple: 3"));
    assert!(out.contains("banana: 2"));
}

#[test]
fn showcase116_imports() {
    let out = run_ore("showcase116.ore");
    assert!(out.contains("5! = 120"));
    assert!(out.contains("10! = 3628800"));
    assert!(out.contains("gcd(12, 8) = 4"));
    assert!(out.contains("lcm(12, 8) = 24"));
    assert!(out.contains("17 is prime"));
    assert!(out.contains("20 is not prime"));
}

#[test]
fn showcase117_testing() {
    let path = fixtures_dir().join("showcase117.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["test", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore test");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("4 passed"), "expected '4 passed' in stderr: {}", stderr);
    assert!(stderr.contains("0 failed"));
}

#[test]
fn showcase118_comprehensions() {
    let out = run_ore("showcase118.ore");
    assert!(out.contains("1, 4, 9, 16, 25"));
    assert!(out.contains("4, 16, 36, 64, 100"));
    assert!(out.contains("quick, brown, jumps, over, lazy"));
}

#[test]
fn showcase119_data_processing() {
    let out = run_ore("showcase119.ore");
    assert!(out.contains("Employee Performance Report"));
    assert!(out.contains("Alice"));
    assert!(out.contains("[ACTIVE]"));
    assert!(out.contains("Total:   391"));
    assert!(out.contains("Average: 78"));
}

#[test]
fn showcase120_iterators() {
    let out = run_ore("showcase120.ore");
    assert!(out.contains("1, 2, Fizz, 4, Buzz"));
    assert!(out.contains("FizzBuzz"));
    assert!(out.contains("collatz(9) = 19 steps"));
    assert!(out.contains("6 is perfect"));
    assert!(out.contains("28 is perfect"));
    assert!(out.contains("1 3 3 1"));
}

#[test]
fn showcase121_chained_if_then_else() {
    let out = run_ore("showcase121.ore");
    assert!(out.contains("-5C: freezing"));
    assert!(out.contains("25C: warm"));
    assert!(out.contains("95: A"));
    assert!(out.contains("50: F"));
    assert!(out.contains("-3: negative"));
    assert!(out.contains("0: zero"));
    assert!(out.contains("42 is even"));
}

#[test]
fn showcase122_generic_data_structures() {
    let out = run_ore("showcase122.ore");
    assert!(out.contains("top: 30"));
    assert!(out.contains("after pop: 20"));
    assert!(out.contains("front: 1"));
    assert!(out.contains("after dequeue: 2"));
    assert!(out.contains("string stack: a, b, c"));
}

#[test]
fn showcase123_typed_list_params() {
    let out = run_ore("showcase123.ore");
    assert!(out.contains("Alice, Bob, Carol, Dave"));
    assert!(out.contains("sum: 150"));
    assert!(out.contains("longest: rhinoceros"));
    assert!(out.contains("X, Y, Z"));
}

#[test]
fn showcase124_option_result() {
    let out = run_ore("showcase124.ore");
    assert!(out.contains("ID 1: Alice"));
    assert!(out.contains("ID 3: not found"));
    assert!(out.contains("10 / 3 = 3"));
    assert!(out.contains("10 / 0: division by zero"));
    assert!(out.contains("x: 42"));
    assert!(out.contains("y: -1"));
    assert!(out.contains("z: unknown"));
}

#[test]
fn showcase125_impl_blocks() {
    let out = run_ore("showcase125.ore");
    assert!(out.contains("v1 magnitude: 5.0"));
    assert!(out.contains("v1 + v2 = (4.0, 6.0)"));
    assert!(out.contains("v1 . v2 = 11.0"));
    assert!(out.contains("Counter(0, step=5)"));
    assert!(out.contains("Counter(20, step=5)"));
}

#[test]
fn showcase126_self_type() {
    let out = run_ore("showcase126.ore");
    assert!(out.contains("rgb(255, 0, 0)"));
    assert!(out.contains("rgb(127, 127, 0)"));
    assert!(out.contains("area: 15.0"));
    assert!(out.contains("scaled area: 60.0"));
    assert!(out.contains("is square: true"));
}

#[test]
fn showcase127_result_methods() {
    let out = run_ore("showcase127.ore");
    assert!(out.contains("r1: 42"));
    assert!(out.contains("r2: -1"));
    assert!(out.contains("ok.is_ok: true"));
    assert!(out.contains("err.is_err: true"));
    assert!(out.contains("sqrt(16): 4.0"));
    assert!(out.contains("sqrt(-4): 0.0"));
    assert!(out.contains("42 -> 42"));
    assert!(out.contains("-5 -> error: not a positive number"));
}

#[test]
fn showcase128_builder_pattern() {
    let out = run_ore("showcase128.ore");
    assert!(out.contains("host=example.com"));
    assert!(out.contains("port=443"));
    assert!(out.contains("debug=true"));
    assert!(out.contains("timeout=60s"));
    assert!(out.contains("host=localhost"));
}

#[test]
fn showcase129_all_types() {
    let out = run_ore("showcase129.ore");
    assert!(out.contains("Int: 42"));
    assert!(out.contains("Float: 3.14"));
    assert!(out.contains("Bool: true"));
    assert!(out.contains("Option: 99"));
    assert!(out.contains("Result: 42"));
    assert!(out.contains("Multiples of 3 squared: 9, 36, 81"));
    assert!(out.contains("2^10: 1024"));
}

#[test]
fn showcase130_file_io_json() {
    let out = run_ore("showcase130.ore");
    assert!(out.contains("file written: true"));
    assert!(out.contains("Hello from Ore!"));
    assert!(out.contains("lines: 3"));
    assert!(out.contains("after append: 4 lines"));
    assert!(out.contains("parsed keys: awesome, name, version"));
}

#[test]
fn showcase131_stress_test() {
    let out = run_ore("showcase131.ore");
    assert!(out.contains("fib(12) = 144"));
    assert!(out.contains("gcd(48, 36) = 12"));
    assert!(out.contains("Quick Brown Jumps Over Lazy"));
    assert!(out.contains("total items: 58"));
    assert!(out.contains("0: zero"));
    assert!(out.contains("15: large"));
}

#[test]
fn showcase132_enum_float_fields() {
    let out = run_ore("showcase132.ore");
    assert!(out.contains("circle r=5.0"));
    assert!(out.contains("area: 78.53975"));
    assert!(out.contains("rect 3.0x4.0"));
    assert!(out.contains("area: 12.0"));
    assert!(out.contains("point"));
    assert!(out.contains("area: 0.0"));
    assert!(out.contains("100.5 meters"));
    assert!(out.contains("23.7 C"));
    assert!(out.contains("42 items"));
    assert!(out.contains("combined area: 29.0"));
    assert!(out.contains("circumference: 62.8318"));
}

#[test]
fn showcase133_result_map_unwrap() {
    let out = run_ore("showcase133.ore");
    assert!(out.contains("10/2 doubled: 10"));
    assert!(out.contains("10/0 doubled is_err: true"));
    assert!(out.contains("10/0 doubled fallback: -1"));
    assert!(out.contains("chained: 90"));
    assert!(out.contains("unwrap: 7"));
    assert!(out.contains("Ok is_ok: true"));
    assert!(out.contains("Err is_ok: false"));
    assert!(out.contains("Ok unwrap_or: 4"));
    assert!(out.contains("Err unwrap_or: 0"));
}

#[test]
fn showcase134_error_handling() {
    let out = run_ore("showcase134.ore");
    assert!(out.contains("95 -> A"));
    assert!(out.contains("55 -> F"));
    assert!(out.contains("-5 -> -1"));
    assert!(out.contains("88 doubled: 176"));
    assert!(out.contains("75 + 10, * 2: 170"));
    assert!(out.contains("Some(42) map +1: 43"));
    assert!(out.contains("None is_none: true"));
}

#[test]
fn showcase135_pattern_matching() {
    let out = run_ore("showcase135.ore");
    assert!(out.contains("red"));
    assert!(out.contains("rgb(128, 64, 255)"));
    assert!(out.contains("42 = 42"));
    assert!(out.contains("3 + 4 = 7"));
    assert!(out.contains("5 * 6 = 30"));
    assert!(out.contains("3: fizz"));
    assert!(out.contains("0 is zero"));
    assert!(out.contains("25 is large"));
}

#[test]
fn showcase136_functional_patterns() {
    let out = run_ore("showcase136.ore");
    assert!(out.contains("apply_twice (*2) 3: 12"));
    assert!(out.contains("even squares: 4 + 16 + 36 + 64 + 100"));
    assert!(out.contains("sum of even squares: 220"));
    assert!(out.contains("10!: 3628800"));
    assert!(out.contains("product: 210"));
    assert!(out.contains("longest word: quick"));
}

#[test]
fn showcase137_match_guards() {
    let out = run_ore("showcase137.ore");
    assert!(out.contains("-5: negative"));
    assert!(out.contains("0: zero"));
    assert!(out.contains("3: small"));
    assert!(out.contains("42: medium"));
    assert!(out.contains("100: large"));
    assert!(out.contains("FizzBuzz"));
    assert!(out.contains("hi: tiny"));
    assert!(out.contains("greetings: long"));
}

#[test]
fn showcase138_utility_functions() {
    let out = run_ore("showcase138.ore");
    assert!(out.contains("racecar: palindrome? yes"));
    assert!(out.contains("hello: palindrome? no"));
    assert!(out.contains("encrypted: Khoor Zruog"));
    assert!(out.contains("decrypted: Hello World"));
    assert!(out.contains("Alice from NYC"));
}

#[test]
fn showcase139_map_processing() {
    let out = run_ore("showcase139.ore");
    assert!(out.contains("hello: 3"));
    assert!(out.contains("total items: 158"));
    assert!(out.contains("after merge: 6 types"));
    assert!(out.contains("has sword: true"));
    assert!(out.contains("wand count: 0"));
    assert!(out.contains("Alice: 95"));
}

#[test]
fn showcase140_test_driven() {
    // This uses `ore test`, not `ore run`
    let path = fixtures_dir().join("showcase140.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["test", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore test");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "ore test failed: {}", stderr);
    assert!(stderr.contains("5 passed, 0 failed"));
}

#[test]
fn showcase141_impl_blocks() {
    let out = run_ore("showcase141.ore");
    assert!(out.contains("a.length = 5.0"));
    assert!(out.contains("a dot b = 11.0"));
    assert!(out.contains("counter: 8"));
    assert!(out.contains("distance from origin: 13.0"));
}

#[test]
fn showcase142_aoc_style() {
    let out = run_ore("showcase142.ore");
    assert!(out.contains("abc1def2ghi3 -> 13"));
    assert!(out.contains("no_digits -> 0"));
    assert!(out.contains("banana: 'a' appears 3 times"));
    assert!(out.contains("(-5, 5) -> distance 10"));
    assert!(out.contains("# count in grid: 8"));
}

#[test]
fn showcase143_string_processing() {
    let out = run_ore("showcase143.ore");
    assert!(out.contains("Hello World From Ore"));
    assert!(out.contains("hello_world_from_ore"));
    assert!(out.contains("four three two one"));
    assert!(out.contains("3 vowels"));
    assert!(out.contains("pad_left: '000042'"));
    assert!(out.contains("pad_right: 'hi....'"));
}

#[test]
fn showcase144_recursion_algorithms() {
    let out = run_ore("showcase144.ore");
    assert!(out.contains("found 23 at index 5"));
    assert!(out.contains("1 not found"));
    assert!(out.contains("2^10 mod 1000 = 24"));
    assert!(out.contains("collatz(27): 111 steps"));
}

#[test]
fn showcase145_enum_match_guards() {
    let out = run_ore("showcase145.ore");
    assert!(out.contains("200: success"));
    assert!(out.contains("201: created"));
    assert!(out.contains("301: other success (301)"));
    assert!(out.contains("404: not found"));
    assert!(out.contains("503: error (503)"));
    assert!(out.contains("zero"));
    assert!(out.contains("negative"));
    assert!(out.contains("3 + 4"));
}

#[test]
fn showcase146_data_pipeline() {
    let out = run_ore("showcase146.ore");
    assert!(out.contains("total: 1800"));
    assert!(out.contains("top 3: 420, 310, 260"));
    assert!(out.contains("unique: 8"));
    assert!(out.contains("most common: be (2x)"));
    assert!(out.contains("prime sum: 77"));
    assert!(out.contains("sum of squares of multiples of 15 (1-100): 20475"));
}

#[test]
fn showcase147_comprehensions() {
    let out = run_ore("showcase147.ore");
    assert!(out.contains("squares: 1, 4, 9, 16, 25"));
    assert!(out.contains("doubled odds: 2, 6, 10, 14, 18"));
    assert!(out.contains("x=3 y=4 z=5"));
    assert!(out.contains("all even: true"));
    assert!(out.contains("any > 10: false"));
}

#[test]
fn showcase148_concurrency() {
    let out = run_ore("showcase148.ore");
    assert!(out.contains("total: 70300"));
    assert!(out.contains("match: true"));
}

#[test]
fn showcase149_generics() {
    let out = run_ore("showcase149.ore");
    assert!(out.contains("42"));
    assert!(out.contains("hello"));
    assert!(out.contains("swap [1,2]: 2, 1"));
    assert!(out.contains("repeat 3 x5: 3, 3, 3, 3, 3"));
}

#[test]
fn showcase150_feature_summary() {
    let out = run_ore("showcase150.ore");
    assert!(out.contains("even squares: 4, 16, 36, 64, 100"));
    assert!(out.contains("sum 1..100: 5050"));
    assert!(out.contains("circle area: 78.5398"));
    assert!(out.contains("distance: 5.0"));
    assert!(out.contains("Some map: 43"));
    assert!(out.contains("10/3 map *2: 6"));
    assert!(out.contains("0: zero"));
    assert!(out.contains("150 showcases"));
}

#[test]
fn showcase151_mutable_data() {
    let out = run_ore("showcase151.ore");
    assert!(out.contains("stack: 1, 2, 3"));
    assert!(out.contains("popped: 3"));
    assert!(out.contains("primes < 30: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29"));
    assert!(out.contains("the: 3"));
    assert!(out.contains("max: 9, increases: 3"));
}

#[test]
fn showcase152_math_numerical() {
    let out = run_ore("showcase152.ore");
    assert!(out.contains("perfect numbers < 1000: 6, 28, 496"));
    assert!(out.contains("gcd(12, 8) = 4, lcm = 24"));
    assert!(out.contains("fibonacci < 100: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89"));
    assert!(out.contains("digit_sum(123) = 6"));
    assert!(out.contains("powers of 2: 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024"));
}

#[test]
fn showcase153_matrix_ops() {
    let out = run_ore("showcase153.ore");
    assert!(out.contains("10 10 10"));
    assert!(out.contains("row 0: 6"));
    assert!(out.contains("trace: 15"));
    assert!(out.contains("diagonal: 1, 5, 9"));
}

#[test]
fn showcase154_state_machine() {
    let out = run_ore("showcase154.ore");
    assert!(out.contains("idle"));
    assert!(out.contains("loading (25%)"));
    assert!(out.contains("complete (100)"));
    assert!(out.contains("more than halfway (75%)"));
    assert!(out.contains("just started (10%)"));
    assert!(out.contains("404: failed (404)"));
}

#[test]
fn showcase155_iterators() {
    let out = run_ore("showcase155.ore");
    assert!(out.contains("primes <= 50: 2, 3, 5, 7, 11, 13"));
    assert!(out.contains("count: 15"));
    assert!(out.contains("(3, 5)"));
    assert!(out.contains("(41, 43)"));
    assert!(out.contains("1-100 sum: 5050"));
    assert!(out.contains("running max: 0, 4, 7, 7, 9, 9, 9, 9, 9, 9, 10"));
}

#[test]
fn showcase156_literal_match_enums() {
    let out = run_ore("showcase156.ore");
    assert!(out.contains("success: 200"));
    assert!(out.contains("failure: 404"));
    assert!(out.contains("failure: 500"));
    assert!(out.contains("failure: 418"));
    assert!(out.contains("Monday"));
    assert!(out.contains("Wednesday"));
    assert!(out.contains("Weekend"));
    assert!(out.contains("zero"));
    assert!(out.contains("positive"));
    assert!(out.contains("negative"));
}

#[test]
fn showcase157_accumulator_patterns() {
    let out = run_ore("showcase157.ore");
    assert!(out.contains("running totals: 0, 10, 30, 60, 100, 150"));
    assert!(out.contains("after 3 values: avg = 6"));
    assert!(out.contains("FizzBuzz"));
    assert!(out.contains("Fizz"));
    assert!(out.contains("Buzz"));
    assert!(out.contains("collatz(27): 111 steps"));
}

#[test]
fn showcase158_functional_data() {
    let out = run_ore("showcase158.ore");
    assert!(out.contains("the: 3"));
    assert!(out.contains("even sum: 110"));
    assert!(out.contains("odd sum: 100"));
    assert!(out.contains("even squares: 4 + 16 + 36 + 64 + 100"));
    assert!(out.contains("total: 220"));
}

#[test]
fn showcase159_enum_expressions() {
    let out = run_ore("showcase159.ore");
    assert!(out.contains("42 = 42"));
    assert!(out.contains("3 + 4 = 7"));
    assert!(out.contains("5 * 6 = 30"));
    assert!(out.contains("-10 = -10"));
}

#[test]
fn showcase160_enum_collections() {
    let out = run_ore("showcase160.ore");
    assert!(out.contains("circle(r=5): area = 75"));
    assert!(out.contains("square(s=4): area = 16"));
    assert!(out.contains("triangle(b=6,h=3): area = 9"));
    assert!(out.contains("total area: 400"));
    assert!(out.contains("square(s=3): 9"));
}

#[test]
fn showcase161_command_pattern() {
    let out = run_ore("showcase161.ore");
    assert!(out.contains("add 10 -> 10"));
    assert!(out.contains("mul 3 -> 30"));
    assert!(out.contains("reset -> 0"));
    assert!(out.contains("add 42 -> 42"));
    assert!(out.contains("final state: 42"));
}

#[test]
fn showcase162_stats_records() {
    let out = run_ore("showcase162.ore");
    assert!(out.contains("count: 10"));
    assert!(out.contains("sum: 257"));
    assert!(out.contains("min: 4"));
    assert!(out.contains("max: 56"));
    assert!(out.contains("avg: 25"));
}

#[test]
fn showcase163_sorting_searching() {
    let out = run_ore("showcase163.ore");
    assert!(out.contains("sorted: 11, 12, 22, 25, 34, 64, 90"));
    assert!(out.contains("found 25 at index 3"));
    assert!(out.contains("50 not found"));
    assert!(out.contains("min: 1"));
    assert!(out.contains("max: 10"));
}

#[test]
fn showcase164_graph_bfs() {
    let out = run_ore("showcase164.ore");
    assert!(out.contains("0 -> 1 -> 2 -> 3 -> 4 -> 5"));
    assert!(out.contains("node 2: degree 3"));
    assert!(out.contains("graph is connected: yes"));
}

#[test]
fn showcase165_rpn_evaluator() {
    let out = run_ore("showcase165.ore");
    assert!(out.contains("expression: 3 + 5 * 2 - 1"));
    assert!(out.contains("result: 12"));
    assert!(out.contains("result: 26"));
    assert!(out.contains("result: 4"));
}

#[test]
fn showcase166_set_operations() {
    let out = run_ore("showcase166.ore");
    assert!(out.contains("banana"));
    assert!(out.contains("date"));
    assert!(out.contains("size: 6"));
}

#[test]
fn showcase167_cellular_automaton() {
    let out = run_ore("showcase167.ore");
    assert!(out.contains("...............#..............."));
    assert!(out.contains("live cells: 13/31"));
}

#[test]
fn showcase168_higher_order() {
    let out = run_ore("showcase168.ore");
    assert!(out.contains("double twice 3: 12"));
    assert!(out.contains("square twice 3: 81"));
    assert!(out.contains("5! = 120"));
    assert!(out.contains("10! = 3628800"));
    assert!(out.contains("long words: HELLO, WORLD"));
}

#[test]
fn showcase169_result_handling() {
    let out = run_ore("showcase169.ore");
    assert!(out.contains("42 -> ok: 42"));
    assert!(out.contains("0 -> error: not positive"));
    assert!(out.contains("division by zero"));
    assert!(out.contains("ok(10) unwrap: 10"));
    assert!(out.contains("err unwrap_or(99): 99"));
}

#[test]
fn showcase170_data_transforms() {
    let out = run_ore("showcase170.ore");
    assert!(out.contains("3: ##### (5)"));
    assert!(out.contains("7 4 1"));
    assert!(out.contains("RLE: 1x3, 2x2, 3x4, 1x2"));
}

#[test]
fn showcase171_first_class_functions() {
    let out = run_ore("showcase171.ore");
    assert!(out.contains("apply(double, 5) = 10"));
    assert!(out.contains("apply(square, 4) = 16"));
    assert!(out.contains("apply_twice(double, 3) = 12"));
    assert!(out.contains("compose(double, square, 3) = 18"));
    assert!(out.contains("squares: 1, 4, 9, 16, 25"));
}

#[test]
fn showcase172_multiline_if_then_else() {
    let out = run_ore("showcase172.ore");
    assert!(out.contains("extreme heat"));
    assert!(out.contains("warm"));
    assert!(out.contains("freezing"));
    assert!(out.contains("95: A"));
    assert!(out.contains("45: F"));
    assert!(out.contains("FizzBuzz"));
}

#[test]
fn showcase173_config_and_csv() {
    let out = run_ore("showcase173.ore");
    assert!(out.contains("name: MyApp"));
    assert!(out.contains("port: 8080"));
    assert!(out.contains("Alice: score=95 grade=A"));
    assert!(out.contains("average: 78"));
    assert!(out.contains("total: 60"));
}

#[test]
fn showcase174_game_of_life() {
    let out = run_ore("showcase174.ore");
    assert!(out.contains("..#.."));
    assert!(out.contains(".###."));
    assert!(out.contains("live cells gen 0: 3"));
    assert!(out.contains("live cells gen 1: 3"));
}

#[test]
fn showcase175_bit_manipulation() {
    let out = run_ore("showcase175.ore");
    assert!(out.contains("42 -> 101010 (3 bits set)"));
    assert!(out.contains("255 -> 11111111 (8 bits set)"));
    assert!(out.contains("101010 -> 42"));
    assert!(out.contains("2^9 = 512"));
}

#[test]
fn showcase176_text_formatting() {
    let out = run_ore("showcase176.ore");
    assert!(out.contains("Hello World"));
    assert!(out.contains("+-------------+"));
    assert!(out.contains("The quick brown fox"));
}

#[test]
fn showcase177_stack_operations() {
    let out = run_ore("showcase177.ore");
    assert!(out.contains("5 -> 4 -> 3 -> 2 -> 1"));
    assert!(out.contains("pushed 30, top = 30, size = 3"));
    assert!(out.contains("popped 50, size = 4"));
}

#[test]
fn showcase178_stack_machine() {
    let out = run_ore("showcase178.ore");
    assert!(out.contains("49"));
    assert!(out.contains("7\n7"));
}

#[test]
fn showcase179_pipeline_showcase() {
    let out = run_ore("showcase179.ore");
    assert!(out.contains("primes < 50: 2, 3, 5, 7, 11, 13"));
    assert!(out.contains("sum: 328"));
    assert!(out.contains("shouting: HELLO WORLD FROM ORE LANGUAGE"));
    assert!(out.contains("1-100 sum: 5050"));
    assert!(out.contains("5! = 120"));
}

#[test]
fn showcase180_type_system() {
    let out = run_ore("showcase180.ore");
    assert!(out.contains("rgb(128,64,255)"));
    assert!(out.contains("N -> S"));
    assert!(out.contains("E -> W"));
    assert!(out.contains("bright red-ish"));
}

#[test]
fn showcase181_enum_list_integration() {
    let out = run_ore("showcase181.ore");
    assert!(out.contains("leaf(1): sum = 1"));
    assert!(out.contains("branch(5,6): sum = 11"));
    assert!(out.contains("[ERROR] priority=3"));
    assert!(out.contains("info: 2"));
}

#[test]
fn showcase182_string_operations() {
    let out = run_ore("showcase182.ore");
    assert!(out.contains("racecar: palindrome? yes"));
    assert!(out.contains("hello: palindrome? no"));
    assert!(out.contains("programming: 3 vowels"));
    assert!(out.contains("replace: Hello, Ore!"));
}

#[test]
fn showcase183_priority_queue() {
    let out = run_ore("showcase183.ore");
    assert!(out.contains("insert 8: 3, 8, 15, 28, 42, 67, 91"));
    assert!(out.contains("merged: 1, 2, 4, 5, 7, 8, 10, 11"));
}

#[test]
fn showcase184_testing_framework() {
    let out = run_ore("showcase184.ore");
    assert!(!out.contains("FAIL"));
    assert!(out.contains("PASS: 5!"));
    assert!(out.contains("PASS: sum of squares"));
}

#[test]
fn showcase185_maze_solver() {
    let out = run_ore("showcase185.ore");
    assert!(out.contains("Solution found!"));
    assert!(out.contains("Path length: 13"));
    assert!(out.contains("**#####"));
}

#[test]
fn showcase186_state_machine() {
    let out = run_ore("showcase186.ore");
    assert!(out.contains("unlock: locked -> closed"));
    assert!(out.contains("open: closed -> open"));
    assert!(out.contains("close: open -> closed"));
    assert!(out.contains("lock: closed -> locked"));
    assert!(out.contains("open: locked -> locked"));
}

#[test]
fn showcase187_mini_database() {
    let out = run_ore("showcase187.ore");
    assert!(out.contains("=== All Employees ==="));
    assert!(out.contains("alice: engineer"));
    assert!(out.contains("bob: designer"));
    assert!(out.contains("=== Role Counts ==="));
    assert!(out.contains("engineer: 2"));
    assert!(out.contains("designer: 2"));
    assert!(out.contains("Updated alice: senior engineer"));
    assert!(out.contains("After removing dave: 4 employees"));
}

#[test]
fn showcase188_math_puzzles() {
    let out = run_ore("showcase188.ore");
    assert!(out.contains("Is magic: true"));
    assert!(out.contains("Magic sum: 15"));
    assert!(out.contains("Is magic: false"));
    assert!(out.contains("Pascal's triangle:"));
    assert!(out.contains("1 4 6 4 1"));
}

#[test]
fn showcase189_functional_composition() {
    let out = run_ore("showcase189.ore");
    assert!(out.contains("sum of squares 1-100: 338350"));
    assert!(out.contains("sum of doubled multiples of 3 (1-50): 816"));
    assert!(out.contains("running sums: 0, 10, 30, 60, 100, 150"));
    assert!(out.contains("pipe3(5, *2, +10, *3) = 60"));
    assert!(out.contains("pipe3(3, ^2, -1, *4) = 32"));
}

#[test]
fn showcase190_number_theory() {
    let out = run_ore("showcase190.ore");
    assert!(out.contains("gcd(48, 18) = 6"));
    assert!(out.contains("lcm(12, 18) = 36"));
    assert!(out.contains("2 3 5 7 11 13"));
    assert!(out.contains("6 is perfect"));
    assert!(out.contains("28 is perfect"));
    assert!(out.contains("496 is perfect"));
}

#[test]
fn showcase191_matrix_operations() {
    let out = run_ore("showcase191.ore");
    assert!(out.contains("10 10 10"));
    assert!(out.contains("30 24 18"));
    assert!(out.contains("84 69 54"));
    assert!(out.contains("det(A) = 0"));
    assert!(out.contains("trace(A) = 15"));
}

#[test]
fn showcase192_text_analysis() {
    let out = run_ore("showcase192.ore");
    assert!(out.contains("Word count: 11"));
    assert!(out.contains("Vowels: 13"));
    assert!(out.contains("the: 3"));
    assert!(out.contains("fox: 2"));
    assert!(out.contains("Longest word: quick"));
}

#[test]
fn showcase193_coordinate_geometry() {
    let out = run_ore("showcase193.ore");
    assert!(out.contains("Distance p1-p2: 5.0"));
    assert!(out.contains("Triangle area: 12.0"));
    assert!(out.contains("Centroid:"));
}

#[test]
fn showcase194_iterator_patterns() {
    let out = run_ore("showcase194.ore");
    assert!(out.contains("1. Alice - 95"));
    assert!(out.contains("take_while < 6: 1, 3, 5"));
    assert!(out.contains("drop_while < 6: 7, 2, 4, 6, 8"));
    assert!(out.contains("Sum of all groups: 45"));
}

#[test]
fn showcase195_rpg_combat() {
    let out = run_ore("showcase195.ore");
    assert!(out.contains("=== RPG Combat ==="));
    assert!(out.contains("Player wins with 50 HP remaining!"));
}

#[test]
fn showcase196_encoding() {
    let out = run_ore("showcase196.ore");
    assert!(out.contains("Encoded (shift 3): defabc"));
    assert!(out.contains("Decoded: abcdef"));
    assert!(out.contains("RLE of 'aaabbbccddddee': 3a3b2c4d2e"));
}

#[test]
fn showcase197_recursive_data() {
    let out = run_ore("showcase197.ore");
    assert!(out.contains("2^10 = 1024"));
    assert!(out.contains("sum_digits(12345) = 15"));
    assert!(out.contains("reverse(12345) = 54321"));
    assert!(out.contains("1 2 3 4 5 6 7 8 9 11"));
}

#[test]
fn showcase198_sequences() {
    let out = run_ore("showcase198.ore");
    assert!(out.contains("0, 1, 1, 2, 3, 5, 8, 13, 21, 34"));
    assert!(out.contains("1, 3, 6, 10, 15, 21, 28, 36, 45, 55"));
    assert!(out.contains("Collatz(27): 111 steps"));
    assert!(out.contains("1, 3, 9, 27, 81, 243, 729, 2187"));
}

#[test]
fn showcase199_calculator() {
    let out = run_ore("showcase199.ore");
    assert!(out.contains("Postfix '3 4 + 2 * 1 -' = 13"));
    assert!(out.contains("Postfix '5 1 2 + 4 * + 3 -' = 14"));
    assert!(out.contains("(3 + 4) * 2 - 1 = 13"));
}

#[test]
fn showcase200_comprehensive() {
    let out = run_ore("showcase200.ore");
    assert!(out.contains("=== Ore Language Showcase #200 ==="));
    assert!(out.contains("red: #FF0000"));
    assert!(out.contains("Sum of evens 1-20: 110"));
    assert!(out.contains("Title case: The Quick Brown Fox"));
    assert!(out.contains("=== Showcase Complete ==="));
}

#[test]
fn showcase201_advanced_pipelines() {
    let out = run_ore("showcase201.ore");
    assert!(out.contains("144, 196, 256"));
    assert!(out.contains("5! = 120"));
    assert!(out.contains("10! = 3628800"));
    assert!(out.contains("Long words uppercased: HELLO | WORLD"));
    assert!(out.contains("Multiples of 7 in 1-100: 14"));
}

#[test]
fn showcase202_expression_tree() {
    let out = run_ore("showcase202.ore");
    assert!(out.contains("42 = 42"));
    assert!(out.contains("(3 + 4) = 7"));
    assert!(out.contains("(5 * 6) = 30"));
    assert!(out.contains("(a + 5) * 3 = 45"));
}

#[test]
fn showcase203_string_methods() {
    let out = run_ore("showcase203.ore");
    assert!(out.contains("trim: 'Hello, World!'"));
    assert!(out.contains("to_upper: 'HELLO, WORLD!'"));
    assert!(out.contains("replace: Hello, Ore!"));
    assert!(out.contains("repeat: hahaha"));
    assert!(out.contains("reverse: fedcba"));
    assert!(out.contains("pad_left: '000042'"));
}

#[test]
fn showcase204_nested_loops() {
    let out = run_ore("showcase204.ore");
    assert!(out.contains("Multiplication table"));
    assert!(out.contains(" 5 10 15 20 25"));
    assert!(out.contains("*********"));
    assert!(out.contains("11 12 13 14 15"));
}

#[test]
fn showcase205_graph() {
    let out = run_ore("showcase205.ore");
    assert!(out.contains("Total edges: 6"));
    assert!(out.contains("Max degree: 3"));
    assert!(out.contains("Node B has max degree"));
}

#[test]
fn showcase206_tree_enum() {
    let out = run_ore("showcase206.ore");
    assert!(out.contains("Leaf(1) -> sum = 1"));
    assert!(out.contains("Node(3, 1, 2) -> sum = 6"));
    assert!(out.contains("Big tree sum: 60"));
    assert!(out.contains("Total of all trees: 9"));
}

#[test]
fn showcase207_population() {
    let out = run_ore("showcase207.ore");
    assert!(out.contains("Year 0: 100"));
    assert!(out.contains("Year 10: 256"));
    assert!(out.contains("Predator-Prey simulation:"));
}

#[test]
fn showcase208_binary() {
    let out = run_ore("showcase208.ore");
    assert!(out.contains("42 = 101010 (3 ones)"));
    assert!(out.contains("255 = 11111111 (8 ones)"));
    assert!(out.contains("0 XOR 0 = 0"));
    assert!(out.contains("1 XOR 1 = 0"));
}

#[test]
fn showcase209_statistics() {
    let out = run_ore("showcase209.ore");
    assert!(out.contains("Sum: 468"));
    assert!(out.contains("Mean: 46"));
    assert!(out.contains("Min: 12"));
    assert!(out.contains("Max: 89"));
    assert!(out.contains("Sorted: 12, 21, 23, 34, 43, 45, 56, 67, 78, 89"));
}

#[test]
fn showcase210_pattern_matching() {
    let out = run_ore("showcase210.ore");
    assert!(out.contains("circle with radius 5: area=78"));
    assert!(out.contains("rectangle 10x10: area=100"));
    assert!(out.contains("Large shapes: 2"));
    assert!(out.contains("Total area: 233"));
}

#[test]
fn showcase211_vectors() {
    let out = run_ore("showcase211.ore");
    assert!(out.contains("a = (3.0, 4.0)"));
    assert!(out.contains("a + b = (4.0, 6.0)"));
    assert!(out.contains("a * 2 = (6.0, 8.0)"));
    assert!(out.contains("a . b = 11.0"));
    assert!(out.contains("|a| = 5.0"));
}

#[test]
fn showcase212_advent_of_code() {
    let out = run_ore("showcase212.ore");
    assert!(out.contains("1abc2 -> 12"));
    assert!(out.contains("Total: 142"));
    assert!(out.contains("Valid passwords: 2"));
}

#[test]
fn showcase213_list_operations() {
    let out = run_ore("showcase213.ore");
    assert!(out.contains("Evens: 2, 4, 6, 8"));
    assert!(out.contains("Sum: 210"));
    assert!(out.contains("flat_map [1,2,3] -> [n, n*10]: 1, 10, 2, 20, 3, 30"));
    assert!(out.contains("any > 15: true"));
    assert!(out.contains("all > 5: false"));
    assert!(out.contains("Reverse 1-5: 5, 4, 3, 2, 1"));
}

#[test]
fn showcase214_error_handling() {
    let out = run_ore("showcase214.ore");
    assert!(out.contains("Ok(5)"));
    assert!(out.contains("Err(division by zero)"));
    assert!(out.contains("100 / 5 / 2 = Ok(10)"));
    assert!(out.contains("Successes: 4, Failures: 2"));
}

#[test]
fn showcase215_sorting() {
    let out = run_ore("showcase215.ore");
    assert!(out.contains("Bubble sort: 11, 12, 22, 25, 34, 64, 90"));
    assert!(out.contains("Selection sort: 11, 12, 22, 25, 34, 64, 90"));
    assert!(out.contains("Insertion sort: 11, 12, 22, 25, 34, 64, 90"));
    assert!(out.contains("All sorts agree: true"));
}

#[test]
fn showcase216_game_of_life() {
    let out = run_ore("showcase216.ore");
    assert!(out.contains("Generation 0:"));
    assert!(out.contains(".###...."));
    assert!(out.contains("Generation 4:"));
}

#[test]
fn showcase217_string_builder() {
    let out = run_ore("showcase217.ore");
    assert!(out.contains("| Hello, Ore! |"));
    assert!(out.contains("| Welcome |"));
    assert!(out.contains("| Alice"));
    assert!(out.contains("| Bob"));
}

#[test]
fn showcase218_recursive() {
    let out = run_ore("showcase218.ore");
    assert!(out.contains("Move disk 3 from A to C"));
    assert!(out.contains("A(2,2) = 7"));
    assert!(out.contains("A(3,3) = 61"));
    assert!(out.contains("F(11) = 89"));
}

#[test]
fn showcase219_pipeline() {
    let out = run_ore("showcase219.ore");
    assert!(out.contains("5 |> double |> double |> double = 40"));
    assert!(out.contains("3 |> square |> double |> negate = -18"));
    assert!(out.contains("Even squares: 4, 16, 36"));
    assert!(out.contains("*** HELLO ***"));
}

#[test]
fn showcase220_frequency() {
    let out = run_ore("showcase220.ore");
    assert!(out.contains("a: ##### (5)"));
    assert!(out.contains("b: ## (2)"));
    assert!(out.contains("to: 3"));
    assert!(out.contains("be: 3"));
    assert!(out.contains("Most common:"));
}

#[test]
fn showcase221_mutable_state() {
    let out = run_ore("showcase221.ore");
    assert!(out.contains("Stack: 10, 20, 30"));
    assert!(out.contains("Popped: 30"));
    assert!(out.contains("Dequeued: 1"));
    assert!(out.contains("Sum of squares 1-10: 385"));
}

#[test]
fn showcase222_float_math() {
    let out = run_ore("showcase222.ore");
    assert!(out.contains("pi + e = 5.85987"));
    assert!(out.contains("Area = 78.53975"));
    assert!(out.contains("3^2 + 4^2 = 5.0"));
    assert!(out.contains("x1 = 3.0"));
    assert!(out.contains("x2 = 2.0"));
}

#[test]
fn showcase223_complex_enums() {
    let out = run_ore("showcase223.ore");
    assert!(out.contains("Expression: 3 + 4 * 2 - 1"));
    assert!(out.contains("Operators: 3, Numbers: 4"));
}

#[test]
fn showcase224_data_analysis() {
    let out = run_ore("showcase224.ore");
    assert!(out.contains("Sum: 510"));
    assert!(out.contains("Mean: 51"));
    assert!(out.contains("Sorted: 12, 15, 23, 34, 45, 56, 67, 78, 89, 91"));
    assert!(out.contains("IQR: 55"));
}

#[test]
fn showcase225_validation() {
    let out = run_ore("showcase225.ore");
    assert!(out.contains("user@example.com: VALID"));
    assert!(out.contains("bad-email: INVALID"));
    assert!(out.contains("'hello': alpha"));
    assert!(out.contains("'123': numeric"));
    assert!(out.contains("'': empty"));
}

#[test]
fn showcase226_rgb_records() {
    let out = run_ore("showcase226.ore");
    assert!(out.contains("Red: rgb(255, 0, 0)"));
    assert!(out.contains("White: rgb(255, 255, 255), brightness=255"));
    assert!(out.contains("Red + Green = rgb(127, 127, 0)"));
}

#[test]
fn showcase227_loop_patterns() {
    let out = run_ore("showcase227.ore");
    assert!(out.contains("First n where n^2 > 100: 11"));
    assert!(out.contains("3 * 14 = 42"));
    assert!(out.contains("Liftoff!"));
    assert!(out.contains("sum = 105"));
}

#[test]
fn showcase228_parser() {
    let out = run_ore("showcase228.ore");
    assert!(out.contains("parse('123') = 123"));
    assert!(out.contains("parse('007') = 7"));
    assert!(out.contains("15 + 27 = 42"));
    assert!(out.contains("15 * 27 = 405"));
}

#[test]
fn showcase229_scan() {
    let out = run_ore("showcase229.ore");
    assert!(out.contains("Running sum: 0, 5, 8, 16, 17, 26, 30, 37, 39, 45"));
    assert!(out.contains("Deltas: 5, -3, 8, -2, 7"));
    assert!(out.contains("Cumulative product: 1, 2, 6, 24, 120"));
}

#[test]
fn showcase230_map_operations() {
    let out = run_ore("showcase230.ore");
    assert!(out.contains("fib(10) = 55"));
    assert!(out.contains("Cache size: 15"));
    assert!(out.contains("Contains fib(10): true"));
    assert!(out.contains("Contains fib(20): false"));
    assert!(out.contains("the: 3"));
}

#[test]
fn showcase231_type_conversion() {
    let out = run_ore("showcase231.ore");
    assert!(out.contains("Int 42 -> Float 42.0"));
    assert!(out.contains("5.pow(3) = 125"));
    assert!(out.contains("clamp(15, 0, 10) = 10"));
    assert!(out.contains("min(3, 7) = 3"));
}

#[test]
fn showcase232_string_ops() {
    let out = run_ore("showcase232.ore");
    assert!(out.contains("strip_prefix '/home': /user/file.txt"));
    assert!(out.contains("strip_suffix '.txt': /home/user/file"));
    assert!(out.contains("index_of 'world'"));
    assert!(out.contains("slice(0, 5): 'hello'"));
}

#[test]
fn showcase233_traffic_light() {
    let out = run_ore("showcase233.ore");
    assert!(out.contains("RED for 30s"));
    assert!(out.contains("GREEN for 25s"));
    assert!(out.contains("YELLOW for 5s"));
    assert!(out.contains("RED: YES"));
    assert!(out.contains("GREEN: NO"));
}

#[test]
fn showcase234_heap() {
    let out = run_ore("showcase234.ore");
    assert!(out.contains("Max-Heap: 30, 20, 25, 10, 15, 5, 8"));
    assert!(out.contains("Root (max): 30"));
}

#[test]
fn showcase235_functional() {
    let out = run_ore("showcase235.ore");
    assert!(out.contains("compose(+1, *2)(5) = 12"));
    assert!(out.contains("double^5(1) = 32"));
    assert!(out.contains("Sum of evens (1-10): 30"));
}

#[test]
fn showcase236_cards() {
    let out = run_ore("showcase236.ore");
    assert!(out.contains("AH (red)"));
    assert!(out.contains("KS (black)"));
    assert!(out.contains("95: A"));
    assert!(out.contains("55: F"));
}

#[test]
fn showcase237_patterns() {
    let out = run_ore("showcase237.ore");
    assert!(out.contains("#############"));
    assert!(out.contains("Checkerboard"));
    assert!(out.contains("##  ##"));
}

#[test]
fn showcase238_intervals() {
    let out = run_ore("showcase238.ore");
    assert!(out.contains("[1,5) & [3,7): true"));
    assert!(out.contains("[1,3) & [5,7): false"));
    assert!(out.contains("Total covered: 10"));
    assert!(out.contains("..######..####......"));
}

#[test]
fn showcase239_turtle() {
    let out = run_ore("showcase239.ore");
    assert!(out.contains("Final position: (9, 4)"));
    assert!(out.contains("Distance from start: 5"));
}

#[test]
fn showcase240_pipeline_showcase() {
    let out = run_ore("showcase240.ore");
    assert!(out.contains("sum: 210"));
    assert!(out.contains("fold (*): 120"));
    assert!(out.contains("take(5): 1, 2, 3, 4, 5"));
    assert!(out.contains("scan(+): 0, 1, 3, 6, 10, 15"));
    assert!(out.contains("flat_map: 1, 10, 2, 20, 3, 30"));
    assert!(out.contains("reverse: 5, 4, 3, 2, 1"));
}

#[test]
fn showcase241_cipher() {
    let out = run_ore("showcase241.ore");
    assert!(out.contains("'hello' -> 'uryyb' -> 'hello'"));
    assert!(out.contains("'world' -> 'jbeyq' -> 'world'"));
}

#[test]
fn showcase242_bases() {
    let out = run_ore("showcase242.ore");
    assert!(out.contains("255"));
    assert!(out.contains("11111111"));
    assert!(out.contains("ff"));
}

#[test]
fn showcase243_brainfuck() {
    let out = run_ore("showcase243.ore");
    assert!(out.contains("Cell 0 after 10 increments: 10"));
    assert!(out.contains("Cell 1 after loop: 70"));
    assert!(out.contains("Cell 1 final: 72"));
}

#[test]
fn showcase244_patterns() {
    let out = run_ore("showcase244.ore");
    assert!(out.contains("Sierpinski Triangle"));
    assert!(out.contains("10! = 3628800"));
}

#[test]
fn showcase245_events() {
    let out = run_ore("showcase245.ore");
    assert!(out.contains("Click(100, 200)"));
    assert!(out.contains("Key 'a' pressed"));
    assert!(out.contains("resized to 1024x768"));
    assert!(out.contains("Clicks: 2, KeyPresses: 2, Resizes: 2"));
}

#[test]
fn showcase246_scheduler() {
    let out = run_ore("showcase246.ore");
    assert!(out.contains("Task B ran for 2 [DONE]"));
    assert!(out.contains("All tasks completed at t=12"));
}

#[test]
fn showcase247_financial() {
    let out = run_ore("showcase247.ore");
    assert!(out.contains("Year 10: 1628"));
    assert!(out.contains("Months to pay off: 57"));
}

#[test]
fn showcase248_matrix() {
    let out = run_ore("showcase248.ore");
    assert!(out.contains("1 0 0 0"));
    assert!(out.contains("F(10) = 89"));
    assert!(out.contains("Dot product"));
}

#[test]
fn showcase249_guessing() {
    let out = run_ore("showcase249.ore");
    assert!(out.contains("Guess 6: 73 - Correct!"));
    assert!(out.contains("Found in 6 guesses"));
    assert!(out.contains("Guess 1234: 4 bulls"));
}

#[test]
fn showcase250_grand_finale() {
    let out = run_ore("showcase250.ore");
    assert!(out.contains("=== Ore Language Showcase #250 ==="));
    assert!(out.contains("Rex the dog says Woof!"));
    assert!(out.contains("Sum of squares of evens (1-50): 22100"));
    assert!(out.contains("=== Showcase Complete ==="));
}

#[test]
fn showcase251_match_blocks() {
    let out = run_ore("showcase251.ore");
    assert!(out.contains("Moved to (1, 0)"));
    assert!(out.contains("Said: hello"));
    assert!(out.contains("Final position: (1, -1)"));
    assert!(out.contains("All messages: hello, bye"));
}

#[test]
fn showcase252_graph_bfs() {
    let out = run_ore("showcase252.ore");
    assert!(out.contains("A: B, C"));
    assert!(out.contains("Visiting E"));
    assert!(out.contains("Path found!"));
}

#[test]
fn showcase253_weekdays() {
    let out = run_ore("showcase253.ore");
    assert!(out.contains("6. Saturday (weekend)"));
    assert!(out.contains("Workdays: 5, Weekends: 2"));
}

#[test]
fn showcase254_interpolation() {
    let out = run_ore("showcase254.ore");
    assert!(out.contains("Sum: 10 + 20 = 30"));
    assert!(out.contains("Hello, World!"));
    assert!(out.contains("la la la la la"));
    assert!(out.contains("tick-tock-tick-tock-tick"));
}

#[test]
fn showcase255_transformations() {
    let out = run_ore("showcase255.ore");
    assert!(out.contains("Alice is 30 years old"));
    assert!(out.contains("a: apple, avocado, apricot"));
    assert!(out.contains("Unique: 3, 1, 4, 5, 9, 2, 6"));
}

#[test]
fn showcase256_nested_records() {
    let out = run_ore("showcase256.ore");
    assert!(out.contains("Area: 50.0"));
    assert!(out.contains("Perimeter: 30.0"));
    assert!(out.contains("Center: (5.0, 2.5)"));
}

#[test]
fn showcase257_control_flow() {
    let out = run_ore("showcase257.ore");
    assert!(out.contains("27: 111 steps"));
    assert!(out.contains("1 2 Fizz 4 Buzz Fizz"));
    assert!(out.contains("121"));
}

#[test]
fn showcase258_lambdas() {
    let out = run_ore("showcase258.ore");
    assert!(out.contains("double(5) = 10"));
    assert!(out.contains("Sum 1-100: 5050"));
    assert!(out.contains("Max of [5,2,8,1,9,3]: 9"));
    assert!(out.contains("Top 5 even squares: 4, 16, 36, 64, 100"));
}

#[test]
fn showcase259_maze() {
    let out = run_ore("showcase259.ore");
    assert!(out.contains("#############"));
    assert!(out.contains("Open cells: 39"));
    assert!(out.contains("Total: 91"));
}

#[test]
fn showcase260_features_tour() {
    let out = run_ore("showcase260.ore");
    assert!(out.contains("Factorials: 1, 120, 3628800"));
    assert!(out.contains("Cubes: 1, 8, 27, 64, 125"));
    assert!(out.contains("Replace: Hello, Ore!"));
    assert!(out.contains("42 = the answer"));
}

#[test]
fn showcase261_roman_numerals() {
    let out = run_ore("showcase261.ore");
    assert!(out.contains("42 -> XLII -> 42"));
    assert!(out.contains("1994 -> MCMXCIV -> 1994"));
    assert!(out.contains("3999 -> MMMCMXCIX -> 3999"));
}

#[test]
fn showcase262_rle_encoding() {
    let out = run_ore("showcase262.ore");
    assert!(out.contains("AAABBBCCDDDDDEEE -> 3A3B2C5D3E -> AAABBBCCDDDDDEEE"));
    assert!(out.contains("ABCDE -> ABCDE -> ABCDE"));
}

#[test]
fn showcase263_linked_list() {
    let out = run_ore("showcase263.ore");
    assert!(out.contains("10 -> 20 -> 30 -> 40 -> 50"));
    assert!(out.contains("50 -> 40 -> 30 -> 20 -> 10"));
    assert!(out.contains("After insert 25 at pos 2:"));
    assert!(out.contains("Find 20: index 1"));
}

#[test]
fn showcase264_vector_ops() {
    let out = run_ore("showcase264.ore");
    assert!(out.contains("Dot product [1,2,3].[4,5,6] = 32"));
    assert!(out.contains("Cross product: -3, 6, -3"));
    assert!(out.contains("Sum of squares: 55"));
}

#[test]
fn showcase265_text_statistics() {
    let out = run_ore("showcase265.ore");
    assert!(out.contains("Word count: 13"));
    assert!(out.contains("the: 4"));
    assert!(out.contains("Unique words: 8"));
}

#[test]
fn showcase266_stack_calculator() {
    let out = run_ore("showcase266.ore");
    assert!(out.contains("3 4 + 2 * 7 - = 7"));
    assert!(out.contains("5 1 2 + 4 * + 3 - = 14"));
    assert!(out.contains("0, 1, 1, 2, 3, 5, 8, 13, 21, 34"));
}

#[test]
fn showcase267_look_and_say() {
    let out = run_ore("showcase267.ore");
    assert!(out.contains("1: 1"));
    assert!(out.contains("4: 1211"));
    assert!(out.contains("8: 1113213211"));
}

#[test]
fn showcase268_histogram() {
    let out = run_ore("showcase268.ore");
    assert!(out.contains("Monthly Sales:"));
    assert!(out.contains("Distribution:"));
    assert!(out.contains("(6)"));
}

#[test]
fn showcase269_calendar() {
    let out = run_ore("showcase269.ore");
    assert!(out.contains("Mar 2024"));
    assert!(out.contains("Su Mo Tu We Th Fr Sa"));
    assert!(out.contains("2000 leap year: yes"));
    assert!(out.contains("1900 leap year: no"));
}

#[test]
fn showcase270_mini_database() {
    let out = run_ore("showcase270.ore");
    assert!(out.contains("Lookup 'Carol': index 2"));
    assert!(out.contains("Engineering team:"));
    assert!(out.contains("Eve: 95k"));
    assert!(out.contains("Sorted by salary:"));
}

#[test]
fn showcase271_collatz() {
    let out = run_ore("showcase271.ore");
    assert!(out.contains("Collatz(27): 112 steps"));
    assert!(out.contains("n=97, steps=118"));
}

#[test]
fn showcase272_sieve() {
    let out = run_ore("showcase272.ore");
    assert!(out.contains("Count: 25"));
    assert!(out.contains("Twin primes:"));
    assert!(out.contains("(3, 5)"));
}

#[test]
fn showcase273_pascal() {
    let out = run_ore("showcase273.ore");
    assert!(out.contains("Pascal's Triangle:"));
    assert!(out.contains("Row 0: sum = 1"));
    assert!(out.contains("Row 9: sum = 512"));
}

#[test]
fn showcase274_caesar() {
    let out = run_ore("showcase274.ore");
    assert!(out.contains("Shift 13: Uryyb Jbeyq -> Hello World"));
    assert!(out.contains("ROT13: Uryyb Jbeyq"));
    assert!(out.contains("FOUND!"));
}

#[test]
fn showcase275_set_ops() {
    let out = run_ore("showcase275.ore");
    assert!(out.contains("Union: 1, 2, 3, 4, 5, 6, 7, 8, 9"));
    assert!(out.contains("Intersection: 4, 5, 6"));
    assert!(out.contains("subset of A? true"));
}

#[test]
fn showcase276_queue() {
    let out = run_ore("showcase276.ore");
    assert!(out.contains("Serving: A"));
    assert!(out.contains("[0] Critical"));
    assert!(out.contains("[1] High-task"));
}

#[test]
fn showcase277_base_conversion() {
    let out = run_ore("showcase277.ore");
    assert!(out.contains("Binary:  11111111"));
    assert!(out.contains("Hex:     FF"));
    assert!(out.contains("12345 -> hex 3039 -> 12345"));
}

#[test]
fn showcase278_levenshtein() {
    let out = run_ore("showcase278.ore");
    assert!(out.contains("'kitten' -> 'sitting' = 3"));
    assert!(out.contains("'hello' -> 'hello' = 0"));
    assert!(out.contains("Closest to 'hello': 'hell' (distance 1)"));
}

#[test]
fn showcase279_spiral() {
    let out = run_ore("showcase279.ore");
    assert!(out.contains("Spiral matrix (5x5):"));
    assert!(out.contains("1"));
    assert!(out.contains("25"));
}

#[test]
fn showcase280_polynomial() {
    let out = run_ore("showcase280.ore");
    assert!(out.contains("p(x) = 2 + 3x + x^2"));
    assert!(out.contains("p + q = 3 + 2x + 3x^2"));
    assert!(out.contains("p'(x) = 3 + 2x"));
}

#[test]
fn showcase281_tic_tac_toe() {
    let out = run_ore("showcase281.ore");
    assert!(out.contains("Winner:"));
}

#[test]
fn showcase282_hash_map() {
    let out = run_ore("showcase282.ore");
    assert!(out.contains("apple -> bucket"));
    assert!(out.contains("Bucket occupancy:"));
}

#[test]
fn showcase283_postfix_eval() {
    let out = run_ore("showcase283.ore");
    assert!(out.contains("3 + 4 * 2 = 11"));
    assert!(out.contains("(3 + 4) * 2 = 14"));
}

#[test]
fn showcase284_game_of_life() {
    let out = run_ore("showcase284.ore");
    assert!(out.contains("Game of Life - Glider:"));
    assert!(out.contains("Generation 0:"));
    assert!(out.contains("Generation 4:"));
}

#[test]
fn showcase285_pattern_match() {
    let out = run_ore("showcase285.ore");
    assert!(out.contains("'Hello': found"));
    assert!(out.contains("Hello -> identifier"));
    assert!(out.contains("42 -> number"));
}

#[test]
fn showcase286_tree() {
    let out = run_ore("showcase286.ore");
    assert!(out.contains("DFS preorder: 1 -> 2 -> 4 -> 5 -> 3 -> 6 -> 7"));
    assert!(out.contains("Leaves: 4, 5, 6, 7"));
    assert!(out.contains("Sum: 28"));
}

#[test]
fn showcase287_interpreter() {
    let out = run_ore("showcase287.ore");
    assert!(out.contains("1 + 2 + 3 = 6"));
    assert!(out.contains("2 * 3 * 4 = 24"));
    assert!(out.contains("Tokens of"));
}

#[test]
fn showcase288_combinatorics() {
    let out = run_ore("showcase288.ore");
    assert!(out.contains("10! = 3628800"));
    assert!(out.contains("Catalan numbers:"));
    assert!(out.contains("C(0) = 1"));
}

#[test]
fn showcase289_graph() {
    let out = run_ore("showcase289.ore");
    assert!(out.contains("BFS from 0:"));
    assert!(out.contains("0 -> 1 -> 2 -> 3 -> 4 -> 5"));
    assert!(out.contains("Path 0->5: yes"));
}

#[test]
fn showcase290_hanoi() {
    let out = run_ore("showcase290.ore");
    assert!(out.contains("Move disk 1 from A to C"));
    assert!(out.contains("3 disks: 7 moves"));
    assert!(out.contains("10 disks: 1023 moves"));
}

#[test]
fn showcase291_morse() {
    let out = run_ore("showcase291.ore");
    assert!(out.contains("HELLO -> .... . .-.. .-.. ---"));
    assert!(out.contains("SOS -> ... --- ..."));
}

#[test]
fn showcase292_lcs() {
    let out = run_ore("showcase292.ore");
    assert!(out.contains("LCS('ABCBDAB', 'BDCAB') = 4"));
    assert!(out.contains("LCS('ABCDEF', 'ABCDEF') = 6"));
    assert!(out.contains("LCS('ABC', 'XYZ') = 0"));
}

#[test]
fn showcase293_sudoku() {
    let out = run_ore("showcase293.ore");
    assert!(out.contains("Sudoku is valid!"));
}

#[test]
fn showcase294_bits() {
    let out = run_ore("showcase294.ore");
    assert!(out.contains("255 (11111111): 8 bits set"));
    assert!(out.contains("128: yes"));
    assert!(out.contains("100: no"));
}

#[test]
fn showcase295_parser() {
    let out = run_ore("showcase295.ore");
    assert!(out.contains("Left-to-right: 224"));
    assert!(out.contains("With precedence: 176"));
}

#[test]
fn showcase296_determinant() {
    let out = run_ore("showcase296.ore");
    assert!(out.contains("= -2"));
    assert!(out.contains("Identity 3x3 det = 1"));
}

#[test]
fn showcase297_vigenere() {
    let out = run_ore("showcase297.ore");
    assert!(out.contains("Encrypted: RIJVS UYVJN"));
    assert!(out.contains("Decrypted: HELLO WORLD"));
}

#[test]
fn showcase298_queries() {
    let out = run_ore("showcase298.ore");
    assert!(out.contains("WHERE age > 30:"));
    assert!(out.contains("AVG salary by city:"));
    assert!(out.contains("ORDER BY salary DESC:"));
}

#[test]
fn showcase299_sequences() {
    let out = run_ore("showcase299.ore");
    assert!(out.contains("Perfect numbers up to 1000:"));
    assert!(out.contains("6,"));
    assert!(out.contains("Happy numbers"));
}

#[test]
fn showcase300_grand_finale() {
    let out = run_ore("showcase300.ore");
    assert!(out.contains("=== Ore Language - Showcase 300 ==="));
    assert!(out.contains("Sum 1-100: 5050"));
    assert!(out.contains("5! = 120"));
    assert!(out.contains("=== 300 showcases complete! ==="));
}

#[test]
fn showcase301_pipe_precedence() {
    let out = run_ore("showcase301.ore");
    assert!(out.contains("count > 3: true"));
    assert!(out.contains("empty list"));
    assert!(out.contains("pipe else then pipe: 10"));
}

#[test]
fn showcase302_formatting() {
    let out = run_ore("showcase302.ore");
    assert!(out.contains("Alice"));
    assert!(out.contains("Average age: 30"));
    assert!(out.contains("|-- src/"));
}

#[test]
fn showcase303_functional() {
    let out = run_ore("showcase303.ore");
    assert!(out.contains("Sum of even squares (1-10): 220"));
    assert!(out.contains("5! = 120"));
    assert!(out.contains("FizzBuzz count (1-100):"));
}

#[test]
fn showcase304_result() {
    let out = run_ore("showcase304.ore");
    assert!(out.contains("10 / 3 = 3"));
    assert!(out.contains("10 / 0 = -1"));
    assert!(out.contains("100/4 * 2 = 50"));
}

#[test]
fn showcase305_state_machine() {
    let out = run_ore("showcase305.ore");
    assert!(out.contains("Start: Idle"));
    assert!(out.contains("Running"));
    assert!(out.contains("Paused"));
}

#[test]
fn showcase306_iterators() {
    let out = run_ore("showcase306.ore");
    assert!(out.contains("[0] alpha"));
    assert!(out.contains("Running average:"));
    assert!(out.contains("Mapped x10: 10, 20, 30, 40, 50"));
}

#[test]
fn showcase307_impl() {
    let out = run_ore("showcase307.ore");
    assert!(out.contains("a = (3, 4)"));
    assert!(out.contains("|a|^2 = 25"));
    assert!(out.contains("2(a+b) = (8, 12)"));
}

#[test]
fn showcase308_option() {
    let out = run_ore("showcase308.ore");
    assert!(out.contains("Find user 1: Alice"));
    assert!(out.contains("Find user 99: unknown"));
    assert!(out.contains("is_some: true"));
}

#[test]
fn showcase309_strings() {
    let out = run_ore("showcase309.ore");
    assert!(out.contains("[palindrome]"));
    assert!(out.contains("Palindromes: radar, kayak, noon, civic"));
    assert!(out.contains("Title case: Hello World From Ore"));
}

#[test]
fn showcase310_enums() {
    let out = run_ore("showcase310.ore");
    assert!(out.contains("= 42"));
    assert!(out.contains("Monday"));
    assert!(out.contains("Saturday (weekend)"));
}

#[test]
fn showcase311_huffman() {
    let out = run_ore("showcase311.ore");
    assert!(out.contains("Text: abracadabra"));
    assert!(out.contains("'a': 5"));
}

#[test]
fn showcase312_physics() {
    let out = run_ore("showcase312.ore");
    assert!(out.contains("Projectile motion"));
    assert!(out.contains("Bouncing ball:"));
}

#[test]
fn showcase313_pipelines() {
    let out = run_ore("showcase313.ore");
    assert!(out.contains("Primes 1-50:"));
    assert!(out.contains("Twin primes:"));
}

#[test]
fn showcase314_memoization() {
    let out = run_ore("showcase314.ore");
    assert!(out.contains("F(10) = 55"));
    assert!(out.contains("12! = 479001600"));
}

#[test]
fn showcase315_perceptron() {
    let out = run_ore("showcase315.ore");
    assert!(out.contains("AND gate perceptron"));
    assert!(out.contains("OR gate perceptron"));
    assert!(out.contains("NOT 0 = 1"));
}

#[test]
fn showcase316_iterators() {
    let out = run_ore("showcase316.ore");
    assert!(out.contains("take_while(even): 2, 4, 6"));
    assert!(out.contains("Scan/prefix sum: 1, 3, 6, 10, 15"));
    assert!(out.contains("All even: true"));
}

#[test]
fn showcase317_graph_coloring() {
    let out = run_ore("showcase317.ore");
    assert!(out.contains("Valid coloring: true"));
}

#[test]
fn showcase318_merge_sort() {
    let out = run_ore("showcase318.ore");
    assert!(out.contains("Sorted:   3, 9, 10, 27, 38, 43, 82"));
    assert!(out.contains("apple, banana, cherry"));
}

#[test]
fn showcase319_command_pattern() {
    let out = run_ore("showcase319.ore");
    assert!(out.contains("Hello from command pattern!"));
    assert!(out.contains("10 + 20 = 30"));
    assert!(out.contains("echo!"));
}

#[test]
fn showcase320_testing() {
    let out = run_ore("showcase320.ore");
    assert!(out.contains("Tests defined above"));
}

#[test]
fn showcase321_short_circuit() {
    let out = run_ore("showcase321.ore");
    assert!(out.contains("false and side_effect:"));
    assert!(out.contains("  result: false"));
    assert!(!out.contains("should not run"));
    assert!(out.contains("  called: should run"));
    assert!(out.contains("  result: true"));
    assert!(out.contains("Safe access: first element is 1"));
}

#[test]
fn showcase322_luhn() {
    let out = run_ore("showcase322.ore");
    assert!(out.contains("Card: 4111111111111111"));
    assert!(out.contains("Valid (Luhn check passed)"));
    assert!(out.contains("Invalid (Luhn check failed)"));
    assert!(out.contains("1234567890123452 is valid"));
    assert!(out.contains("5 -> 1"));
}

#[test]
fn showcase323_coin_change() {
    let out = run_ore("showcase323.ore");
    assert!(out.contains("Coins: 1, 5, 10, 25"));
    assert!(out.contains("30 cents: 2 coins"));
    assert!(out.contains("25 cents: 13 ways"));
    assert!(out.contains("50 cents: 49 ways"));
    assert!(out.contains("6: min 2 coins, 4 ways"));
}

#[test]
fn showcase324_dna() {
    let out = run_ore("showcase324.ore");
    assert!(out.contains("DNA: ATGCGATCGATCGTAGCATCG"));
    assert!(out.contains("A: 5"));
    assert!(out.contains("GC content: 52%"));
    assert!(out.contains("Reverse complement: CGATGCTACGATCGATCGCAT"));
    assert!(out.contains("RNA: AUGCGAUCGAUCGUAGCAUCG"));
    assert!(out.contains("Motif 'ATC' found at positions: 5, 9, 17"));
}

#[test]
fn showcase325_bracket_matching() {
    let out = run_ore("showcase325.ore");
    assert!(out.contains("(()): Balanced"));
    assert!(out.contains("((): Unclosed: 1 remaining"));
    assert!(out.contains("(]): Mismatch at 1"));
    assert!(out.contains("((())): depth 3"));
    assert!(out.contains("(a+b)*(c+d): 2 pairs"));
}

#[test]
fn showcase326_number_classification() {
    let out = run_ore("showcase326.ore");
    assert!(out.contains("6: divisor sum = 6, perfect"));
    assert!(out.contains("12: divisor sum = 16, abundant"));
    assert!(out.contains("Perfect numbers up to 500: 6, 28, 496"));
    assert!(out.contains("(220, 284)"));
    assert!(out.contains("Abundant numbers up to 100: 22"));
}

#[test]
fn showcase327_water_pouring() {
    let out = run_ore("showcase327.ore");
    assert!(out.contains("Result: A has 4 gallons!"));
    assert!(out.contains("Target 5: 1 steps"));
    assert!(out.contains("Target 4: 6 steps"));
    assert!(out.contains("Solutions (jugs: 7 and 4):"));
}

#[test]
fn showcase328_kadane() {
    let out = run_ore("showcase328.ore");
    assert!(out.contains("Max subarray sum: 6"));
    assert!(out.contains("Subarray [3..6]: 4, -1, 2, 1"));
    assert!(out.contains("Max subarray sum: 15"));
    assert!(out.contains("Max subarray sum: -1"));
}

#[test]
fn showcase329_josephus() {
    let out = run_ore("showcase329.ore");
    assert!(out.contains("Josephus Problem"));
    assert!(out.contains("n=7: position 6 (person 7)"));
    assert!(out.contains("Order: 3 -> 6 -> 2 -> 7 -> 5 -> 1 -> 4"));
    assert!(out.contains("Survivor: person 4"));
    assert!(out.contains("Formula confirms: person 4"));
}

#[test]
fn showcase330_rational() {
    let out = run_ore("showcase330.ore");
    assert!(out.contains("1/2 + 1/3 = 5/6"));
    assert!(out.contains("1/2 * 1/3 = 1/6"));
    assert!(out.contains("6/8 = 3/4"));
    assert!(out.contains("-4/6 = -2/3"));
    assert!(out.contains("H(6) = 1 + 1/2 + ... + 1/6 = 49/20"));
    assert!(out.contains("1/2 + 1/3 + 1/6 = 1"));
}

#[test]
fn showcase331_topological_sort() {
    let out = run_ore("showcase331.ore");
    assert!(out.contains("Topological Sort (Course Prerequisites)"));
    assert!(out.contains("1. Math"));
    assert!(out.contains("6. Graphics"));
    assert!(out.contains("Build order:"));
    assert!(out.contains("1. libc"));
    assert!(out.contains("6. app"));
}

#[test]
fn showcase332_knapsack() {
    let out = run_ore("showcase332.ore");
    assert!(out.contains("0/1 Knapsack Problem"));
    assert!(out.contains("Maximum value: 200"));
    assert!(out.contains("Total weight: 10"));
    assert!(out.contains("Smaller knapsack (capacity=5)"));
    assert!(out.contains("Maximum value: 110"));
}

#[test]
fn showcase333_gray_code() {
    let out = run_ore("showcase333.ore");
    assert!(out.contains("3-bit Gray code:"));
    assert!(out.contains("0 -> 000"));
    assert!(out.contains("7 -> 111"));
    assert!(out.contains("All consecutive codes differ by exactly 1 bit"));
    assert!(out.contains("5-bit: 32 codes"));
}

#[test]
fn showcase334_isbn() {
    let out = run_ore("showcase334.ore");
    assert!(out.contains("0306406152: true"));
    assert!(out.contains("0306406153: false"));
    assert!(out.contains("9780306406157: true"));
    assert!(out.contains("9780306406158: false"));
    assert!(out.contains("check digit = 7"));
    assert!(out.contains("check digit = 0"));
}

#[test]
fn showcase335_date_validation() {
    let out = run_ore("showcase335.ore");
    assert!(out.contains("2024-Feb-29: true"));
    assert!(out.contains("2023-2-29: false"));
    assert!(out.contains("1900: false"));
    assert!(out.contains("2000: true"));
    assert!(out.contains("2024-12-31: day 366"));
    assert!(out.contains("Total: 366"));
}

#[test]
fn showcase336_pascal_triangle() {
    let out = run_ore("showcase336.ore");
    assert!(out.contains("1 4 6 4 1"));
    assert!(out.contains("Row 7: sum = 128"));
    assert!(out.contains("Diagonal 6: 13"));
    assert!(out.contains("Diagonal 7: 21"));
    assert!(out.contains("Row 10 sum: 1024"));
}

#[test]
fn showcase337_dutch_flag() {
    let out = run_ore("showcase337.ore");
    assert!(out.contains("Sorted: R R R R W W W B B B"));
    assert!(out.contains("Sorted: R R R W W W B B"));
    assert!(out.contains("Red: 4, White: 3, Blue: 3"));
    assert!(out.contains("array is correctly sorted"));
}

#[test]
fn showcase338_lis() {
    let out = run_ore("showcase338.ore");
    assert!(out.contains("LIS length: 4"));
    assert!(out.contains("LIS: 2, 5, 7, 101"));
    assert!(out.contains("LIS: 0, 1, 2, 3"));
    assert!(out.contains("LIS length: 5"));
    assert!(out.contains("LIS length: 1"));
    assert!(out.contains("LIS length: 6"));
}

#[test]
fn showcase339_number_to_words() {
    let out = run_ore("showcase339.ore");
    assert!(out.contains("42 -> forty-two"));
    assert!(out.contains("99 -> ninety-nine"));
    assert!(out.contains("one hundred and one"));
    assert!(out.contains("one thousand two hundred and thirty-four"));
    assert!(out.contains("one million"));
    assert!(out.contains("1234567 -> one million two hundred and thirty-four thousand five hundred and sixty-seven"));
}

#[test]
fn showcase340_string_search() {
    let out = run_ore("showcase340.ore");
    assert!(out.contains("'the' found 3 time(s) at positions: 0, 15, 28"));
    assert!(out.contains("'xyz' not found"));
    assert!(out.contains("ATC: 3"));
    assert!(out.contains("'hi world hello'"));
    assert!(out.contains("b[an][an]a"));
    assert!(out.contains("Count: 5"));
}

#[test]
fn showcase341_circular_buffer() {
    let out = run_ore("showcase341.ore");
    assert!(out.contains("Circular Buffer (Ring Buffer):"));
    assert!(out.contains("Push 10: 10 (size=1)"));
    assert!(out.contains("Push 50: 10, 20, 30, 40, 50 (size=5)"));
    assert!(out.contains("Push 80: 40, 50, 60, 70, 80 (size=5)"));
    assert!(out.contains("buf[0] = 40"));
    assert!(out.contains("Push 7: 5, 6, 7"));
}

#[test]
fn showcase342_number_patterns() {
    let out = run_ore("showcase342.ore");
    assert!(out.contains("0, 3, 6, 9, 12, 15, 18, 21, 24, 27"));
    assert!(out.contains("1, 3, 6, 10, 15, 21, 28, 36, 45, 55"));
    assert!(out.contains("1, 5, 12, 22, 35, 51, 70, 92, 117, 145"));
    assert!(out.contains("0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377"));
    assert!(out.contains("6: 6 -> 3 -> 10 -> 5 -> 16 -> 8 -> 4 -> 2 -> 1 (9 steps)"));
    assert!(out.contains("27:"));
    assert!(out.contains("(112 steps)"));
}

#[test]
fn showcase343_balanced_ternary() {
    let out = run_ore("showcase343.ore");
    assert!(out.contains("Balanced Ternary Representation:"));
    assert!(out.contains("0 -> 0 -> 0"));
    assert!(out.contains("5 -> 1TT -> 5"));
    assert!(out.contains("42 -> 1TTT0 -> 42"));
    assert!(out.contains("100 -> 11T01 -> 100"));
    assert!(out.contains("-1 -> T -> -1"));
    assert!(out.contains("-42 -> T1110 -> -42"));
    assert!(out.contains("All 50 values round-trip correctly!"));
}

#[test]
fn showcase344_zeckendorf() {
    let out = run_ore("showcase344.ore");
    assert!(out.contains("Zeckendorf's Representation:"));
    assert!(out.contains("4 = 3 + 1"));
    assert!(out.contains("10 = 8 + 2"));
    assert!(out.contains("20 = 13 + 5 + 2"));
    assert!(out.contains("100 = 89 + 8 + 3 (3 terms)"));
    assert!(out.contains("All numbers 1-100 verified!"));
}

#[test]
fn showcase345_kaprekar() {
    let out = run_ore("showcase345.ore");
    assert!(out.contains("Kaprekar Routine (6174):"));
    assert!(out.contains("3524 -> 5432 - 2345 = 3087"));
    assert!(out.contains("1234 -> 3087 -> 8352 -> 6174 (3 steps)"));
    assert!(out.contains("1111: all digits same (trivial zero)"));
    assert!(out.contains("6174 -> 7641 - 1467 = 6174"));
}

#[test]
fn showcase346_happy_numbers() {
    let out = run_ore("showcase346.ore");
    assert!(out.contains("First 20 happy numbers:"));
    assert!(out.contains("1, 7, 10, 13, 19, 23, 28, 31, 32, 44"));
    assert!(out.contains("7: 7 -> 49 -> 97 -> 130 -> 10 -> 1 (happy)"));
    assert!(out.contains("2: 2 -> 4"));
    assert!(out.contains("1-100: 20 happy numbers"));
}

#[test]
fn showcase347_abundant_deficient() {
    let out = run_ore("showcase347.ore");
    assert!(out.contains("6: divisor sum = 6, perfect"));
    assert!(out.contains("12: divisor sum = 16, abundant"));
    assert!(out.contains("Perfect numbers up to 10000:"));
    assert!(out.contains("6, 28, 496, 8128"));
    assert!(out.contains("Deficient: 751"));
    assert!(out.contains("Abundant: 246"));
    assert!(out.contains("Perfect: 3"));
}

#[test]
fn showcase348_base64() {
    let out = run_ore("showcase348.ore");
    assert!(out.contains("Base64 Encoding and Decoding:"));
    assert!(out.contains("'Hello' -> 'SGVsbG8='"));
    assert!(out.contains("'Hello, World!' -> 'SGVsbG8sIFdvcmxkIQ=='"));
    assert!(out.contains("'Man' -> 'TWFu' (expected: TWFu)"));
    assert!(out.contains("'Ma' -> 'TWE=' (expected: TWE=)"));
    assert!(out.contains("'M' -> 'TQ==' (expected: TQ==)"));
    assert!(out.contains("[OK]"));
}

#[test]
fn showcase349_sieve_sundaram() {
    let out = run_ore("showcase349.ore");
    assert!(out.contains("Sieve of Sundaram:"));
    assert!(out.contains("Primes up to 100 (25 primes):"));
    assert!(out.contains("2, 3, 5, 7, 11, 13, 17, 19, 23, 29"));
    assert!(out.contains("Counts match!"));
    assert!(out.contains("Up to 1000: 168 primes"));
    assert!(out.contains("(3, 5), (5, 7), (11, 13)"));
}

#[test]
fn showcase350_number_spiral() {
    let out = run_ore("showcase350.ore");
    assert!(out.contains("Number Spiral (Ulam Spiral):"));
    assert!(out.contains("Layer 0: 0 primes out of 1 numbers"));
    assert!(out.contains("Layer 1: 4 primes out of 8 numbers"));
    assert!(out.contains("65, 37, 17, 5, 1, 9, 25, 49, 81"));
    assert!(out.contains("Primes on both diagonals: 9 out of 17"));
}

#[test]
fn showcase351_hamming_numbers() {
    let out = run_ore("showcase351.ore");
    assert!(out.contains("Hamming Numbers (Regular Numbers):"));
    assert!(out.contains("1, 2, 3, 4, 5, 6, 8, 9, 10, 12, 15, 16, 18, 20, 24, 25, 27, 30, 32, 36"));
    assert!(out.contains("Up to 100: 34 Hamming numbers"));
    assert!(out.contains("7, 11, 13, 14, 17, 19, 21, 22, 23, 26"));
}

#[test]
fn showcase352_continued_fractions() {
    let out = run_ore("showcase352.ore");
    assert!(out.contains("Continued Fractions:"));
    assert!(out.contains("Coefficients: 3, 7, 16"));
    assert!(out.contains("Back to fraction: 355/113"));
    assert!(out.contains("C1 = 22/7"));
    assert!(out.contains("C2 = 355/113"));
    assert!(out.contains("C5 = 99/70"));
}

#[test]
fn showcase353_game_of_life() {
    let out = run_ore("showcase353.ore");
    assert!(out.contains("Conway's Game of Life:"));
    assert!(out.contains("Blinker oscillator:"));
    assert!(out.contains("Gen 0 (alive: 3):"));
    assert!(out.contains("Glider:"));
    assert!(out.contains("Gen 0 (alive: 5):"));
    assert!(out.contains("Block (still life):"));
    assert!(out.contains("Gen 0 (alive: 4):"));
}

#[test]
fn showcase354_expression_parser() {
    let out = run_ore("showcase354.ore");
    assert!(out.contains("Simple Expression Parser:"));
    assert!(out.contains("2 + 3 * 4 = 14"));
    assert!(out.contains("10 - 2 * 3 = 4"));
    assert!(out.contains("1 + 2 * 3 + 4 = 11"));
    assert!(out.contains("NUMBER(12)"));
    assert!(out.contains("STAR"));
}

#[test]
fn showcase355_huffman_encoding() {
    let out = run_ore("showcase355.ore");
    assert!(out.contains("Huffman-Style Encoding:"));
    assert!(out.contains("Text: abracadabra"));
    assert!(out.contains("'a': 5"));
    assert!(out.contains("Unique characters: 5"));
    assert!(out.contains("Encoded length: 18 bits"));
    assert!(out.contains("Total: 18 bits"));
}

#[test]
fn showcase356_bfs_pathfinding() {
    let out = run_ore("showcase356.ore");
    assert!(out.contains("Tower Defense Path Finding (BFS):"));
    assert!(out.contains("Shortest path length: 16"));
    assert!(out.contains("Shortest path length: 13"));
    assert!(out.contains("Path has 14 cells"));
}

#[test]
fn showcase357_rps_tournament() {
    let out = run_ore("showcase357.ore");
    assert!(out.contains("Rock-Paper-Scissors Tournament:"));
    assert!(out.contains("Tournament standings:"));
    assert!(out.contains("Tournament winner: Cissy with 10 points!"));
    assert!(out.contains("Rocky vs Paige: 0-5-0 -> Paige wins"));
}

#[test]
fn showcase358_polynomials() {
    let out = run_ore("showcase358.ore");
    assert!(out.contains("Polynomial Evaluation and Operations:"));
    assert!(out.contains("p(x) = 3 + 2x + x^2"));
    assert!(out.contains("p(x) + q(x) = 4 + x + 3x^2"));
    assert!(out.contains("product=77, direct=77"));
    assert!(out.contains("(x+1)^5 = 1 + 5x + 10x^2 + 10x^3 + 5x^4 + x^5"));
    assert!(out.contains("at x=1: 32, at x=2: 243"));
}

#[test]
fn showcase359_hash_functions() {
    let out = run_ore("showcase359.ore");
    assert!(out.contains("Cryptographic Hash Functions:"));
    assert!(out.contains("djb2(\"hello\") = 25db7c56"));
    assert!(out.contains("sdbm(\"hello\") = 12ea453d"));
    assert!(out.contains("fnv1(\"hello\") = 1c64ebbb"));
    assert!(out.contains("Hash distribution"));
}

#[test]
fn showcase360_matrix_chain_multiplication() {
    let out = run_ore("showcase360.ore");
    assert!(out.contains("Matrix Chain Multiplication:"));
    assert!(out.contains("Optimal cost: 5000 scalar multiplications"));
    assert!(out.contains("((M1 x M2) x (M3 x M4))"));
    assert!(out.contains("Optimal cost: 15125 scalar multiplications"));
    assert!(out.contains("Savings: 25375 multiplications"));
    assert!(out.contains("Optimal cost: 26000"));
}

#[test]
fn showcase361_cellular_automaton() {
    let out = run_ore("showcase361.ore");
    assert!(out.contains("Cellular Automaton:"));
    assert!(out.contains("Rule 30 (15 cells, 8 generations):"));
    assert!(out.contains(".......#.......  (1 alive)"));
    assert!(out.contains("......###......  (3 alive)"));
    assert!(out.contains("Rule 110 (15 cells, 8 generations):"));
    assert!(out.contains("Rule 110 is proven to be Turing-complete!"));
    assert!(out.contains("Rule 90 - Sierpinski triangle"));
    assert!(out.contains(".........#.#.........  (2 alive)"));
}

#[test]
fn showcase362_chinese_remainder_theorem() {
    let out = run_ore("showcase362.ore");
    assert!(out.contains("Chinese Remainder Theorem:"));
    assert!(out.contains("Solution: x = 23 (mod 105)"));
    assert!(out.contains("Solution: x = 23 (mod 30)"));
    assert!(out.contains("Solution: x = 6 (mod 210)"));
    assert!(out.contains("Verified: true"));
    assert!(out.contains("gcd(35, 15) = 5"));
    assert!(out.contains("Answer: 1 (and every 60 after that)"));
}

#[test]
fn showcase363_egyptian_fractions() {
    let out = run_ore("showcase363.ore");
    assert!(out.contains("Egyptian Fractions (Greedy Algorithm):"));
    assert!(out.contains("2/3 = 1/2 + 1/6"));
    assert!(out.contains("3/4 = 1/2 + 1/4"));
    assert!(out.contains("5/6 = 1/2 + 1/3"));
    assert!(out.contains("3/7 = 1/3 + 1/11 + 1/231"));
    assert!(out.contains("1/2 + 1/6 = 2/3 (correct!)"));
}

#[test]
fn showcase364_stern_brocot_tree() {
    let out = run_ore("showcase364.ore");
    assert!(out.contains("Stern-Brocot Tree:"));
    assert!(out.contains("Level 0: 1/1"));
    assert!(out.contains("Level 1: 1/2  1/1  2/1"));
    assert!(out.contains("3/5: LRL"));
    assert!(out.contains("7/3: RRLL"));
    assert!(out.contains("1/3: LL"));
    assert!(out.contains("Path to 355/113:"));
    assert!(out.contains("Adjacent fraction property"));
}

#[test]
fn showcase365_perfect_shuffle() {
    let out = run_ore("showcase365.ore");
    assert!(out.contains("Perfect Shuffle (Riffle Shuffle):"));
    assert!(out.contains("Shuffle 1: [1, 5, 2, 6, 3, 7, 4, 8]"));
    assert!(out.contains("8 cards: 3 shuffles"));
    assert!(out.contains("52 cards: 8 shuffles"));
    assert!(out.contains("Out-shuffle restores in 8 perfect shuffles"));
    assert!(out.contains("In-shuffle restores in 52 perfect shuffles"));
}

#[test]
fn showcase366_langtons_ant() {
    let out = run_ore("showcase366.ore");
    assert!(out.contains("Langton's Ant:"));
    assert!(out.contains("Grid after 80 steps (15x15):"));
    assert!(out.contains("Ant position: (7, 3)"));
    assert!(out.contains("Ant direction: up"));
    assert!(out.contains("Black cells: 16"));
    assert!(out.contains("Step 1: ant at (3,2) facing right"));
}

#[test]
fn showcase367_counting_radix_sort() {
    let out = run_ore("showcase367.ore");
    assert!(out.contains("Counting Sort and Radix Sort:"));
    assert!(out.contains("Sorted: [1, 2, 2, 3, 3, 4, 4, 5, 7, 8]"));
    assert!(out.contains("Sorted: [2, 24, 45, 66, 75, 90, 170, 802]"));
    assert!(out.contains("Sorted: [7, 42, 100, 1234, 3210, 5678, 8765, 9999]"));
    assert!(out.contains("Both are non-comparison sorts!"));
}

#[test]
fn showcase368_catalan_numbers() {
    let out = run_ore("showcase368.ore");
    assert!(out.contains("Catalan Numbers:"));
    assert!(out.contains("C(0) = 1"));
    assert!(out.contains("C(5) = 42"));
    assert!(out.contains("C(10) = 16796"));
    assert!(out.contains("C(14) = 2674440"));
    assert!(out.contains("((()))"));
    assert!(out.contains("()()()"));
    assert!(out.contains("5-gon: 5 triangulations"));
}

#[test]
fn showcase369_markov_chain() {
    let out = run_ore("showcase369.ore");
    assert!(out.contains("Markov Chain Simulation:"));
    assert!(out.contains("Weather Markov chain (3 states):"));
    assert!(out.contains("Sunny  -> Sunny:60% Cloudy:30% Rainy:10%"));
    assert!(out.contains("Simulation (20 steps, starting Sunny):"));
    assert!(out.contains("Steady state distribution (power iteration):"));
    assert!(out.contains("Simple 4-page PageRank analogy:"));
}

#[test]
fn showcase370_persistent_stack() {
    let out = run_ore("showcase370.ore");
    assert!(out.contains("Persistent Stack (Functional Data Structure):"));
    assert!(out.contains("s0 (empty): [], size=0"));
    assert!(out.contains("s3 (push 30): [30, 20, 10], size=3"));
    assert!(out.contains("s3 (original): [30, 20, 10], top=30"));
    assert!(out.contains("s3_popped:     [20, 10], top=20"));
    assert!(out.contains("branch_a (push 40): [40, 20, 10]"));
    assert!(out.contains("s2 unchanged:       [20, 10]"));
    assert!(out.contains("Reversed:  [1, 2, 3]"));
}

#[test]
fn showcase371_permutation_cycles() {
    let out = run_ore("showcase371.ore");
    assert!(out.contains("Permutation Cycles and Parity:"));
    assert!(out.contains("Cycle lengths: [3, 1, 1]"));
    assert!(out.contains("Parity: even"));
    assert!(out.contains("Order: 3"));
    assert!(out.contains("Parity: odd"));
    assert!(out.contains("Order: 2"));
    assert!(out.contains("a * b = [0, 1, 2]"));
    assert!(out.contains("p^5 = [0, 1, 2, 3, 4]"));
    assert!(out.contains("every permutation decomposes into disjoint cycles"));
}

#[test]
fn showcase372_nim_game() {
    let out = run_ore("showcase372.ore");
    assert!(out.contains("Nim Game - Strategy and Winning Moves:"));
    assert!(out.contains("3 XOR 5 = 6"));
    assert!(out.contains("1 XOR 2 XOR 3 = 0"));
    assert!(out.contains("Heaps [1, 2, 3]: nim-sum=0, LOSING (P2 wins)"));
    assert!(out.contains("Heaps [3, 4, 5]: nim-sum=2, WINNING (P1 wins)"));
    assert!(out.contains("Player 1 wins!"));
    assert!(out.contains("Player 2 wins!"));
    assert!(out.contains("Sprague-Grundy theorem"));
}

#[test]
fn showcase373_turtle_graphics() {
    let out = run_ore("showcase373.ore");
    assert!(out.contains("Turtle Graphics (Text Output):"));
    assert!(out.contains("Drawing a 6x6 square:"));
    assert!(out.contains("******.."));
    assert!(out.contains("Drawing an L-shape:"));
    assert!(out.contains("Drawing stairs:"));
    assert!(out.contains("Drawing a cross:"));
    assert!(out.contains("***********"));
}

#[test]
fn showcase374_finite_field() {
    let out = run_ore("showcase374.ore");
    assert!(out.contains("Finite Field Arithmetic (GF(p)):"));
    assert!(out.contains("Working in GF(7):"));
    assert!(out.contains("2^(-1) = 4  (verify: 2 * 4 = 1)"));
    assert!(out.contains("3 is a generator"));
    assert!(out.contains("5 is a generator"));
    assert!(out.contains("3^0 = 1"));
    assert!(out.contains("log_3(2) = 2"));
    assert!(out.contains("foundation of modern cryptography"));
}

#[test]
fn showcase375_rle_bitmap() {
    let out = run_ore("showcase375.ore");
    assert!(out.contains("Run-Length Encoding on Bitmap Images:"));
    assert!(out.contains("Row 0: W2 B4 W4"));
    assert!(out.contains("Row 3: B1 W6 B1 W2"));
    assert!(out.contains("Original pixels: 80"));
    assert!(out.contains("Stripe encoded: B10 (2 values)"));
    assert!(out.contains("RLE works best on data with long runs"));
}

#[test]
fn showcase376_maze_solving() {
    let out = run_ore("showcase376.ore");
    assert!(out.contains("Maze Solving (DFS with Backtracking):"));
    assert!(out.contains("Maze 1 (11x7):"));
    assert!(out.contains("Solution found! Path length: 25"));
    assert!(out.contains("Maze 2 (9x5):"));
    assert!(out.contains("Solution found! Path length: 11"));
    assert!(out.contains("DFS explores as deep as possible"));
}

#[test]
fn showcase377_interval_scheduling() {
    let out = run_ore("showcase377.ore");
    assert!(out.contains("Interval Scheduling (Greedy Algorithm):"));
    assert!(out.contains("Optimal selection (3 meetings):"));
    assert!(out.contains("Maximum concurrent meetings: 3"));
    assert!(out.contains("Optimal selection (3 jobs):"));
    assert!(out.contains("Total work time: 9"));
    assert!(out.contains("Greedy by earliest end time"));
}

#[test]
fn showcase378_bankers_algorithm() {
    let out = run_ore("showcase378.ore");
    assert!(out.contains("Banker's Algorithm (Deadlock Avoidance):"));
    assert!(out.contains("System: 5 processes, 3 resource types"));
    assert!(out.contains("System is in a SAFE state!"));
    assert!(out.contains("Safe sequence: P1 -> P3 -> P0 -> P2 -> P4"));
    assert!(out.contains("P1 requests [1, 0, 2]: GRANTED (safe)"));
    assert!(out.contains("P4 requests [3, 3, 0]: DENIED (unsafe)"));
    assert!(out.contains("P0 requests [0, 2, 0]: GRANTED (safe)"));
}

#[test]
fn showcase379_regex_engine() {
    let out = run_ore("showcase379.ore");
    assert!(out.contains("Simple Regex Engine"));
    assert!(out.contains("'hello' =~ 'hello': true"));
    assert!(out.contains("'hello' =~ 'world': false"));
    assert!(out.contains("'h.llo' =~ 'hello': true"));
    assert!(out.contains("'ab*c' =~ 'abbbc': true"));
    assert!(out.contains("'lo' in 'hello world': position 3"));
    assert!(out.contains("'a' in 'banana': positions [1, 3, 5]"));
    assert!(out.contains("Words matching 'c.t':"));
    assert!(out.contains("cat"));
}

#[test]
fn showcase380_number_theory() {
    let out = run_ore("showcase380.ore");
    assert!(out.contains("Number Theory Toolkit:"));
    assert!(out.contains("12 = [2, 2, 3]"));
    assert!(out.contains("360 = [2, 2, 2, 3, 3, 5]"));
    assert!(out.contains("phi(7) = 6 (prime)"));
    assert!(out.contains("n=10: sum=10 [OK]"));
    assert!(out.contains("6 is perfect (divisor sum = 12)"));
    assert!(out.contains("28 is perfect (divisor sum = 56)"));
    assert!(out.contains("496 is perfect (divisor sum = 992)"));
    assert!(out.contains("gcd(12, 18) = 6, lcm(12, 18) = 36"));
}

#[test]
fn showcase381_reservoir_sampling() {
    let out = run_ore("showcase381.ore");
    assert!(out.contains("Reservoir Sampling Algorithm:"));
    assert!(out.contains("Stream: 0 to 49 (50 elements)"));
    assert!(out.contains("Seed 42:"));
    assert!(out.contains("k=1: got 1 items [OK]"));
    assert!(out.contains("k=5: got 5 items [OK]"));
    assert!(out.contains("k=10: got 10 items [OK]"));
    assert!(out.contains("O(n) time, O(k) space"));
}

#[test]
fn showcase382_edit_distance() {
    let out = run_ore("showcase382.ore");
    assert!(out.contains("Edit Distance with Alignment:"));
    assert!(out.contains("\"kitten\" -> \"sitting\" (distance: 3)"));
    assert!(out.contains("\"saturday\" -> \"sunday\" (distance: 3)"));
    assert!(out.contains("replace 'k' with 's'"));
    assert!(out.contains("\"\" -> \"abc\" (distance: 3)"));
    assert!(out.contains("Distance matrix:"));
}

#[test]
fn showcase383_stack_interpreter() {
    let out = run_ore("showcase383.ore");
    assert!(out.contains("Simple Stack-Based Interpreter:"));
    assert!(out.contains("output: 14"));
    assert!(out.contains("output: 49"));
    assert!(out.contains("output: 25"));
    assert!(out.contains("0 is even"));
    assert!(out.contains("1 is odd"));
    assert!(out.contains("output: 120"));
}

#[test]
fn showcase384_knights_tour() {
    let out = run_ore("showcase384.ore");
    assert!(out.contains("Knight's Tour (Warnsdorff's Rule):"));
    assert!(out.contains("5x5 board starting at (0, 0):"));
    assert!(out.contains("Tour status: Complete"));
    assert!(out.contains("8x8 board starting at (0, 0):"));
    assert!(out.contains("(0,0): OK"));
    assert!(out.contains("Warnsdorff's heuristic"));
}

#[test]
fn showcase385_huffman_coding() {
    let out = run_ore("showcase385.ore");
    assert!(out.contains("Huffman Coding Tree Traversal:"));
    assert!(out.contains("'a': 0 (freq: 5)"));
    assert!(out.contains("'s': 0 (freq: 4)"));
    assert!(out.contains("'i': 11 (freq: 4)"));
    assert!(out.contains("Compression ratio:"));
}

#[test]
fn showcase386_bloom_filter() {
    let out = run_ore("showcase386.ore");
    assert!(out.contains("Bloom Filter Simulation:"));
    assert!(out.contains("Filter size: 64 bits, 3 hash functions"));
    assert!(out.contains("10: probably yes"));
    assert!(out.contains("100: probably yes"));
    assert!(out.contains("false negatives: 0"));
    assert!(out.contains("space-efficient probabilistic"));
}

#[test]
fn showcase387_lcg_prng() {
    let out = run_ore("showcase387.ore");
    assert!(out.contains("Linear Congruential Generator (PRNG):"));
    assert!(out.contains("Parameters: a=1103515245"));
    assert!(out.contains("Period: 25 (m = 101)"));
    assert!(out.contains("Distribution test"));
    assert!(out.contains("Dice roll simulation"));
    assert!(out.contains("Seed sensitivity"));
}

#[test]
fn showcase388_stable_marriage() {
    let out = run_ore("showcase388.ore");
    assert!(out.contains("Stable Marriage Problem (Gale-Shapley):"));
    assert!(out.contains("Man 0 proposes to Woman 0 -> accepted (both free)"));
    assert!(out.contains("Man 0 <-> Woman 0"));
    assert!(out.contains("Stable: YES"));
    assert!(out.contains("Man 1 proposes to Woman 0 -> accepted (dumped Man 0)"));
    assert!(out.contains("guarantees a stable matching"));
}

#[test]
fn showcase389_compression() {
    let out = run_ore("showcase389.ore");
    assert!(out.contains("Dictionary-Based Compression:"));
    assert!(out.contains("\"aabbbcccc\" -> [0, 2, 1, 3, 2, 4] (3 pairs)"));
    assert!(out.contains("Verify: OK"));
    assert!(out.contains("\"aaaaaaa\" -> [0, 7] (1 pairs)"));
    assert!(out.contains("Original: 45 chars -> 3 pairs"));
    assert!(out.contains("'at' appears 3 times"));
}

#[test]
fn showcase390_union_find() {
    let out = run_ore("showcase390.ore");
    assert!(out.contains("Disjoint Set / Union-Find:"));
    assert!(out.contains("Components: 10"));
    assert!(out.contains("Union(0, 1): merged"));
    assert!(out.contains("Root 0: [0, 1, 2, 3, 8, 9] (size: 6)"));
    assert!(out.contains("Connected(1, 3): YES"));
    assert!(out.contains("Connected(0, 5): NO"));
    assert!(out.contains("MST total weight: 11"));
    assert!(out.contains("After find(0): root = 7"));
    assert!(out.contains("(All nodes now point directly to root)"));
}

#[test]
fn showcase391_astar_pathfinding() {
    let out = run_ore("showcase391.ore");
    assert!(out.contains("A* Pathfinding on 8x8 Grid:"));
    assert!(out.contains("Path length: 15 steps"));
    assert!(out.contains("(0,0)"));
    assert!(out.contains("(7,7)"));
    assert!(out.contains("Path length: 9 steps"));
    assert!(out.contains("A* finds shortest path using heuristic-guided search."));
}

#[test]
fn showcase392_polynomial_division() {
    let out = run_ore("showcase392.ore");
    assert!(out.contains("Polynomial Long Division:"));
    assert!(out.contains("Quotient:  x^2 + x + 3"));
    assert!(out.contains("Remainder: 5"));
    assert!(out.contains("Quotient:  2x^2 - x + 1"));
    assert!(out.contains("Remainder: 3"));
    assert!(out.contains("Quotient:  x^3 + x^2 + x + 1"));
    assert!(out.contains("Remainder: 0"));
    assert!(out.contains("Verify Q*D+R: x^4 - 1"));
    assert!(out.contains("Quotient:  3x^2 + x - 4"));
    assert!(out.contains("Remainder: 6"));
}

#[test]
fn showcase393_type_checker() {
    let out = run_ore("showcase393.ore");
    assert!(out.contains("Type Checker for Tiny Expression Language:"));
    assert!(out.contains("(42 + 10)"));
    assert!(out.contains("Type: Int [OK]"));
    assert!(out.contains("Type: Bool [OK]"));
    assert!(out.contains("Error: add expects Int+Int, got Int+Bool [FAIL]"));
    assert!(out.contains("Error: if condition must be Bool, got Int [FAIL]"));
    assert!(out.contains("Error: if branches must match, got Int and Bool [FAIL]"));
    assert!(out.contains("Error: eq expects same types, got Int and Bool [FAIL]"));
    assert!(out.contains("Type checking ensures expressions are well-formed before evaluation."));
}

#[test]
fn showcase394_sudoku_solver() {
    let out = run_ore("showcase394.ore");
    assert!(out.contains("Sudoku Solver (4x4 Mini-Puzzles):"));
    assert!(out.contains("Solved:"));
    assert!(out.contains("1 2 | 3 4"));
    assert!(out.contains("4 3 | 2 1"));
    assert!(out.contains("Number of valid completions: 72"));
    assert!(out.contains("Sudoku solving uses backtracking with constraint checking."));
}

#[test]
fn showcase395_red_black_tree() {
    let out = run_ore("showcase395.ore");
    assert!(out.contains("Red-Black Tree Properties:"));
    assert!(out.contains("Property 1 - Root is black: YES"));
    assert!(out.contains("Property 2 - No red-red parent-child: YES"));
    assert!(out.contains("Property 3 - Black height: 3"));
    assert!(out.contains("In-order traversal: [1, 5, 8, 10, 15, 20, 25, 30]"));
    assert!(out.contains("Is sorted: YES"));
    assert!(out.contains("Total nodes: 8"));
    assert!(out.contains("Red-black trees guarantee O(log n) operations via color invariants."));
}

#[test]
fn showcase396_checksums() {
    let out = run_ore("showcase396.ore");
    assert!(out.contains("Checksum Calculations:"));
    assert!(out.contains("fletcher16(\"hello\") = 11542"));
    assert!(out.contains("consistent: YES"));
    assert!(out.contains("Different inputs: different checksums"));
    assert!(out.contains("Additive detects swap: NO (expected)"));
    assert!(out.contains("Weighted detects swap: YES"));
    assert!(out.contains("Error detected: YES"));
    assert!(out.contains("Luhn([0]): valid"));
    assert!(out.contains("Luhn([1, 8]): valid"));
    assert!(out.contains("Luhn([1, 9]): invalid"));
    assert!(out.contains("Checksums: essential for data integrity in networking and storage."));
}

#[test]
fn showcase397_query_engine() {
    let out = run_ore("showcase397.ore");
    assert!(out.contains("Simple Database Query Engine:"));
    assert!(out.contains("(7 rows)"));
    assert!(out.contains("Alice | Engineering | 95000 | 30"));
    assert!(out.contains("WHERE dept = Engineering:"));
    assert!(out.contains("(3 rows)"));
    assert!(out.contains("WHERE salary > 80000:"));
    assert!(out.contains("(4 rows)"));
    assert!(out.contains("ORDER BY salary ASC LIMIT 3:"));
    assert!(out.contains("Diana | Marketing | 68000 | 28"));
    assert!(out.contains("Engineering: count=3, avg_salary=103333"));
    assert!(out.contains("Engineering AND salary > 100000:"));
    assert!(out.contains("(2 rows)"));
    assert!(out.contains("Query engine demonstrates filter, sort, group, and limit operations."));
}

#[test]
fn showcase398_minimax() {
    let out = run_ore("showcase398.ore");
    assert!(out.contains("Minimax Tic-Tac-Toe AI:"));
    assert!(out.contains("Game 1: AI (X) vs AI (O)"));
    assert!(out.contains("Result: Draw!"));
    assert!(out.contains("Game 2: Position evaluation"));
    assert!(out.contains("Best move for X:"));
    assert!(out.contains("Best move for O:"));
    assert!(out.contains("X's best move: (0,2) (should complete row)"));
    assert!(out.contains("O's best move: (0,2) (should block X)"));
    assert!(out.contains("Minimax guarantees optimal play - perfect play leads to a draw."));
}

#[test]
fn showcase399_fast_exponentiation() {
    let out = run_ore("showcase399.ore");
    assert!(out.contains("Fast Exponentiation (Binary Method):"));
    assert!(out.contains("2^10 = 1024"));
    assert!(out.contains("2^20 = 1048576"));
    assert!(out.contains("[OK]"));
    assert!(out.contains("Final: 2^13 = 8192"));
    assert!(out.contains("2^10 mod 1000 = 24"));
    assert!(out.contains("3^6 mod 7 = 1"));
    assert!(out.contains("3^18 mod 19 = 1"));
    assert!(out.contains("Binary exponentiation: O(log n) vs naive O(n) multiplications."));
}

#[test]
fn showcase400_grand_showcase() {
    let out = run_ore("showcase400.ore");
    assert!(out.contains("Grand Showcase - Ore Language Features:"));
    assert!(out.contains("Manhattan distance: 7"));
    assert!(out.contains("Circle: area = 75"));
    assert!(out.contains("Rectangle: area = 24"));
    assert!(out.contains("map(x => x * 2): [2, 4, 6, 8, 10]"));
    assert!(out.contains("reduce(+): 15"));
    assert!(out.contains("0, 1, 1, 2, 3, 5, 8, 13, 21, 34"));
    assert!(out.contains("GCD(48, 18) = 6"));
    assert!(out.contains("Encrypted: khoor zruog"));
    assert!(out.contains("Round-trip: OK"));
    assert!(out.contains("Sorted: [3, 9, 10, 27, 38, 43, 82]"));
    assert!(out.contains("\"the\": 3"));
    assert!(out.contains("sum(1..10) = 55"));
    assert!(out.contains("FizzBuzz: 1 2 Fizz 4 Buzz"));
    assert!(out.contains("Squares: [1, 4, 9, 16, 25, 36, 49]"));
    assert!(out.contains("10! = 3628800"));
    assert!(out.contains("Sum 1..100 (while): 5050"));
    assert!(out.contains("Collatz(27): 111 steps to reach 1"));
    assert!(out.contains("This is showcase 400 - celebrating the milestone!"));
}

#[test]
fn showcase401_nqueens() {
    let out = run_ore("showcase401.ore");
    assert!(out.contains("N-Queens Problem (Backtracking):"));
    assert!(out.contains("Queen positions (row->col): [1, 3, 0, 2]"));
    assert!(out.contains("Queen positions: [0, 4, 7, 5, 2, 6, 1, 3]"));
    assert!(out.contains("4-Queens: 2 solutions"));
    assert!(out.contains("8-Queens: 92 solutions"));
    assert!(out.contains("N-Queens solved via backtracking with pruning."));
}

#[test]
fn showcase402_convex_hull() {
    let out = run_ore("showcase402.ore");
    assert!(out.contains("Convex Hull (Gift Wrapping):"));
    assert!(out.contains("Convex hull (4 vertices):"));
    assert!(out.contains("P0(0,0) -> P1(4,0) -> P2(4,4) -> P3(0,4)"));
    assert!(out.contains("3 vertices (triangle itself)"));
    assert!(out.contains("Convex hull (9 vertices):"));
    assert!(out.contains("Manhattan perimeter: 32"));
    assert!(out.contains("Gift wrapping computes convex hull in O(nh) time."));
}

#[test]
fn showcase403_bitwise_ops() {
    let out = run_ore("showcase403.ore");
    assert!(out.contains("Bitwise Operations (Manual Bit Manipulation):"));
    assert!(out.contains("42 = 00101010"));
    assert!(out.contains("AND: 00000000 = 0"));
    assert!(out.contains("OR:  11111111 = 255"));
    assert!(out.contains("XOR: 11111111 = 255"));
    assert!(out.contains("NOT 170: 01010101 = 85"));
    assert!(out.contains("Set bit 0: 00101011 = 43"));
    assert!(out.contains("128: 1 ones, power of 2: yes"));
    assert!(out.contains("Encrypted: [98, 79, 70, 70, 69]"));
    assert!(out.contains("Round-trip: OK"));
}

#[test]
fn showcase404_sparse_matrix() {
    let out = run_ore("showcase404.ore");
    assert!(out.contains("Sparse Matrix Operations:"));
    assert!(out.contains("Matrix A (4x4, 6 non-zeros):"));
    assert!(out.contains("Density: 37%"));
    assert!(out.contains("A + B (4x4, 10 non-zeros):"));
    assert!(out.contains("A * B (4x4, 6 non-zeros):"));
    assert!(out.contains("A transposed (4x4, 6 non-zeros):"));
    assert!(out.contains("A * I (should equal A)"));
    assert!(out.contains("Sparse matrices save memory for mostly-zero data."));
}

#[test]
fn showcase405_assembler_vm() {
    let out = run_ore("showcase405.ore");
    assert!(out.contains("Simple Assembler/VM:"));
    assert!(out.contains("LOAD R0, 5"));
    assert!(out.contains("Output: [8]"));
    assert!(out.contains("Output: [5, 4, 3, 2, 1]"));
    assert!(out.contains("Output: [42]"));
    assert!(out.contains("Output: [15]"));
    assert!(out.contains("Output: [89]"));
    assert!(out.contains("Simple VM executes bytecode with registers and branching."));
}

#[test]
fn showcase406_fractions() {
    let out = run_ore("showcase406.ore");
    assert!(out.contains("Fraction Arithmetic:"));
    assert!(out.contains("2/6 simplified = 1/3"));
    assert!(out.contains("1/2 + 1/3 = 5/6"));
    assert!(out.contains("3/4 - 1/2 = 1/4"));
    assert!(out.contains("1/2 * 3/4 = 3/8"));
    assert!(out.contains("3/4 / 1/3 = 9/4"));
    assert!(out.contains("H(10) = 7381/2520"));
    assert!(out.contains("5/7 = 1/2 + 1/5 + 1/70"));
    assert!(out.contains("C6 = 239/169"));
    assert!(out.contains("Exact fraction arithmetic avoids floating-point errors."));
}

#[test]
fn showcase407_tsp() {
    let out = run_ore("showcase407.ore");
    assert!(out.contains("Traveling Salesman Problem:"));
    assert!(out.contains("Tour: 0 -> 1 -> 3 -> 4 -> 2 -> 0 (cost: 85)"));
    assert!(out.contains("Optimal cost: 85"));
    assert!(out.contains("Best nearest-neighbor: 85"));
    assert!(out.contains("Optimal tour: 0 -> 1 -> 2 -> 3 -> 0 (cost: 78)"));
    assert!(out.contains("TSP: brute force guarantees optimality, NN is a fast heuristic."));
}

#[test]
fn showcase408_btree() {
    let out = run_ore("showcase408.ore");
    assert!(out.contains("B-Tree Concepts (2-3 Tree):"));
    assert!(out.contains("In-order traversal: [3, 5, 8, 10, 15, 20, 25, 30]"));
    assert!(out.contains("Search(5): FOUND"));
    assert!(out.contains("Search(10): FOUND"));
    assert!(out.contains("Search(7): NOT FOUND"));
    assert!(out.contains("Search(30): FOUND"));
    assert!(out.contains("Total nodes: 7"));
    assert!(out.contains("B-trees keep data sorted and balanced for efficient search."));
}

#[test]
fn showcase409_image_processing() {
    let out = run_ore("showcase409.ore");
    assert!(out.contains("Image Processing (Kernel Convolution):"));
    assert!(out.contains("Original image (7x7):"));
    assert!(out.contains("Box blur (3x3 average) (5x5):"));
    assert!(out.contains("Sharpen kernel (5x5):"));
    assert!(out.contains("Combined edge magnitude (5x5):"));
    assert!(out.contains("Edge threshold (>100) (5x5):"));
    assert!(out.contains("Min: 0, Max: 200, Sum: 1600"));
    assert!(out.contains("Convolution kernels transform images for blur, sharpen, and edge detection."));
}

#[test]
fn showcase410_lambda_calculus() {
    let out = run_ore("showcase410.ore");
    assert!(out.contains("Lambda Calculus Evaluator:"));
    assert!(out.contains("church(3) applied to succ(0) = 3"));
    assert!(out.contains("plus(2, 3) = 5"));
    assert!(out.contains("mult(3, 4) = 12"));
    assert!(out.contains("pow(2, 5) = 32"));
    assert!(out.contains("apply double 3 times to 1: 8"));
    assert!(out.contains("pred(3) = 2"));
    assert!(out.contains("I(42) = 42"));
    assert!(out.contains("S K K x = I x (proof: S K K 7 = 7)"));
    assert!(out.contains("(Lx. x + 1) 5 => 5 + 1"));
    assert!(out.contains("5! = 120"));
    assert!(out.contains("true AND false = false"));
    assert!(out.contains("Lambda calculus: the foundation of functional programming."));
}

#[test]
fn showcase411_mandelbrot() {
    let out = run_ore("showcase411.ore");
    assert!(out.contains("Mandelbrot Set (ASCII Art):"));
    assert!(out.contains("c=(0, 0): 20 iterations (inside set)"));
    assert!(out.contains("c=(-1, 0): 20 iterations (inside set)"));
    assert!(out.contains("c=(2, 0): 2 iterations (escapes immediately)"));
    assert!(out.contains("Mandelbrot set rendered with fixed-point arithmetic."));
}

#[test]
fn showcase412_genetic_algorithm() {
    let out = run_ore("showcase412.ore");
    assert!(out.contains("Genetic Algorithm (String Evolution):"));
    assert!(out.contains("Target: \"hello world\""));
    assert!(out.contains("Population size: 20"));
    assert!(out.contains("Mutation rate: 15%"));
    assert!(out.contains("Gen 0:"));
    assert!(out.contains("Final result after"));
    assert!(out.contains("Genetic algorithm evolves strings toward a target."));
}

#[test]
fn showcase413_postfix_to_infix() {
    let out = run_ore("showcase413.ore");
    assert!(out.contains("Postfix (RPN) to Infix Converter:"));
    assert!(out.contains("Infix:  3 + 4"));
    assert!(out.contains("Value:  7"));
    assert!(out.contains("Infix:  (3 + 4) * 2"));
    assert!(out.contains("Value:  14"));
    assert!(out.contains("Infix:  (2 + 3) * (4 - 5)"));
    assert!(out.contains("Value:  -5"));
    assert!(out.contains("Infix:  10 - 2"));
    assert!(out.contains("Value:  8"));
    assert!(out.contains("Postfix expressions converted to infix with correct parentheses."));
}

#[test]
fn showcase414_dijkstra() {
    let out = run_ore("showcase414.ore");
    assert!(out.contains("Dijkstra's Shortest Path Algorithm:"));
    assert!(out.contains("to B: distance=1, path=A -> B"));
    assert!(out.contains("to C: distance=2, path=A -> C"));
    assert!(out.contains("to E: distance=4, path=A -> B -> E"));
    assert!(out.contains("to F: distance=11, path=A -> C -> F"));
    assert!(out.contains("to D: distance=20, path=A -> C -> D"));
    assert!(out.contains("Dijkstra's algorithm finds shortest paths in weighted graphs."));
}

#[test]
fn showcase415_look_and_say() {
    let out = run_ore("showcase415.ore");
    assert!(out.contains("Conway's Look-and-Say Sequence:"));
    assert!(out.contains("Term 1: 1 (length 1)"));
    assert!(out.contains("Term 2: 11 (length 2)"));
    assert!(out.contains("Term 3: 21 (length 2)"));
    assert!(out.contains("Term 4: 1211 (length 4)"));
    assert!(out.contains("Term 5: 111221 (length 6)"));
    assert!(out.contains("22: 22 -> 22 -> 22 -> 22 -> 22 -> 22"));
    assert!(out.contains("The look-and-say sequence grows at Conway's constant ratio ~1.303577."));
}

#[test]
fn showcase416_brainfuck() {
    let out = run_ore("showcase416.ore");
    assert!(out.contains("Brainfuck Interpreter:"));
    assert!(out.contains("Output: 1 2 3 4 5"));
    assert!(out.contains("Output: 7"));
    assert!(out.contains("Output: 15"));
    assert!(out.contains("Output: 5 4 3 2 1"));
    assert!(out.contains("Output: 12"));
    assert!(out.contains("Output: 1 4 9 16 25"));
    assert!(out.contains("Output: 1 3 6 10"));
    assert!(out.contains("Brainfuck: Turing-complete with just 8 commands."));
}

#[test]
fn showcase417_text_justification() {
    let out = run_ore("showcase417.ore");
    assert!(out.contains("Text Justification Algorithm:"));
    assert!(out.contains("Justified to 50 columns:"));
    assert!(out.contains("Narrow justification (30 columns):"));
    assert!(out.contains("|              CHAPTER ONE               |"));
    assert!(out.contains("|             The Beginning              |"));
    assert!(out.contains("|             by Author Name             |"));
    assert!(out.contains("Text justification distributes spaces evenly across lines."));
}

#[test]
fn showcase418_power_set() {
    let out = run_ore("showcase418.ore");
    assert!(out.contains("Power Set Generation:"));
    assert!(out.contains("[1 2 3]"));
    assert!(out.contains("Total: 8 subsets"));
    assert!(out.contains("[a b c d]"));
    assert!(out.contains("Total: 16 subsets"));
    assert!(out.contains("[3 1 8] = 12"));
    assert!(out.contains("[8 4] = 12"));
    assert!(out.contains("Found 3 subsets"));
    assert!(out.contains("Size 0: 1 subsets"));
    assert!(out.contains("Size 3: 10 subsets"));
    assert!(out.contains("Power sets grow exponentially: |P(S)| = 2^|S|."));
}

#[test]
fn showcase419_lcs_diff() {
    let out = run_ore("showcase419.ore");
    assert!(out.contains("Longest Common Subsequence with Diff:"));
    assert!(out.contains("LCS length: 4"));
    assert!(out.contains("LCS length: 5"));
    assert!(out.contains("LCS length: 3 (same = no diff)"));
    assert!(out.contains("LCS length: 0"));
    assert!(out.contains("+ 2"));
    assert!(out.contains("- 4"));
    assert!(out.contains("+ 9"));
    assert!(out.contains("LCS computes the longest common subsequence for diff output."));
}

#[test]
fn showcase420_spreadsheet() {
    let out = run_ore("showcase420.ore");
    assert!(out.contains("Simple Spreadsheet Evaluator:"));
    assert!(out.contains("A5 (total revenue) = 5500"));
    assert!(out.contains("C5 (total profit) = 2500"));
    assert!(out.contains("A1 = 1000"));
    assert!(out.contains("D4 = 50"));
    assert!(out.contains("Grade Calculator Spreadsheet:"));
    assert!(out.contains("Weighted average formula: HW*30% + Midterm*30% + Final*40%"));
    assert!(out.contains("Spreadsheet evaluator: formulas compute cell dependencies."));
}

#[test]
fn showcase421_huffman() {
    let out = run_ore("showcase421.ore");
    assert!(out.contains("Huffman Encoding/Decoding:"));
    assert!(out.contains("'a': 5"));
    assert!(out.contains("'a' -> 0"));
    assert!(out.contains("Encoded length: 23 bits"));
    assert!(out.contains("Compression ratio: 26%"));
    assert!(out.contains("Decoded: abracadabra"));
    assert!(out.contains("Decoding verified: matches original!"));
    assert!(out.contains("Huffman coding: optimal prefix-free variable-length encoding."));
}

#[test]
fn showcase422_perceptron() {
    let out = run_ore("showcase422.ore");
    assert!(out.contains("Perceptron Neural Network Training:"));
    assert!(out.contains("(0, 0) -> 0  (expected: 0)"));
    assert!(out.contains("(1, 1) -> 1  (expected: 1)"));
    assert!(out.contains("OR gate results:"));
    assert!(out.contains("NAND gate results:"));
    assert!(out.contains("XOR has 2 errors - not linearly separable!"));
    assert!(out.contains("Perceptron: the simplest neural network unit."));
}

#[test]
fn showcase423_maze() {
    let out = run_ore("showcase423.ore");
    assert!(out.contains("Maze Generation (DFS Backtracker):"));
    assert!(out.contains("48 cells visited"));
    assert!(out.contains("| S "));
    assert!(out.contains(" E |"));
    assert!(out.contains("Path length: 20 steps"));
    assert!(out.contains("Maze generation with DFS backtracker and BFS solver."));
}

#[test]
fn showcase424_rbtree() {
    let out = run_ore("showcase424.ore");
    assert!(out.contains("Red-Black Tree Balancing:"));
    assert!(out.contains("Insert 10: (fixes: 0)"));
    assert!(out.contains("Insert 30: (fixes: 1)"));
    assert!(out.contains("Root is black: true"));
    assert!(out.contains("No red-red parent-child: true"));
    assert!(out.contains("Total nodes: 7"));
    assert!(out.contains("Red-black trees: self-balancing BST with O(log n) operations."));
}

#[test]
fn showcase425_pretty_printer() {
    let out = run_ore("showcase425.ore");
    assert!(out.contains("JSON-like Pretty Printer:"));
    assert!(out.contains("name: Alice"));
    assert!(out.contains("age: 30"));
    assert!(out.contains("- 95"));
    assert!(out.contains("host: localhost"));
    assert!(out.contains("Total nodes: 8"));
    assert!(out.contains("Pretty printing: structured display of nested data."));
}

#[test]
fn showcase426_forth() {
    let out = run_ore("showcase426.ore");
    assert!(out.contains("Forth-like Stack Language Interpreter:"));
    assert!(out.contains("=> 7"));
    assert!(out.contains("=> 16"));
    assert!(out.contains("=> 49"));
    assert!(out.contains("=> -7"));
    assert!(out.contains("5! = 120"));
    assert!(out.contains("0 1 1 2 3 5 8 13 21 34"));
    assert!(out.contains("Result: 110"));
    assert!(out.contains("Forth: stack-based concatenative programming language."));
}

#[test]
fn showcase427_lu_decomposition() {
    let out = run_ore("showcase427.ore");
    assert!(out.contains("LU Decomposition (Integer Arithmetic):"));
    assert!(out.contains("Decomposition verified: L * U = A"));
    assert!(out.contains("Forward substitution (y): [100, 0, -100]"));
    assert!(out.contains("Back substitution (x): [50, 50, -50]"));
    assert!(out.contains("Row 0: 100 (expected: 100)"));
    assert!(out.contains("LU decomposition: factor matrix into lower and upper triangular."));
}

#[test]
fn showcase428_dfa_minimization() {
    let out = run_ore("showcase428.ore");
    assert!(out.contains("DFA Minimization:"));
    assert!(out.contains("Original DFA (5 states):"));
    assert!(out.contains("States: 3 (reduced from 5)"));
    assert!(out.contains("Minimized state 0 = original states [0, 3]"));
    assert!(out.contains("'ab' -> minimized state 2, accept: true"));
    assert!(out.contains("'bb' -> minimized state 0, accept: false"));
    assert!(out.contains("DFA minimization: equivalent states merged via partition refinement."));
}

#[test]
fn showcase429_compiler_phases() {
    let out = run_ore("showcase429.ore");
    assert!(out.contains("Compiler Phases Demo (Lex -> Parse -> Eval):"));
    assert!(out.contains("=== Phase 1: Lexer ==="));
    assert!(out.contains("[0] NUM(3)"));
    assert!(out.contains("[3] STAR"));
    assert!(out.contains("=== Phase 2: Parser ==="));
    assert!(out.contains("AST (7 nodes):"));
    assert!(out.contains("=== Phase 3: Evaluator ==="));
    assert!(out.contains("Result: 3 + x * (2 + y) = 48"));
    assert!(out.contains("Compiler pipeline: source -> tokens -> AST -> result."));
}

#[test]
fn showcase430_dynamic_programming() {
    let out = run_ore("showcase430.ore");
    assert!(out.contains("Dynamic Programming Collection:"));
    assert!(out.contains("Max revenue: 22"));
    assert!(out.contains("Optimal cuts: 2 + 6"));
    assert!(out.contains("Minimum coins: 3"));
    assert!(out.contains("LIS length: 4"));
    assert!(out.contains("Edit distance: 3"));
    assert!(out.contains("Max value: 10"));
    assert!(out.contains("Dynamic programming: optimal substructure + overlapping subproblems."));
}

#[test]
fn showcase431_deque() {
    let out = run_ore("showcase431.ore");
    assert!(out.contains("Deque (Double-Ended Queue):"));
    assert!(out.contains("push_back(30) -> deque: 10, 20, 30"));
    assert!(out.contains("pop_front() -> 10, deque: 20, 30, 40, 50"));
    assert!(out.contains("push_front(1) -> deque: 1, 5, 30, 40, 50"));
    assert!(out.contains("pop_back() -> 50, deque: 1, 5, 30, 40"));
    assert!(out.contains("maximums: 3, 3, 5, 5, 6, 7"));
    assert!(out.contains("\"racecar\" is a palindrome"));
    assert!(out.contains("\"hello\" is not a palindrome"));
    assert!(out.contains("Deque: versatile double-ended queue for stacks, queues, and sliding windows."));
}

#[test]
fn showcase432_trie() {
    let out = run_ore("showcase432.ore");
    assert!(out.contains("Trie (Prefix Tree):"));
    assert!(out.contains("Total nodes: 14"));
    assert!(out.contains("search(\"apple\") -> found"));
    assert!(out.contains("search(\"ap\") -> prefix only, not a word"));
    assert!(out.contains("search(\"bay\") -> not found"));
    assert!(out.contains("LCP: \"ap\""));
    assert!(out.contains("Trie: efficient prefix-based string storage and retrieval."));
}

#[test]
fn showcase433_skip_list() {
    let out = run_ore("showcase433.ore");
    assert!(out.contains("Skip List Simulation:"));
    assert!(out.contains("insert(12) at level 2"));
    assert!(out.contains("Level 0: 3 -> 6 -> 7 -> 9 -> 12 -> 17 -> 19 -> 21 -> 25 -> 26"));
    assert!(out.contains("Result: found"));
    assert!(out.contains("Result: not found"));
    assert!(out.contains("Total elements: 10"));
    assert!(out.contains("Skip list: probabilistic O(log n) search via express lanes."));
}

#[test]
fn showcase434_levenshtein() {
    let out = run_ore("showcase434.ore");
    assert!(out.contains("Levenshtein Distance and Fuzzy Matching:"));
    assert!(out.contains("d(\"kitten\", \"sitting\") = 3"));
    assert!(out.contains("d(\"abc\", \"abc\") = 0"));
    assert!(out.contains("\"pythn\" -> best match: \"python\" (distance 1)"));
    assert!(out.contains("\"teh\" -> \"the\""));
    assert!(out.contains("Corrections made: 5"));
    assert!(out.contains("\"ore\" vs \"ore\": 100% similar (distance 0)"));
    assert!(out.contains("Levenshtein: foundational metric for fuzzy string matching."));
}

#[test]
fn showcase435_kmeans() {
    let out = run_ore("showcase435.ore");
    assert!(out.contains("K-Means Clustering (1D):"));
    assert!(out.contains("Initial centroids: 2, 15, 30"));
    assert!(out.contains("Cluster 0 (5 points): 1, 2, 3, 4, 5"));
    assert!(out.contains("Cluster 1 (5 points): 10, 11, 12, 14, 15"));
    assert!(out.contains("Cluster 2 (5 points): 20, 22, 25, 28, 30"));
    assert!(out.contains("Final centroids: 3, 12, 25"));
    assert!(out.contains("WCSS (within-cluster sum of squares): 96"));
    assert!(out.contains("K-means: iterative centroid-based clustering algorithm."));
}

#[test]
fn showcase436_sat_solver() {
    let out = run_ore("showcase436.ore");
    assert!(out.contains("SAT Solver (Brute Force):"));
    assert!(out.contains("SATISFIABLE"));
    assert!(out.contains("UNSATISFIABLE"));
    assert!(out.contains("First solution: x1=0, x2=1, x3=0"));
    assert!(out.contains("Formula 1: SAT (2 models out of 8)"));
    assert!(out.contains("Formula 2: UNSAT (0 models out of 2)"));
    assert!(out.contains("Formula 3: SAT (3 models out of 8)"));
    assert!(out.contains("SAT solver: exhaustive search over boolean assignments."));
}

#[test]
fn showcase437_intervals() {
    let out = run_ore("showcase437.ore");
    assert!(out.contains("Interval Operations:"));
    assert!(out.contains("Merged intervals (2):"));
    assert!(out.contains("[1, 13)"));
    assert!(out.contains("[15, 20)"));
    assert!(out.contains("Point 3: in 3 interval(s)"));
    assert!(out.contains("Point 20: not in any interval"));
    assert!(out.contains("Total overlapping pairs: 8"));
    assert!(out.contains("Coverage: 89%"));
    assert!(out.contains("Interval operations: merge, query, intersect, and coverage analysis."));
}

#[test]
fn showcase438_bwt() {
    let out = run_ore("showcase438.ore");
    assert!(out.contains("Burrows-Wheeler Transform:"));
    assert!(out.contains("BWT output: annb.aa"));
    assert!(out.contains("Original string at row: 4"));
    assert!(out.contains("Reconstructed: banana."));
    assert!(out.contains("Verification: PASSED"));
    assert!(out.contains("Burrows-Wheeler: reversible transform for improved compression."));
}

#[test]
fn showcase439_toposort() {
    let out = run_ore("showcase439.ore");
    assert!(out.contains("Topological Sort with Cycle Detection:"));
    assert!(out.contains("Topological order: Math -> CS101 -> CS201 -> DB -> CS301 -> AI"));
    assert!(out.contains("No cycle detected"));
    assert!(out.contains("CYCLE DETECTED! Could only order 1 of 4 nodes"));
    assert!(out.contains("Nodes involved in cycle: B, C, D"));
    assert!(out.contains("1. libutil"));
    assert!(out.contains("4. app"));
    assert!(out.contains("Topological sort: ordering nodes respecting directed dependencies."));
}

#[test]
fn showcase440_kmp() {
    let out = run_ore("showcase440.ore");
    assert!(out.contains("KMP String Matching Algorithm:"));
    assert!(out.contains("Failure function: 0, 0, 1, 2, 0, 1, 2, 3, 4"));
    assert!(out.contains("Found at position(s): 10"));
    assert!(out.contains("Total matches: 7"));
    assert!(out.contains("Not found"));
    assert!(out.contains("Found at position(s): 0, 8, 12"));
    assert!(out.contains("Naive comparisons: 85"));
    assert!(out.contains("KMP: linear-time string matching with failure function preprocessing."));
}

#[test]
fn showcase441_prims() {
    let out = run_ore("showcase441.ore");
    assert!(out.contains("Prim's Algorithm (Minimum Spanning Tree):"));
    assert!(out.contains("Step 1: Add edge 0-2 (weight 2)"));
    assert!(out.contains("Total MST weight: 13"));
    assert!(out.contains("All 6 vertices are in the MST."));
    assert!(out.contains("Prim's: greedy MST algorithm using cut property."));
}

#[test]
fn showcase442_floyd_warshall() {
    let out = run_ore("showcase442.ore");
    assert!(out.contains("Floyd-Warshall (All-Pairs Shortest Paths):"));
    assert!(out.contains("0 -> 1: 1"));
    assert!(out.contains("0 -> 2: -3"));
    assert!(out.contains("0 -> 4: -4"));
    assert!(out.contains("No negative cycles."));
    assert!(out.contains("Floyd-Warshall: O(V^3) all-pairs shortest paths with negative edge support."));
}

#[test]
fn showcase443_toposort_dfs() {
    let out = run_ore("showcase443.ore");
    assert!(out.contains("Topological Sort via DFS (finishing times):"));
    assert!(out.contains("A -> B, C"));
    assert!(out.contains("G -> (none)"));
    assert!(out.contains("Ordering is valid!"));
    assert!(out.contains("A: finish time 7"));
    assert!(out.contains("G: finish time 1"));
    assert!(out.contains("Topological sort: DFS post-order reversal for DAG linearization."));
}

#[test]
fn showcase444_ford_fulkerson() {
    let out = run_ore("showcase444.ore");
    assert!(out.contains("Ford-Fulkerson Maximum Flow:"));
    assert!(out.contains("Maximum Flow: 23"));
    assert!(out.contains("path 0->1->3->5 bottleneck=12"));
    assert!(out.contains("1->3: 12/12"));
    assert!(out.contains("Ford-Fulkerson: augmenting path method for maximum network flow."));
}

#[test]
fn showcase445_bellman_ford() {
    let out = run_ore("showcase445.ore");
    assert!(out.contains("Bellman-Ford Shortest Path Algorithm:"));
    assert!(out.contains("No negative-weight cycles found."));
    assert!(out.contains("To 3: 2"));
    assert!(out.contains("0 -> 3: [0 -> 1 -> 3] distance=2"));
    assert!(out.contains("NEGATIVE CYCLE DETECTED"));
    assert!(out.contains("Bellman-Ford: handles negative weights, detects negative cycles in O(VE)."));
}

#[test]
fn showcase446_kosaraju() {
    let out = run_ore("showcase446.ore");
    assert!(out.contains("Kosaraju's Algorithm (Strongly Connected Components):"));
    assert!(out.contains("SCC 0: [3, 7]"));
    assert!(out.contains("SCC 1: [4, 5, 6]"));
    assert!(out.contains("SCC 2: [0, 1, 2]"));
    assert!(out.contains("Total SCCs: 3"));
    assert!(out.contains("Kosaraju's: two-pass DFS algorithm for finding SCCs in O(V+E)."));
}

#[test]
fn showcase447_euler() {
    let out = run_ore("showcase447.ore");
    assert!(out.contains("Euler Path and Circuit Detection:"));
    assert!(out.contains("Euler CIRCUIT exists (all degrees even)"));
    assert!(out.contains("Euler PATH exists between vertices 0 and 3"));
    assert!(out.contains("No Euler path or circuit (4 odd-degree vertices)"));
    assert!(out.contains("Directed Euler CIRCUIT exists (in-degree = out-degree for all)"));
    assert!(out.contains("Euler paths: exist iff 0 or 2 odd-degree vertices (undirected graphs)."));
}

#[test]
fn showcase448_bipartite() {
    let out = run_ore("showcase448.ore");
    assert!(out.contains("Bipartite Graph Checking (BFS 2-coloring):"));
    assert!(out.contains("Set A (red):  [0, 2, 4]"));
    assert!(out.contains("Set B (blue): [1, 3, 5]"));
    assert!(out.contains("Bipartite: NO (conflict at edge"));
    assert!(out.contains("Bipartite: YES (all trees are bipartite)"));
    assert!(out.contains("Bipartite check: BFS 2-coloring detects odd cycles in O(V+E)."));
}

#[test]
fn showcase449_articulation() {
    let out = run_ore("showcase449.ore");
    assert!(out.contains("Articulation Points and Bridges:"));
    assert!(out.contains("Found 3: [1, 3, 4]"));
    assert!(out.contains("Found 2:"));
    assert!(out.contains("1 - 3"));
    assert!(out.contains("3 - 4"));
    assert!(out.contains("Removing vertex 1: graph disconnected"));
    assert!(out.contains("Articulation points and bridges: found via DFS disc/low values."));
}

#[test]
fn showcase450_graph_coloring() {
    let out = run_ore("showcase450.ore");
    assert!(out.contains("Graph Coloring:"));
    assert!(out.contains("Greedy chromatic number: 3"));
    assert!(out.contains("Coloring is valid!"));
    assert!(out.contains("k=2: No valid coloring"));
    assert!(out.contains("Chromatic number: 3"));
    assert!(out.contains("Valid coloring confirmed."));
    assert!(out.contains("Graph coloring: greedy gives upper bound, backtracking finds exact chromatic number."));
}

#[test]
fn showcase451_physics() {
    let out = run_ore("showcase451.ore");
    assert!(out.contains("Simple Physics Engine:"));
    assert!(out.contains("Projectile Motion"));
    assert!(out.contains("Momentum conserved!"));
    assert!(out.contains("Kinetic energy conserved!"));
    assert!(out.contains("Bouncing Ball"));
    assert!(out.contains("Bounce 5:"));
    assert!(out.contains("Physics engine: projectile motion, elastic collisions, bouncing simulation."));
}

#[test]
fn showcase452_music_theory() {
    let out = run_ore("showcase452.ore");
    assert!(out.contains("Music Theory Engine:"));
    assert!(out.contains("C Major: C D E F G A B C"));
    assert!(out.contains("A Minor: A B C D E F G A"));
    assert!(out.contains("C Major: C E G"));
    assert!(out.contains("C Dominant 7th: C E G A#"));
    assert!(out.contains("Circle of Fifths"));
    assert!(out.contains("C -> G -> D -> A -> E -> B"));
    assert!(out.contains("I-V-vi-IV: C -> G -> Am -> F"));
    assert!(out.contains("Music theory: scales, chords, intervals, circle of fifths."));
}

#[test]
fn showcase453_calendar() {
    let out = run_ore("showcase453.ore");
    assert!(out.contains("Calendar Calculations:"));
    assert!(out.contains("Moon Landing (1969-7-20): Sunday"));
    assert!(out.contains("Y2K (2000-1-1): Saturday"));
    assert!(out.contains("Total: 366 days (leap year)"));
    assert!(out.contains("Feb: 29 days"));
    assert!(out.contains("Count: 13"));
    assert!(out.contains("Calendar: day of week, date differences, leap years, month analysis."));
}

#[test]
fn showcase454_color_spaces() {
    let out = run_ore("showcase454.ore");
    assert!(out.contains("Color Space Conversions:"));
    assert!(out.contains("Red: RGB(255, 0, 0) = #FF0000"));
    assert!(out.contains("White: RGB(255, 255, 255) = #FFFFFF"));
    assert!(out.contains("Red: HSL(0, 100%, 50%)"));
    assert!(out.contains("#808080 -> RGB(128, 128, 128) [Gray]"));
    assert!(out.contains("Grayscale Conversion"));
    assert!(out.contains("Color spaces: RGB, HSL, hex, blending, grayscale conversion."));
}

#[test]
fn showcase455_unit_converter() {
    let out = run_ore("showcase455.ore");
    assert!(out.contains("Unit Converter:"));
    assert!(out.contains("0C = 32F = 273K"));
    assert!(out.contains("100C = 212F = 373K"));
    assert!(out.contains("-40C = -40F"));
    assert!(out.contains("100 km/h = 62 mph"));
    assert!(out.contains("1 pound = 0.454 kg"));
    assert!(out.contains("Unit converter: temperature, distance, weight, speed, area."));
}

#[test]
fn showcase456_probability() {
    let out = run_ore("showcase456.ore");
    assert!(out.contains("Probability Calculations:"));
    assert!(out.contains("10! = 3628800"));
    assert!(out.contains("C(5,*): 1 5 10 10 5 1"));
    assert!(out.contains("Sum 7: 6 ways"));
    assert!(out.contains("Expected value of 2d6: 7"));
    assert!(out.contains("Bayes' Theorem"));
    assert!(out.contains("23 people: ~50.7% chance of shared birthday"));
    assert!(out.contains("Total 5-card hands: 2598960"));
    assert!(out.contains("Probability: combinations, dice, Bayes theorem, birthday problem, poker."));
}

#[test]
fn showcase457_database() {
    let out = run_ore("showcase457.ore");
    assert!(out.contains("Simple Database Engine:"));
    assert!(out.contains("Total records: 8"));
    assert!(out.contains("Eng: Alice, Bob, Diana, Grace"));
    assert!(out.contains("Query: salary > 80000"));
    assert!(out.contains("AVG(salary) GROUP BY dept"));
    assert!(out.contains("Eng: avg=$83750 (count=4)"));
    assert!(out.contains("Binary Search on Salary Index"));
    assert!(out.contains("Database engine: insert, index, query, binary search, update."));
}

#[test]
fn showcase458_text_adventure() {
    let out = run_ore("showcase458.ore");
    assert!(out.contains("Text Adventure Game Engine:"));
    assert!(out.contains("You are in: Entrance Hall"));
    assert!(out.contains("Picked up: torch"));
    assert!(out.contains("You go north."));
    assert!(out.contains("You are now in: Library"));
    assert!(out.contains("Items collected: 5/5"));
    assert!(out.contains("Final score: 50"));
    assert!(out.contains("Text adventure: rooms, items, navigation, inventory, scoring."));
}

#[test]
fn showcase459_compression() {
    let out = run_ore("showcase459.ore");
    assert!(out.contains("Compression Analysis Engine:"));
    assert!(out.contains("\"AAABBBCCCCDDAA\" -> \"3A3B4C2D2A\""));
    assert!(out.contains("\"AAAAAAAAA\" -> \"9A\" (ratio: 22%)"));
    assert!(out.contains("Frequency Analysis"));
    assert!(out.contains("Dictionary Compression"));
    assert!(out.contains("Compression analysis: RLE, frequency, entropy, dictionary methods."));
}

#[test]
fn showcase460_differentiation() {
    let out = run_ore("showcase460.ore");
    assert!(out.contains("Symbolic Differentiation Engine:"));
    assert!(out.contains("d/dx [x^n]   = n*x^(n-1) (power rule)"));
    assert!(out.contains("f'(x) = 6x + 5"));
    assert!(out.contains("f(1) = 15, f'(1) = 11"));
    assert!(out.contains("Newton's Method"));
    assert!(out.contains("Step 4: x = 1.414213"));
    assert!(out.contains("Both methods agree!"));
    assert!(out.contains("Symbolic differentiation: polynomial, product, chain, Newton's method."));
}

#[test]
fn showcase461_event_sourcing() {
    let out = run_ore("showcase461.ore");
    assert!(out.contains("Event Sourcing Pattern:"));
    assert!(out.contains("+ 1000 -> balance = 1000"));
    assert!(out.contains("Final balance: 2100"));
    assert!(out.contains("Balance at time 2: 1500"));
    assert!(out.contains("Balance at time 4: 1600"));
    assert!(out.contains("Deposits: 4 totaling 2550"));
    assert!(out.contains("Withdrawals: 3 totaling 450"));
    assert!(out.contains("Balance after undoing last 2 events: 1450"));
    assert!(out.contains("Peak balance: 2100"));
    assert!(out.contains("Event sourcing: event log, replay, point-in-time, undo, statistics."));
}

#[test]
fn showcase462_register_vm() {
    let out = run_ore("showcase462.ore");
    assert!(out.contains("Simple Register VM:"));
    assert!(out.contains("MUL r4 = r2 * r3 = 20"));
    assert!(out.contains("Result: 20"));
    assert!(out.contains("Sum 1..5 = 15"));
    assert!(out.contains("fib(9) = 34"));
    assert!(out.contains("Register VM: arithmetic, loops, jumps, fibonacci."));
}

#[test]
fn showcase463_constraint_satisfaction() {
    let out = run_ore("showcase463.ore");
    assert!(out.contains("Constraint Satisfaction Solver:"));
    assert!(out.contains("Solution 1: [1, 3, 0, 2]"));
    assert!(out.contains("Solution 2: [2, 0, 3, 1]"));
    assert!(out.contains("Total 4-Queens solutions: 2"));
    assert!(out.contains("Valid magic square!"));
    assert!(out.contains("All columns sum to 10 - valid latin square!"));
    assert!(out.contains("[7, 8]"));
    assert!(out.contains("Total subsets summing to 15: 10"));
    assert!(out.contains("Constraint satisfaction: n-queens, magic square, latin square, subset sum."));
}

#[test]
fn showcase464_mcmc() {
    let out = run_ore("showcase464.ore");
    assert!(out.contains("Markov Chain Monte Carlo:"));
    assert!(out.contains("Pi estimate (x1000): 3118"));
    assert!(out.contains("Metropolis Sampler"));
    assert!(out.contains("Random Walk Statistics"));
    assert!(out.contains("Walk 1:"));
    assert!(out.contains("E[X^2] for dice (6000 rolls): 14"));
    assert!(out.contains("MCMC: pi estimation, Metropolis sampler, random walks, expected values."));
}

#[test]
fn showcase465_ast_pretty_printer() {
    let out = run_ore("showcase465.ore");
    assert!(out.contains("AST Pretty Printer:"));
    assert!(out.contains("Prefix:  * + 3 4 2"));
    assert!(out.contains("Postfix: 3 4 + 2 *"));
    assert!(out.contains("Result: 23"));
    assert!(out.contains("Result: 49"));
    assert!(out.contains("Node 8 (div): 15"));
    assert!(out.contains("Final result: 15"));
    assert!(out.contains("Reconstructed infix: (((2 + 3) * (7 - 1)) / 2)"));
    assert!(out.contains("AST pretty printer: trees, evaluation, traversal, serialization."));
}

#[test]
fn showcase466_build_system() {
    let out = run_ore("showcase466.ore");
    assert!(out.contains("Simple Build System:"));
    assert!(out.contains("app <- main.o, libapp.a"));
    assert!(out.contains("Step 1: build main.o"));
    assert!(out.contains("Step 5: build app"));
    assert!(out.contains("utils.o: REBUILD (source newer)"));
    assert!(out.contains("[BUILD] ar rcs libapp.a utils.o math.o"));
    assert!(out.contains("Built 4 targets, skipped 1"));
    assert!(out.contains("Level 0: [main.o, utils.o, math.o]"));
    assert!(out.contains("Build system: dependencies, topo-sort, rebuild check, parallel analysis."));
}

#[test]
fn showcase467_othello() {
    let out = run_ore("showcase467.ore");
    assert!(out.contains("Othello/Reversi Game:"));
    assert!(out.contains("Valid moves for Black:"));
    assert!(out.contains("d3"));
    assert!(out.contains("Flipped 1 piece(s)"));
    assert!(out.contains("Score: Black=3, White=3"));
    assert!(out.contains("Othello: board display, valid moves, flipping, scoring."));
}

#[test]
fn showcase468_gc_simulation() {
    let out = run_ore("showcase468.ore");
    assert!(out.contains("Garbage Collector Simulation:"));
    assert!(out.contains("Root set: [1, 7]"));
    assert!(out.contains("Marked 4 objects"));
    assert!(out.contains("[3] Map: FREED (256B)"));
    assert!(out.contains("Freed: 4 objects, 416 bytes"));
    assert!(out.contains("Live: 4 objects, 288 bytes"));
    assert!(out.contains("Heap after compaction: 288 bytes used"));
    assert!(out.contains("Generation 0 (young): 5 objects"));
    assert!(out.contains("GC simulation: mark-sweep, compaction, generations, statistics."));
}

#[test]
fn showcase469_http_parser() {
    let out = run_ore("showcase469.ore");
    assert!(out.contains("HTTP Request Parser:"));
    assert!(out.contains("Method: GET"));
    assert!(out.contains("Path: /index.html"));
    assert!(out.contains("Host = example.com"));
    assert!(out.contains("Method: POST"));
    assert!(out.contains("name = John"));
    assert!(out.contains("page = 1"));
    assert!(out.contains("/ -> home_handler"));
    assert!(out.contains("/missing -> 404 Not Found"));
    assert!(out.contains("HTTP parser: request parsing, URL parsing, routing, response building."));
}

#[test]
fn showcase470_etl_pipeline() {
    let out = run_ore("showcase470.ore");
    assert!(out.contains("Data Pipeline (ETL):"));
    assert!(out.contains("Extracted 10 records"));
    assert!(out.contains("Alice: Engineering, $85000, 5yr -> Mid"));
    assert!(out.contains("Hank: Engineering, $95000, 8yr -> Senior"));
    assert!(out.contains("Avg salary: $87500"));
    assert!(out.contains("#1: Hank (score=119, dept=Engineering)"));
    assert!(out.contains("All records valid!"));
    assert!(out.contains("Total payroll: $747000"));
    assert!(out.contains("Junior: 4, Mid: 4, Senior: 2"));
    assert!(out.contains("ETL pipeline: extract, transform, aggregate, validate, report."));
}

#[test]
fn showcase471_packet_router() {
    let out = run_ore("showcase471.ore");
    assert!(out.contains("Packet Router Simulation:"));
    assert!(out.contains("192.168.1.0 -> eth0 (metric=1)"));
    assert!(out.contains("Packets in queue: 8"));
    assert!(out.contains("Pkt 5 (192.168.3.0) -> DROPPED (no route)"));
    assert!(out.contains("Routed: 7"));
    assert!(out.contains("Dropped: 1"));
    assert!(out.contains("eth2: 3 packets"));
    assert!(out.contains("GET /index -> server-A"));
    assert!(out.contains("PUT /data -> server-A"));
    assert!(out.contains("#1: [1] critical-1"));
    assert!(out.contains("#6: [5] idle"));
    assert!(out.contains("Packet router: routing, load balancing, priority queues."));
}

#[test]
fn showcase472_finite_automaton() {
    let out = run_ore("showcase472.ore");
    assert!(out.contains("Finite Automaton Simulation:"));
    assert!(out.contains("\"ac\" -> ACCEPT"));
    assert!(out.contains("\"abbc\" -> ACCEPT"));
    assert!(out.contains("\"bc\" -> REJECT"));
    assert!(out.contains("\"abb\" matches (a|b)*abb -> YES"));
    assert!(out.contains("\"ab\" matches (a|b)*abb -> NO"));
    assert!(out.contains("\"abababb\" matches (a|b)*abb -> YES"));
    assert!(out.contains("DFA has 4 states (already minimal)"));
    assert!(out.contains("'b': S2 -> S3"));
    assert!(out.contains("Result: ACCEPT"));
    assert!(out.contains("Finite automata: DFA, NFA simulation, transitions, minimization."));
}

#[test]
fn showcase473_chess_moves() {
    let out = run_ore("showcase473.ore");
    assert!(out.contains("Chess Move Validation:"));
    assert!(out.contains("Rook (3,3) -> (3,5): VALID"));
    assert!(out.contains("Rook (3,3) -> (5,6): INVALID"));
    assert!(out.contains("Knight (4,4) -> (2,3): VALID"));
    assert!(out.contains("Knight (4,4) -> (5,5): INVALID"));
    assert!(out.contains("Bishop (3,3) -> (5,5): VALID"));
    assert!(out.contains("Bishop (3,3) -> (4,3): INVALID"));
    assert!(out.contains("Valid king moves: 8"));
    assert!(out.contains("corner(0,0): 2 moves"));
    assert!(out.contains("center(3,3): 8 moves"));
    assert!(out.contains("Opening: Ruy Lopez"));
    assert!(out.contains("Chess: rook, knight, bishop, king validation, mobility, notation."));
}

#[test]
fn showcase474_crossword_solver() {
    let out = run_ore("showcase474.ore");
    assert!(out.contains("Crossword Puzzle Solver:"));
    assert!(out.contains("Row 2, col 0, len 5"));
    assert!(out.contains("Col 0, row 0, len 5"));
    assert!(out.contains("Match: CAN"));
    assert!(out.contains("Match: DANCE"));
    assert!(out.contains("HELLO: 9"));
    assert!(out.contains("CANDY: 7"));
    assert!(out.contains("Across words: CAT, DANCE, DEN"));
    assert!(out.contains("Crossword: grid layout, word slots, constraints, scoring, fill."));
}

#[test]
fn showcase475_eigenvalue_estimation() {
    let out = run_ore("showcase475.ore");
    assert!(out.contains("Matrix Eigenvalue Estimation:"));
    assert!(out.contains("Result: [3, 5, 3]"));
    assert!(out.contains("eigenvalue~4"));
    assert!(out.contains("Expected dominant eigenvalue: 4"));
    assert!(out.contains("A*v = [2000, 4000, 2000]"));
    assert!(out.contains("Trace: 7"));
    assert!(out.contains("Determinant: 8"));
    assert!(out.contains("Matrix is symmetric"));
    assert!(out.contains("Eigenvalues: power iteration, trace, determinant, symmetry."));
}

#[test]
fn showcase476_turing_machine() {
    let out = run_ore("showcase476.ore");
    assert!(out.contains("Turing Machine Simulator:"));
    assert!(out.contains("Output: 0 1 1 1 1 0 0 0"));
    assert!(out.contains("Output: 2 0 1 0 0 1 2 2"));
    assert!(out.contains("Ones written: 4"));
    assert!(out.contains("Steps: 6"));
    assert!(out.contains("BB(2): 4 ones in 6 steps"));
    assert!(out.contains("BB(4): 13 ones in 107 steps"));
    assert!(out.contains("Turing machine: incrementer, inverter, busy beaver, transitions."));
}

#[test]
fn showcase477_ecs_pattern() {
    let out = run_ore("showcase477.ore");
    assert!(out.contains("Entity Component System:"));
    assert!(out.contains("Created 6 entities"));
    assert!(out.contains("Player: Position Health Velocity Damage"));
    assert!(out.contains("Enemy1: (10,8) -> (9,9)"));
    assert!(out.contains("Bullet: (6,6) -> (9,6)"));
    assert!(out.contains("Enemy1 takes 25 dmg: 50 -> 25 HP"));
    assert!(out.contains("Alive entities: 6/6"));
    assert!(out.contains("Spawned: Bullet2 at (5,6)"));
    assert!(out.contains("ECS: entities, components, movement, collision, damage, queries."));
}

#[test]
fn showcase478_expression_tree() {
    let out = run_ore("showcase478.ore");
    assert!(out.contains("Expression Tree Builder:"));
    assert!(out.contains("Result: (3 + 5) * (10 - 2) = 64"));
    assert!(out.contains("Infix: ((3 + 5) * (10 - 2))"));
    assert!(out.contains("Prefix: * + 3 5 - 10 2"));
    assert!(out.contains("Postfix: 3 5 + 10 2 - *"));
    assert!(out.contains("RPN result: 64"));
    assert!(out.contains("Tree depth: 2"));
    assert!(out.contains("Leaf nodes: 4"));
    assert!(out.contains("Expression trees: build, evaluate, infix, prefix, postfix, RPN."));
}

#[test]
fn showcase479_filesystem_sim() {
    let out = run_ore("showcase479.ore");
    assert!(out.contains("File System Simulation:"));
    assert!(out.contains("Created 12 filesystem entries"));
    assert!(out.contains("f doc.txt (1024 bytes)"));
    assert!(out.contains("/home/user/photo.jpg (5120 bytes)"));
    assert!(out.contains("/: 17152 bytes (6 files)"));
    assert!(out.contains("*.log: system.log, error.log"));
    assert!(out.contains("Total files: 6"));
    assert!(out.contains("Total size: 17152 bytes"));
    assert!(out.contains("Largest file: system.log (8192 bytes)"));
    assert!(out.contains("Filesystem: listing, paths, sizes, search, disk usage."));
}

#[test]
fn showcase480_battleship() {
    let out = run_ore("showcase480.ore");
    assert!(out.contains("Battleship Game Logic:"));
    assert!(out.contains("Placed 5 ships (17 cells total)"));
    assert!(out.contains("Shot (1,3): HIT!"));
    assert!(out.contains("Shot (0,0): miss"));
    assert!(out.contains("Carrier: 3/5 hits"));
    assert!(out.contains("Hits: 7"));
    assert!(out.contains("Misses: 3"));
    assert!(out.contains("Ship cells remaining: 10"));
    assert!(out.contains("Game continues..."));
    assert!(out.contains("Battleship: placement, firing, ship status, statistics."));
}

#[test]
fn showcase481_nim_variations() {
    let out = run_ore("showcase481.ore");
    assert!(out.contains("Player 1 wins!"));
    assert!(out.contains("Wythoff's Game"));
    assert!(out.contains("Sprague-Grundy Values"));
    assert!(out.contains("Fibonacci Nim"));
}

#[test]
fn showcase482_dns_resolver() {
    let out = run_ore("showcase482.ore");
    assert!(out.contains("RESOLVED -> 93.184.216.34"));
    assert!(out.contains("NXDOMAIN"));
    assert!(out.contains("CACHE HIT"));
    assert!(out.contains("Hit rate: 40%"));
    assert!(out.contains("CNAME Chain Resolution"));
}

#[test]
fn showcase483_memory_allocator() {
    let out = run_ore("showcase483.ore");
    assert!(out.contains("Alloc A (128B): placed at offset 0"));
    assert!(out.contains("Alloc E (128B): FAILED"));
    assert!(out.contains("Coalescing adjacent blocks"));
    assert!(out.contains("Fragmentation: 46%"));
}

#[test]
fn showcase484_scheduling() {
    let out = run_ore("showcase484.ore");
    assert!(out.contains("FCFS"));
    assert!(out.contains("SJF (Non-Preemptive)"));
    assert!(out.contains("Round Robin (quantum=3)"));
    assert!(out.contains("Priority Scheduling"));
    assert!(out.contains("Gantt Chart"));
}

#[test]
fn showcase485_bloom_filter() {
    let out = run_ore("showcase485.ore");
    assert!(out.contains("Bloom Filter Simulation"));
    assert!(out.contains("PROBABLY IN (true positive)"));
    assert!(out.contains("false positive!"));
    assert!(out.contains("Counting Bloom Filter"));
}

#[test]
fn showcase486_logic_gates() {
    let out = run_ore("showcase486.ore");
    assert!(out.contains("Half Adder"));
    assert!(out.contains("Full Adder"));
    assert!(out.contains("4-Bit Adder"));
    assert!(out.contains("S = 1000 (8)"));
    assert!(out.contains("SR Latch"));
}

#[test]
fn showcase487_game_of_life() {
    let out = run_ore("showcase487.ore");
    assert!(out.contains("Blinker"));
    assert!(out.contains("Block (Still Life)"));
    assert!(out.contains("Glider"));
    assert!(out.contains("Pattern Catalog"));
    assert!(out.contains("Population Over Time"));
}

#[test]
fn showcase488_inventory() {
    let out = run_ore("showcase488.ore");
    assert!(out.contains("Product Catalog"));
    assert!(out.contains("Total revenue: 113250"));
    assert!(out.contains("ALERT: Widget"));
    assert!(out.contains("Total value: 106500"));
    assert!(out.contains("ABC Analysis"));
}

#[test]
fn showcase489_regex_nfa() {
    let out = run_ore("showcase489.ore");
    assert!(out.contains("Regex to NFA"));
    assert!(out.contains("'ab' accepted: yes"));
    assert!(out.contains("'a' accepted: true"));
    assert!(out.contains("NFA to DFA"));
}

#[test]
fn showcase490_arithmetic_coding() {
    let out = run_ore("showcase490.ore");
    assert!(out.contains("Arithmetic Coding Concepts"));
    assert!(out.contains("Decoded: 'abra'"));
    assert!(out.contains("Savings: ~30%"));
    assert!(out.contains("Adaptive vs Static"));
}

#[test]
fn showcase491_ray_casting() {
    let out = run_ore("showcase491.ore");
    assert!(out.contains("2D Ray Casting:"));
    assert!(out.contains("Ray 45deg: HIT at (40, 40)"));
    assert!(out.contains("E: BLOCKED"));
    assert!(out.contains("Top-Down View"));
    assert!(out.contains("Ray casting: intersection, walls, shadows, rendering, shading."));
}

#[test]
fn showcase492_compression() {
    let out = run_ore("showcase492.ore");
    assert!(out.contains("Compression Comparison:"));
    assert!(out.contains("RLE: 'a3b3c2d4a2b2'"));
    assert!(out.contains("Savings: 30%"));
    assert!(out.contains("Decompressed: 'aaabbbccddddaabb'"));
    assert!(out.contains("Matches original: true"));
}

#[test]
fn showcase493_cellular_automata() {
    let out = run_ore("showcase493.ore");
    assert!(out.contains("Cellular Automata Garden:"));
    assert!(out.contains("Rule 30"));
    assert!(out.contains("Rule 110"));
    assert!(out.contains("Game of Life (Blinker)"));
    assert!(out.contains("Turing complete"));
    assert!(out.contains("Cellular automata: Rule 30, Rule 110, Game of Life, growth, classification."));
}

#[test]
fn showcase494_database_indexing() {
    let out = run_ore("showcase494.ore");
    assert!(out.contains("Database Indexing:"));
    assert!(out.contains("Binary search for id=58: index=4"));
    assert!(out.contains("Found: 4 records"));
    assert!(out.contains("Lookup id=42: bucket=2, FOUND"));
    assert!(out.contains("Lookup id=99: bucket=3, NOT FOUND"));
}

#[test]
fn showcase495_task_scheduler() {
    let out = run_ore("showcase495.ore");
    assert!(out.contains("Task Scheduler:"));
    assert!(out.contains("Topological Sort"));
    assert!(out.contains("Critical Path"));
    assert!(out.contains("Parallel Execution"));
    assert!(out.contains("Task scheduler: dependencies, topological sort, timeline, critical path, parallelism."));
}

#[test]
fn showcase496_virtual_dom() {
    let out = run_ore("showcase496.ore");
    assert!(out.contains("Virtual DOM Diffing:"));
    assert!(out.contains("UPDATE node 1: 'Hello' -> 'Hi'"));
    assert!(out.contains("INSERT node 7: <li>Item 5"));
    assert!(out.contains("Total patches: 4"));
    assert!(out.contains("DELETE 'b'"));
}

#[test]
fn showcase497_fourier_transform() {
    let out = run_ore("showcase497.ore");
    assert!(out.contains("Fourier Transform Concepts:"));
    assert!(out.contains("X[0] = 10 + 0i"));
    assert!(out.contains("Reconstructed: [1, 2, 3, 4]"));
    assert!(out.contains("Dominant frequency bin: k=1"));
}

#[test]
fn showcase498_compiler_optimizations() {
    let out = run_ore("showcase498.ore");
    assert!(out.contains("Compiler Optimization Passes:"));
    assert!(out.contains("Constant Folding"));
    assert!(out.contains("Dead Code Elimination"));
    assert!(out.contains("Strength Reduction"));
    assert!(out.contains("REUSE r10"));
    assert!(out.contains("Total optimizations: 7"));
}

#[test]
fn showcase499_network_protocol() {
    let out = run_ore("showcase499.ore");
    assert!(out.contains("Network Protocol Simulation:"));
    assert!(out.contains("Connection ESTABLISHED"));
    assert!(out.contains("Send packet 3 [LOST]"));
    assert!(out.contains("All 10 packets delivered"));
    assert!(out.contains("CONGESTION!"));
    assert!(out.contains("Checksum: 244"));
}

#[test]
fn showcase500_grand_finale() {
    let out = run_ore("showcase500.ore");
    assert!(out.contains("SHOWCASE 500: GRAND FINALE"));
    assert!(out.contains("Fibonacci(0..9): 0, 1, 1, 2, 3, 5, 8, 13, 21, 34"));
    assert!(out.contains("Primes < 30: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29"));
    assert!(out.contains("GCD(48, 18) = 6"));
    assert!(out.contains("Circle: radius=5"));
    assert!(out.contains("Rect: 4x6, area=24"));
    assert!(out.contains("Sum: 55"));
    assert!(out.contains("FizzBuzz: 1 2 Fizz 4 Buzz"));
    assert!(out.contains("Collatz(27): 111 steps"));
    assert!(out.contains("det(A) = 0"));
    assert!(out.contains("Median: 55"));
    assert!(out.contains("SHOWCASE 500 COMPLETE"));
    assert!(out.contains("500 showcases and counting!"));
}

#[test]
fn showcase501_blockchain() {
    let out = run_ore("showcase501.ore");
    assert!(out.contains("Blockchain Simulation:"));
    assert!(out.contains("Block 0 (genesis): hash="));
    assert!(out.contains("Chain integrity: VALID"));
    assert!(out.contains("TAMPER DETECTED: hash mismatch!"));
    assert!(out.contains("Final: Alice=920, Bob=960, Carol=1055, Dave=1065"));
    assert!(out.contains("Blockchain: hashing, mining, validation, tamper detection, transactions."));
}

#[test]
fn showcase502_neural_network() {
    let out = run_ore("showcase502.ore");
    assert!(out.contains("Neural Network Forward Pass:"));
    assert!(out.contains("ReLU output: 22"));
    assert!(out.contains("Hidden (after ReLU): [2, 18, 0]"));
    assert!(out.contains("Output: [-16, 23]"));
    assert!(out.contains("MSE (x100): 600"));
    assert!(out.contains("Predicted class: cat (logit=12)"));
    assert!(out.contains("Neural network: neurons, layers, batch processing, loss, classification."));
}

#[test]
fn showcase503_audio_synthesis() {
    let out = run_ore("showcase503.ore");
    assert!(out.contains("Audio Synthesis Concepts:"));
    assert!(out.contains("Square wave: ####____####____####____####____"));
    assert!(out.contains("Mixed signal: 2211431022114310"));
    assert!(out.contains("A4: 440 Hz, period=100 samples"));
    assert!(out.contains("Audio synthesis: square, triangle, sawtooth, mixing, ADSR, note frequencies."));
}

#[test]
fn showcase504_constraint_solver() {
    let out = run_ore("showcase504.ore");
    assert!(out.contains("Constraint Solver:"));
    assert!(out.contains("Validation: PASS (all rows sum to 10)"));
    assert!(out.contains("Solution found:"));
    assert!(out.contains("Coloring: VALID"));
    assert!(out.contains("6-Queens: solution found"));
    assert!(out.contains("Constraint solver: sudoku, n-queens, graph coloring, backtracking."));
}

#[test]
fn showcase505_map_reduce() {
    let out = run_ore("showcase505.ore");
    assert!(out.contains("Map-Reduce Framework:"));
    assert!(out.contains("the: 6"));
    assert!(out.contains("'cat' -> [doc0, doc2, doc3] (3 docs)"));
    assert!(out.contains("eng: count=3, sum=245, avg=81, min=75, max=90"));
    assert!(out.contains("Grand total: 2909"));
    assert!(out.contains("Top 5: 44, 41, 39, 35, 33"));
    assert!(out.contains("Map-Reduce: word count, inverted index, group-by, pipeline, filtering."));
}

#[test]
fn showcase506_garbage_collector() {
    let out = run_ore("showcase506.ore");
    assert!(out.contains("Garbage Collector Simulation:"));
    assert!(out.contains("Alive: 3 objects, 37 bytes"));
    assert!(out.contains("Freed: 5 objects, 88 bytes"));
    assert!(out.contains("Sweep: freed 2 objects (30 bytes)"));
    assert!(out.contains("Fragments: 5"));
    assert!(out.contains("GC simulation: ref counting, mark-sweep, memory stats, fragmentation."));
}

#[test]
fn showcase507_chess_engine() {
    let out = run_ore("showcase507.ore");
    assert!(out.contains("Chess Engine Simulation:"));
    assert!(out.contains("White material: 490"));
    assert!(out.contains("Balance: 0 (0 = equal)"));
    assert!(out.contains("Total pawn moves: 16"));
    assert!(out.contains("Best move: e2e4 (score=3)"));
    assert!(out.contains("Phase: OPENING"));
    assert!(out.contains("Chess engine: board, material, position, moves, search, game phase."));
}

#[test]
fn showcase508_signal_processing() {
    let out = run_ore("showcase508.ore");
    assert!(out.contains("Signal Processing:"));
    assert!(out.contains("Convolution: 12, 20, 28, 32, 28, 20, 12"));
    assert!(out.contains("Peak at position 2 (value=45)"));
    assert!(out.contains("Edge positions: [3, 4, 7, 8]"));
    assert!(out.contains("Amplitude: 12"));
    assert!(out.contains("Downsample x2: 2, 6, 10, 14, 18, 22"));
    assert!(out.contains("Signal processing: moving average, convolution, correlation, edges, stats, downsampling."));
}

#[test]
fn showcase509_operating_system() {
    let out = run_ore("showcase509.ore");
    assert!(out.contains("Operating System Concepts:"));
    assert!(out.contains("PID 1: init [running] prio=0 mem=128KB"));
    assert!(out.contains("Total time: 18"));
    assert!(out.contains("VA 0 -> page=0, offset=0 -> PA 12"));
    assert!(out.contains("VA 10 -> page=2, offset=2 -> PAGE FAULT"));
    assert!(out.contains("Allocate 100KB: block 3 at 256"));
    assert!(out.contains("Total seek: 299 cylinders"));
    assert!(out.contains("OS concepts: processes, scheduling, paging, memory allocation, disk scheduling."));
}

#[test]
fn showcase510_compiler_backend() {
    let out = run_ore("showcase510.ore");
    assert!(out.contains("Compiler Backend Simulation:"));
    assert!(out.contains("t1 = 8, t2 = 6, t3 = 48, t4 = 55"));
    assert!(out.contains("MOV result, t4"));
    assert!(out.contains("Registers used: 4 of 4"));
    assert!(out.contains("Total: 10 -> 5 instructions (50% reduction)"));
    assert!(out.contains("Compiler backend: TAC, instruction selection, liveness, register allocation, assembly."));
}

#[test]
fn showcase511_movie_recommendation() {
    let out = run_ore("showcase511.ore");
    assert!(out.contains("Movie Recommendation Engine:"));
    assert!(out.contains("Alice <-> Eve: similarity = 32"));
    assert!(out.contains("Most similar user: Eve (score: 32)"));
    assert!(out.contains("Recommend: Casablanca (rated 3 by Eve)"));
    assert!(out.contains("Recommendation engine: ratings, similarity, collaborative filtering, popularity."));
}

#[test]
fn showcase512_weather_forecast() {
    let out = run_ore("showcase512.ore");
    assert!(out.contains("Weather Forecast Model:"));
    assert!(out.contains("Average: 72F, Min: 68F, Max: 77F"));
    assert!(out.contains("Forecast day 15: 74F"));
    assert!(out.contains("Trend: stable"));
    assert!(out.contains("Total anomalies: 3 of 14 days"));
    assert!(out.contains("Weather forecast: historical data, moving average, trend detection, conditions."));
}

#[test]
fn showcase513_sql_query() {
    let out = run_ore("showcase513.ore");
    assert!(out.contains("SQL Query Parser and Executor:"));
    assert!(out.contains("Rows returned: 3"));
    assert!(out.contains("Eve: 95"));
    assert!(out.contains("eng: count=3, avg_salary=90"));
    assert!(out.contains("Alice -> Building A"));
    assert!(out.contains("Rows above average: 3"));
    assert!(out.contains("SQL executor: select, where, order by, group by, join, subquery."));
}

#[test]
fn showcase514_peg_solitaire() {
    let out = run_ore("showcase514.ore");
    assert!(out.contains("Conway's Soldiers (Peg Solitaire):"));
    assert!(out.contains("Pegs: 32, Empty: 1"));
    assert!(out.contains("Total valid moves: 4"));
    assert!(out.contains("Move 1: (1,3) -> (3,3) via down. Pegs left: 31"));
    assert!(out.contains("Remaining pegs: 26"));
    assert!(out.contains("Peg solitaire: board setup, move validation, jump execution, scoring."));
}

#[test]
fn showcase515_elevator() {
    let out = run_ore("showcase515.ore");
    assert!(out.contains("Elevator Simulation:"));
    assert!(out.contains("Building: 10 floors, 2 elevators"));
    assert!(out.contains("Total requests: 8"));
    assert!(out.contains("Total travel distance: 43 floors"));
    assert!(out.contains("ElevA: 5 trips, final floor 2"));
    assert!(out.contains("Elevator simulation: dispatch, statistics, peak hour, energy tracking."));
}

#[test]
fn showcase516_cellular_growth() {
    let out = run_ore("showcase516.ore");
    assert!(out.contains("Cellular Growth Simulation:"));
    assert!(out.contains("Alive cells: 5"));
    assert!(out.contains("Gen 1: 5 alive cells"));
    assert!(out.contains("Gen 4: 5 alive cells"));
    assert!(out.contains("Top-Left: 5 cells"));
    assert!(out.contains("Cellular growth: Game of Life, evolution, population tracking, density analysis."));
}

#[test]
fn showcase517_debugger() {
    let out = run_ore("showcase517.ore");
    assert!(out.contains("Simple Debugger Simulation:"));
    assert!(out.contains("Total breakpoints: 3"));
    assert!(out.contains("Step 3: z := x + y [BREAKPOINT HIT]"));
    assert!(out.contains("z = 30"));
    assert!(out.contains("result = 55"));
    assert!(out.contains("Stack depth: 4"));
    assert!(out.contains("Debugger: breakpoints, step execution, variable inspection, call stack, watchpoints."));
}

#[test]
fn showcase518_recipe_scaling() {
    let out = run_ore("showcase518.ore");
    assert!(out.contains("Recipe Ingredient Scaling:"));
    assert!(out.contains("flour: 200 -> 500 g"));
    assert!(out.contains("Total: 1510 cal (377 cal per serving)"));
    assert!(out.contains("300ml milk = 5/4 cups"));
    assert!(out.contains("Oven: 350F = 176C"));
    assert!(out.contains("Recipe scaling: ingredients, scale up/down, shopping list, nutrition, conversions."));
}

#[test]
fn showcase519_version_control() {
    let out = run_ore("showcase519.ore");
    assert!(out.contains("Simple Version Control:"));
    assert!(out.contains("Changes detected: 2"));
    assert!(out.contains("a1b2c3 | alice | Initial commit (3 files)"));
    assert!(out.contains("Merge conflicts resolved: 1"));
    assert!(out.contains("Total file changes: 15"));
    assert!(out.contains("Version control: diff, commits, branching, merge, blame, statistics."));
}

#[test]
fn showcase520_neural_network() {
    let out = run_ore("showcase520.ore");
    assert!(out.contains("Neural Network Backpropagation:"));
    assert!(out.contains("Total parameters: 26"));
    assert!(out.contains("Hidden (after ReLU): [12, 27, 17, 33]"));
    assert!(out.contains("Total loss (MSE): 6561"));
    assert!(out.contains("dL/dout0 = -162"));
    assert!(out.contains("Loss reduction: 99%"));
    assert!(out.contains("Neural network: forward pass, loss, backpropagation, weight update, training."));
}

#[test]
fn showcase521_parser_combinator() {
    let out = run_ore("showcase521.ore");
    assert!(out.contains("Parser Combinator Library:"));
    assert!(out.contains("[0] NUM(3)"));
    assert!(out.contains("[3] OP(*)"));
    assert!(out.contains("Result: 287"));
    assert!(out.contains("Total AST nodes: 7"));
    assert!(out.contains("Parser tests: 5/5 passed"));
    assert!(out.contains("Parser combinators: tokenizer, recursive descent, grammar rules, AST analysis."));
}

#[test]
fn showcase522_poker_hand() {
    let out = run_ore("showcase522.ore");
    assert!(out.contains("Poker Hand Evaluator:"));
    assert!(out.contains("Deck: 52 cards (4 suits x 13 ranks)"));
    assert!(out.contains("Hand 2: 2H 7H 9H 5H QH -> Flush (Hearts)"));
    assert!(out.contains("Full House (Ks over 4s)"));
    assert!(out.contains("Best hand: Hand 4 - Full House (rank 7)"));
    assert!(out.contains("Total pot: 345"));
    assert!(out.contains("Poker: deck, hand evaluation, ranking, comparison, probability, payouts."));
}

#[test]
fn showcase523_jit_compiler() {
    let out = run_ore("showcase523.ore");
    assert!(out.contains("JIT Compiler Simulation:"));
    assert!(out.contains("Bytecode (14 instructions):"));
    assert!(out.contains("Interpreter result: sum = 10"));
    assert!(out.contains("Hot instructions: 10/14"));
    assert!(out.contains("Native code (7 instructions):"));
    assert!(out.contains("JIT is faster by 14 cycles"));
    assert!(out.contains("JIT compiler: bytecode, interpreter, hot paths, native codegen, performance."));
}

#[test]
fn showcase524_ant_colony() {
    let out = run_ore("showcase524.ore");
    assert!(out.contains("Ant Colony Optimization:"));
    assert!(out.contains("Cities: 5"));
    assert!(out.contains("Best: Ant 1 with length 85"));
    assert!(out.contains("Evaporation rate: 10%"));
    assert!(out.contains("Improvement: 25 (25%)"));
    assert!(out.contains("Ant colony: graph, pheromones, tours, evaporation, convergence."));
}

#[test]
fn showcase525_http_routing() {
    let out = run_ore("showcase525.ore");
    assert!(out.contains("HTTP Server Routing Simulation:"));
    assert!(out.contains("GET /users -> 200 (list_users)"));
    assert!(out.contains("PATCH /users -> 405 (405 Method Not Allowed)"));
    assert!(out.contains("Total processing time: 80us"));
    assert!(out.contains("Blocked clients: 2/5"));
    assert!(out.contains("HTTP routing: routes, matching, middleware, rate limiting, logging."));
}

#[test]
fn showcase526_graph_database() {
    let out = run_ore("showcase526.ore");
    assert!(out.contains("Graph Database Query Engine:"));
    assert!(out.contains("Nodes: 8"));
    assert!(out.contains("Alice -> watched -> Matrix"));
    assert!(out.contains("Results: 7 rows"));
    assert!(out.contains("Recommend: Interstellar (watched by Bob)"));
    assert!(out.contains("Graph density: 15%"));
    assert!(out.contains("Graph database: nodes, edges, pattern matching, aggregation, paths, recommendations."));
}

#[test]
fn showcase527_decision_tree() {
    let out = run_ore("showcase527.ore");
    assert!(out.contains("Decision Tree Classifier:"));
    assert!(out.contains("Yes: 6, No: 4"));
    assert!(out.contains("Best split: outlook (gain=0.24)"));
    assert!(out.contains("|-- Overcast: Yes"));
    assert!(out.contains("Accuracy: 5/5 = 100%"));
    assert!(out.contains("Total nodes: 8"));
    assert!(out.contains("Decision tree: entropy, information gain, splitting, prediction, accuracy."));
}

#[test]
fn showcase528_circuit_simulator() {
    let out = run_ore("showcase528.ore");
    assert!(out.contains("Digital Circuit Simulator:"));
    assert!(out.contains("1 AND 1 = 1"));
    assert!(out.contains("1 XOR 0 = 1"));
    assert!(out.contains("15      1111"));
    assert!(out.contains("[1, 0, 1, 1]"));
    assert!(out.contains("Max frequency: 100MHz"));
    assert!(out.contains("Circuit simulator: gates, SR latch, D flip-flop, counter, shift register, timing."));
}

#[test]
fn showcase529_garbage_collection() {
    let out = run_ore("showcase529.ore");
    assert!(out.contains("Garbage Collection Strategies:"));
    assert!(out.contains("Total blocks: 16"));
    assert!(out.contains("Garbage blocks: 7"));
    assert!(out.contains("Blocks freed by RC: 7"));
    assert!(out.contains("Swept 7 garbage blocks"));
    assert!(out.contains("Fragments after compaction: 1"));
    assert!(out.contains("GC strategies: reference counting, mark-sweep, compaction, comparison."));
}

#[test]
fn showcase530_crypto_mining() {
    let out = run_ore("showcase530.ore");
    assert!(out.contains("Cryptocurrency Mining Simulation:"));
    assert!(out.contains("Chain integrity: VALID"));
    assert!(out.contains("MinerC: 350 coins (35%)"));
    assert!(out.contains("Total fees: 13 coins"));
    assert!(out.contains("Miner earns: 50 (reward) + 11 (fees) = 61"));
    assert!(out.contains("Total coins minted: 300"));
    assert!(out.contains("Crypto mining: blockchain, proof-of-work, pool, mempool, difficulty, stats."));
}

#[test]
fn showcase531_event_driven() {
    let out = run_ore("showcase531.ore");
    assert!(out.contains("Event-Driven Architecture Simulation:"));
    assert!(out.contains("user.created -> 2 handlers: [EmailService, LogService]"));
    assert!(out.contains("Total dispatches: 11"));
    assert!(out.contains("P1: critical.error"));
    assert!(out.contains("Dead letters: 2/3"));
    assert!(out.contains("Event-driven: bus, subscribers, dispatch, priority queue, dead letters, sourcing."));
}

#[test]
fn showcase532_search_engine() {
    let out = run_ore("showcase532.ore");
    assert!(out.contains("Simple Search Engine:"));
    assert!(out.contains("'programming' -> 3 docs"));
    assert!(out.contains("Found 3 results"));
    assert!(out.contains("IDF('compiler') = 25"));
    assert!(out.contains("Unique terms indexed: 8"));
    assert!(out.contains("Search engine: inverted index, TF-IDF, ranking, multi-term queries, statistics."));
}

#[test]
fn showcase533_packet_network() {
    let out = run_ore("showcase533.ore");
    assert!(out.contains("Packet Switching Network Simulation:"));
    assert!(out.contains("A -- B (10ms)"));
    assert!(out.contains("Fragments needed: 5"));
    assert!(out.contains("Packet 0: A->F size=512B latency=33ms"));
    assert!(out.contains("Congested links: 3/8"));
    assert!(out.contains("Packet network: topology, routing, fragmentation, congestion, statistics."));
}

#[test]
fn showcase534_theorem_prover() {
    let out = run_ore("showcase534.ore");
    assert!(out.contains("Simple Theorem Prover (Propositional Logic):"));
    assert!(out.contains("1 AND 1 = 1"));
    assert!(out.contains("Result: TAUTOLOGY"));
    assert!(out.contains("NOT A TAUTOLOGY (contradiction)"));
    assert!(out.contains("De Morgan 1: VERIFIED"));
    assert!(out.contains("Total theorems proved: 4"));
    assert!(out.contains("Theorem prover: truth tables, evaluation, tautologies, De Morgan, inference rules."));
}

#[test]
fn showcase535_image_dithering() {
    let out = run_ore("showcase535.ore");
    assert!(out.contains("Image Dithering (Floyd-Steinberg on Number Grid):"));
    assert!(out.contains("White pixels: 8, Black pixels: 16"));
    assert!(out.contains("Floyd-Steinberg result (rows 0-1):"));
    assert!(out.contains("Image size: 4x6 = 24 pixels"));
    assert!(out.contains("Image dithering: threshold, Floyd-Steinberg, error diffusion, histogram, analysis."));
}

#[test]
fn showcase536_format_parser() {
    let out = run_ore("showcase536.ore");
    assert!(out.contains("Simple File Format Parser:"));
    assert!(out.contains("Record 1: name=Alice, age=30, city=NYC"));
    assert!(out.contains("Average age: 29"));
    assert!(out.contains("database.host = localhost"));
    assert!(out.contains("Youngest: Bob (age 25)"));
    assert!(out.contains("Total items parsed: 10"));
    assert!(out.contains("Format parser: CSV, INI, validation, statistics, generation."));
}

#[test]
fn showcase537_bin_packing() {
    let out = run_ore("showcase537.ore");
    assert!(out.contains("Resource Allocation (Bin Packing):"));
    assert!(out.contains("Total demand: 225 units"));
    assert!(out.contains("Theoretical minimum bins: 3"));
    assert!(out.contains("Bin 0: [App-A, App-B, App-C] (100/100)"));
    assert!(out.contains("FFD is at least as good as FF"));
    assert!(out.contains("Bin packing: first fit, first fit decreasing, utilization, waste analysis."));
}

#[test]
fn showcase538_game_physics() {
    let out = run_ore("showcase538.ore");
    assert!(out.contains("Simple Game Physics (2D Platformer):"));
    assert!(out.contains("Jump! vy=-20, vx=5"));
    assert!(out.contains("Standing on platform 1"));
    assert!(out.contains("Total collisions: 2"));
    assert!(out.contains("Game physics: gravity, jumping, platforms, AABB collision, projectiles."));
}

#[test]
fn showcase539_automata_validation() {
    let out = run_ore("showcase539.ore");
    assert!(out.contains("Automata-Based String Validation:"));
    assert!(out.contains("'110' (=6): ACCEPT (mod 3 = 0)"));
    assert!(out.contains("'111' (=7): REJECT (mod 3 = 1)"));
    assert!(out.contains("'0011': MATCH"));
    assert!(out.contains("Valid: 5/7"));
    assert!(out.contains("Total strings validated: 24"));
    assert!(out.contains("Automata: DFA, pattern matching, integer validation, identifier validation."));
}

#[test]
fn showcase540_code_generator() {
    let out = run_ore("showcase540.ore");
    assert!(out.contains("Simple Machine Code Generator:"));
    assert!(out.contains("AST: ADD(3, MUL(4, 2))"));
    assert!(out.contains("Registers used: 4/8"));
    assert!(out.contains("Execution result: sum = 10"));
    assert!(out.contains("Total instructions generated: 28"));
    assert!(out.contains("Code generator: instructions, registers, if-else, loops, statistics."));
}

#[test]
fn showcase541_lz77_compression() {
    let out = run_ore("showcase541.ore");
    assert!(out.contains("Lempel-Ziv (LZ77) Compression Concepts:"));
    assert!(out.contains("Match: YES - lossless compression verified"));
    assert!(out.contains("Compression ratio: 97%"));
    assert!(out.contains("Literals: 3"));
    assert!(out.contains("Matches: 2"));
    assert!(out.contains("LZ77 compression: sliding window, match finding, tokens, decoding, dictionary."));
}

#[test]
fn showcase542_cpu_pipeline() {
    let out = run_ore("showcase542.ore");
    assert!(out.contains("Simple CPU Pipeline Simulator:"));
    assert!(out.contains("Total cycles (ideal): 10"));
    assert!(out.contains("RAW: I1 -> I2 on R1"));
    assert!(out.contains("Cycles saved: 4"));
    assert!(out.contains("Accuracy: 75%"));
    assert!(out.contains("CPU pipeline: stages, hazards, stalls, forwarding, branch prediction, IPC."));
}

#[test]
fn showcase543_regex_features() {
    let out = run_ore("showcase543.ore");
    assert!(out.contains("Regular Expression Features Showcase:"));
    assert!(out.contains("Letters: 9, Digits: 3"));
    assert!(out.contains("Found 6 digit(s) at positions: 3, 4, 5, 9, 10, 11"));
    assert!(out.contains("'abc' -> REJECT"));
    assert!(out.contains("Regex features: matching, NFA, quantifiers, groups, alternation, anchors."));
}

#[test]
fn showcase544_fractal_generation() {
    let out = run_ore("showcase544.ore");
    assert!(out.contains("Fractal Generation (Sierpinski, Koch):"));
    assert!(out.contains("Fractal dimension: log(3)/log(2) ~ 1.585"));
    assert!(out.contains("(0,0): in set (no escape in 20 iterations)"));
    assert!(out.contains("(1,0): escaped at iteration 2"));
    assert!(out.contains("Fractals: Sierpinski triangle, Koch snowflake, Cantor set, Mandelbrot escape."));
}

#[test]
fn showcase545_lisp_interpreter() {
    let out = run_ore("showcase545.ore");
    assert!(out.contains("Simple LISP Interpreter:"));
    assert!(out.contains("(* (+ 1 2) (- 5 3)) = 6"));
    assert!(out.contains("(fact 6) = 720"));
    assert!(out.contains("(map square list) = (1 4 9 16 25)"));
    assert!(out.contains("LISP interpreter: tokenizer, parser, evaluator, environment, lambda, lists."));
}

#[test]
fn showcase546_cache_policies() {
    let out = run_ore("showcase546.ore");
    assert!(out.contains("Cache Replacement Policies (LRU, FIFO, LFU):"));
    assert!(out.contains("FIFO: 3 hits, 9 misses, rate=25%"));
    assert!(out.contains("LRU: 2 hits, 10 misses, rate=16%"));
    assert!(out.contains("Working set exceeds cache: thrashing likely"));
    assert!(out.contains("Cache policies: FIFO, LRU, LFU comparison, working set analysis."));
}

#[test]
fn showcase547_network_topology() {
    let out = run_ore("showcase547.ore");
    assert!(out.contains("Network Topology Analysis:"));
    assert!(out.contains("Hub node: Switch-C (degree 4)"));
    assert!(out.contains("Switch-D: 7ms"));
    assert!(out.contains("Reachable from Router-A: 6/6"));
    assert!(out.contains("Network topology: BFS, shortest paths, degrees, metrics, connectivity."));
}

#[test]
fn showcase548_spreadsheet_engine() {
    let out = run_ore("showcase548.ore");
    assert!(out.contains("Simple Spreadsheet Formula Engine:"));
    assert!(out.contains("SUM(Row 1): 100"));
    assert!(out.contains("Grand total: 270"));
    assert!(out.contains("COUNTIF(>20): 5 cells"));
    assert!(out.contains("Spreadsheet engine: SUM, AVG, MIN, MAX, IF, COUNTIF, SUMIF, cell references."));
}

#[test]
fn showcase549_constraint_programming() {
    let out = run_ore("showcase549.ore");
    assert!(out.contains("Constraint Logic Programming Concepts:"));
    assert!(out.contains("Solution: x=3, y=7"));
    assert!(out.contains("Total solutions: 2"));
    assert!(out.contains("All constraints satisfied: YES"));
    assert!(out.contains("Constraint programming: CSP, N-queens, magic square, graph coloring, sudoku."));
}

#[test]
fn showcase550_ray_marching() {
    let out = run_ore("showcase550.ore");
    assert!(out.contains("Ray Marching (Signed Distance Functions):"));
    assert!(out.contains("Union(50, 80) = 50"));
    assert!(out.contains("Hit at distance 400 in 2 steps"));
    assert!(out.contains("Pixels hit: 3/9"));
    assert!(out.contains("Ray marching: SDF primitives, boolean ops, marching, rendering, shading."));
}

#[test]
fn showcase551_wavelet_transform() {
    let out = run_ore("showcase551.ore");
    assert!(out.contains("Wavelet Transform Concepts:"));
    assert!(out.contains("Level 1 averages: 5, 5, 4, 4"));
    assert!(out.contains("Level 1 details: 1, 3, -1, 3"));
    assert!(out.contains("Reconstructed from L1: 6, 4, 8, 2, 3, 5, 7, 1"));
    assert!(out.contains("Match: 8/8 values correct"));
    assert!(out.contains("Signal energy: 204"));
    assert!(out.contains("Wavelet transform: Haar decomposition, coefficients, reconstruction, thresholding, energy."));
}

#[test]
fn showcase552_git_concepts() {
    let out = run_ore("showcase552.ore");
    assert!(out.contains("Simple Git Concepts:"));
    assert!(out.contains("a1b2c3 Initial commit (root)"));
    assert!(out.contains("main -> m3n4o5 (Add tests)"));
    assert!(out.contains("Common ancestor: g7h8i9 (Add feature X)"));
    assert!(out.contains("Stats: +1 -0"));
    assert!(out.contains("Total: 2 staged, 2 unstaged"));
    assert!(out.contains("Git concepts: commit graph, branching, merge, diff, rebase, staging."));
}

#[test]
fn showcase553_quine_self_referential() {
    let out = run_ore("showcase553.ore");
    assert!(out.contains("Quine and Self-Referential Programs:"));
    assert!(out.contains("Total fixed points: 4"));
    assert!(out.contains("Original: hello"));
    assert!(out.contains("Encoded:  uryyb"));
    assert!(out.contains("Self-inverse: YES (ROT13(ROT13(x)) = x)"));
    assert!(out.contains("Anti-diagonal: 01101"));
    assert!(out.contains("Quines: self-reference, fixed points, recursion theorem, self-inverse, diagonal."));
}

#[test]
fn showcase554_music_sequencer() {
    let out = run_ore("showcase554.ore");
    assert!(out.contains("Simple Music Sequencer:"));
    assert!(out.contains("A4: 440 Hz"));
    assert!(out.contains("Total beats: 32"));
    assert!(out.contains("C major: C E G"));
    assert!(out.contains("Kick:  X...X...X...X..."));
    assert!(out.contains("C major scale: C D E F G A B C"));
    assert!(out.contains("Total bars: 52"));
    assert!(out.contains("Music sequencer: notes, melody, chords, drums, scales, song structure."));
}

#[test]
fn showcase555_reed_solomon() {
    let out = run_ore("showcase555.ore");
    assert!(out.contains("Reed-Solomon Error Correction Concepts:"));
    assert!(out.contains("3 * 5 = 1 (mod 7)"));
    assert!(out.contains("Codeword: 1, 4, 6, 0, 0, 6"));
    assert!(out.contains("Error detected: YES"));
    assert!(out.contains("Correction verified: YES"));
    assert!(out.contains("Overhead: 50%"));
    assert!(out.contains("Reed-Solomon: Galois fields, polynomial encoding, error detection, correction."));
}

#[test]
fn showcase556_packet_filter() {
    let out = run_ore("showcase556.ore");
    assert!(out.contains("Simple Packet Filter / Firewall:"));
    assert!(out.contains("Rule 1: ALLOW TCP port 80 (HTTP)"));
    assert!(out.contains("Allowed: 3, Denied: 4"));
    assert!(out.contains("BLOCKED: 172.16.0.1 -> 192.168.1.10:22 (TCP)"));
    assert!(out.contains("Block rate: 57%"));
    assert!(out.contains("Packet filter: rules, filtering, connection tracking, rate limiting, logging."));
}

#[test]
fn showcase557_huffman_adaptive() {
    let out = run_ore("showcase557.ore");
    assert!(out.contains("Huffman Adaptive Coding:"));
    assert!(out.contains("Total characters: 100"));
    assert!(out.contains("Tree complete! Root weight: 100"));
    assert!(out.contains("Huffman total bits: 224"));
    assert!(out.contains("Compression ratio: 74%"));
    assert!(out.contains("Average bits/symbol: 2.24"));
    assert!(out.contains("Huffman coding: frequency analysis, tree construction, encoding, adaptive coding."));
}

#[test]
fn showcase558_os_scheduler() {
    let out = run_ore("showcase558.ore");
    assert!(out.contains("Simple Operating System Scheduler:"));
    assert!(out.contains("PID=1 init priority=0 burst=2 arrival=0"));
    assert!(out.contains("Average wait time: 7"));
    assert!(out.contains("Time quantum: 3"));
    assert!(out.contains("Best for this workload: SJF (avg wait = 6)"));
    assert!(out.contains("OS scheduler: FCFS, SJF, Round Robin, Priority scheduling, comparison."));
}

#[test]
fn showcase559_automata_theory() {
    let out = run_ore("showcase559.ore");
    assert!(out.contains("Automata Theory Toolkit:"));
    assert!(out.contains("101: ACCEPT (final state: q2)"));
    assert!(out.contains("100: REJECT (final state: q1)"));
    assert!(out.contains("((())): ACCEPT"));
    assert!(out.contains("Output tape: 1100 (=12)"));
    assert!(out.contains("Increment: 11 -> 12"));
    assert!(out.contains("Automata: DFA, NFA, PDA, Turing machine, Chomsky hierarchy."));
}

#[test]
fn showcase560_grand_algorithm() {
    let out = run_ore("showcase560.ore");
    assert!(out.contains("Grand Algorithm Showcase:"));
    assert!(out.contains("Found at index: 5 in 3 steps"));
    assert!(out.contains("After:  11, 12, 22, 25, 34, 45, 64, 90"));
    assert!(out.contains("gcd(48, 18) = 6"));
    assert!(out.contains("Count: 15"));
    assert!(out.contains("F(20) = 6765"));
    assert!(out.contains("Found: nums[0] + nums[1] = 2 + 7 = 9"));
    assert!(out.contains("Encoded: 3A2B4C1A"));
    assert!(out.contains("Grand showcase: binary search, sorting, GCD, primes, Fibonacci, two sum, RLE."));
}

#[test]
fn showcase561_raft_leader_election() {
    let out = run_ore("showcase561.ore");
    assert!(out.contains("Raft-like Leader Election:"));
    assert!(out.contains("Node 3 times out first (120ms)"));
    assert!(out.contains("Node 3 becomes candidate, term=1"));
    assert!(out.contains("Majority reached!"));
    assert!(out.contains("Node 3 is now LEADER (term 1)"));
    assert!(out.contains("Entry 'SET x=1' committed (5/5 acks)"));
    assert!(out.contains("Votes received: 5/5"));
    assert!(out.contains("Raft consensus: election timeout, vote collection, leader establishment, log replication."));
}

#[test]
fn showcase562_type_inference() {
    let out = run_ore("showcase562.ore");
    assert!(out.contains("Simple Type Inference Engine:"));
    assert!(out.contains("E0: 42 => Int (literal rule)"));
    assert!(out.contains("E3: sum > 0 => Bool (comparison rule)"));
    assert!(out.contains("E6: len(x) => Int (function signature)"));
    assert!(out.contains("Unify b ~ a => b = Int"));
    assert!(out.contains("Types inferred: 7/7"));
    assert!(out.contains("Type inference: literals, operators, unification, environment lookup."));
}

#[test]
fn showcase563_sparse_vectors() {
    let out = run_ore("showcase563.ore");
    assert!(out.contains("Sparse Vector Operations:"));
    assert!(out.contains("Dot product: 19"));
    assert!(out.contains("A + B (7 non-zeros)"));
    assert!(out.contains("||A||^2 = 87"));
    assert!(out.contains("||B||^2 = 66"));
    assert!(out.contains("Sparse vectors: dot product, addition, norms, scalar multiply, sparsity analysis."));
}

#[test]
fn showcase564_garbage_collector() {
    let out = run_ore("showcase564.ore");
    assert!(out.contains("Simple Garbage Collector with Generations:"));
    assert!(out.contains("Total allocated: 60 bytes in 8 objects"));
    assert!(out.contains("Marked: 7/8"));
    assert!(out.contains("Sweep obj[7] (freed 4 bytes)"));
    assert!(out.contains("Promoted: 7 objects"));
    assert!(out.contains("Live objects: 7"));
    assert!(out.contains("GC simulation: mark-sweep, generational collection, promotion, reference tracing."));
}

#[test]
fn showcase565_protobuf() {
    let out = run_ore("showcase565.ore");
    assert!(out.contains("Protocol Buffers Concepts:"));
    assert!(out.contains("Encoded bytes: [172, 2]"));
    assert!(out.contains("Decoded: 300"));
    assert!(out.contains("Total message size: 23 bytes"));
    assert!(out.contains("Space savings: 60%"));
    assert!(out.contains("change type: UNSAFE"));
    assert!(out.contains("Protobuf concepts: schema, wire types, varint encoding, serialization, evolution."));
}

#[test]
fn showcase566_os_boot() {
    let out = run_ore("showcase566.ore");
    assert!(out.contains("Simple OS Boot Sequence:"));
    assert!(out.contains("POST complete: 5/5 passed"));
    assert!(out.contains("Total memory: 16384 KB (16 MB)"));
    assert!(out.contains("Kernel init: 22ms"));
    assert!(out.contains("Drivers loaded: 5/5"));
    assert!(out.contains("Starting ssh ... PID 102"));
    assert!(out.contains("OS boot: POST, memory detection, bootloader, kernel init, drivers, filesystem, services."));
}

#[test]
fn showcase567_query_optimization() {
    let out = run_ore("showcase567.ore");
    assert!(out.contains("Query Optimization:"));
    assert!(out.contains(">> Hash Join: cost=20 << SELECTED"));
    assert!(out.contains("Speedup: 25000x"));
    assert!(out.contains("Cost reduction: 500000 -> 20"));
    assert!(out.contains("Query optimization: table stats, plan enumeration, cost estimation, index selection."));
}

#[test]
fn showcase568_finite_element() {
    let out = run_ore("showcase568.ore");
    assert!(out.contains("Finite Element Method (1D):"));
    assert!(out.contains("Elements: 5"));
    assert!(out.contains("Diagonal: 5, 10, 10, 10, 10, 5"));
    assert!(out.contains("Max error: 0 (exact for linear problem)"));
    assert!(out.contains("DOF (degrees of freedom): 4"));
    assert!(out.contains("FEM 1D: mesh generation, stiffness assembly, load vector, boundary conditions, solution."));
}

#[test]
fn showcase569_smart_contract() {
    let out = run_ore("showcase569.ore");
    assert!(out.contains("Blockchain Smart Contract:"));
    assert!(out.contains("TX1: Alice deposits 200 tokens"));
    assert!(out.contains("Vault total: 450 tokens"));
    assert!(out.contains("TX4: Alice withdraws 50 tokens"));
    assert!(out.contains("TX REVERTED: Bob tried to withdraw 999 (only has 150)"));
    assert!(out.contains("Total supply: 1800"));
    assert!(out.contains("Smart contract: token vault, deposits, withdrawals, state queries, transaction log."));
}

#[test]
fn showcase570_error_recovery() {
    let out = run_ore("showcase570.ore");
    assert!(out.contains("Compiler Error Recovery Strategies:"));
    assert!(out.contains("ERROR: unexpected '@' -- skipping"));
    assert!(out.contains("Syncing to next ';' ..."));
    assert!(out.contains("Cascading errors prevented: 2"));
    assert!(out.contains("... too many errors, aborting compilation"));
    assert!(out.contains("Reported: 10, suppressed: 2"));
    assert!(out.contains("Error recovery: panic mode, error productions, cascade prevention, error limiting."));
}

#[test]
fn showcase571_dns_zone_parser() {
    let out = run_ore("showcase571.ore");
    assert!(out.contains("DNS Zone File Parser:"));
    assert!(out.contains("@  3600  IN  SOA  ns1.example.com admin.example.com"));
    assert!(out.contains("Query: www A -> 192.168.1.1 (TTL: 300)"));
    assert!(out.contains("Query: unknown A -> NXDOMAIN"));
    assert!(out.contains("Final: ftp.example.com = 192.168.1.1"));
    assert!(out.contains("Zone valid: true"));
    assert!(out.contains("DNS zone parser: record types, TTL analysis, query resolution, CNAME chains, validation."));
}

#[test]
fn showcase572_kalman_filter() {
    let out = run_ore("showcase572.ore");
    assert!(out.contains("Kalman Filter (1D State Estimation):"));
    assert!(out.contains("t=0: gain=95%, estimate=45, error=50"));
    assert!(out.contains("Initial gain: 95%"));
    assert!(out.contains("Final gain: 15%"));
    assert!(out.contains("Trend: decreasing (filter converging)"));
    assert!(out.contains("Filter reduces noise spread"));
    assert!(out.contains("Kalman filter: predict-update cycle, gain convergence, noise reduction, state estimation."));
}

#[test]
fn showcase573_load_balancer() {
    let out = run_ore("showcase573.ore");
    assert!(out.contains("Load Balancer Simulation:"));
    assert!(out.contains("Request 1 -> srv-a"));
    assert!(out.contains("Total weight: 10"));
    assert!(out.contains("srv-b: UNHEALTHY (removed from pool)"));
    assert!(out.contains("Active servers: 3/4"));
    assert!(out.contains("srv-b fully drained"));
    assert!(out.contains("Load balancer: round robin, weighted, least connections, health checks, draining."));
}

#[test]
fn showcase574_bplus_tree() {
    let out = run_ore("showcase574.ore");
    assert!(out.contains("B+ Tree Concepts:"));
    assert!(out.contains("Search(12): FOUND"));
    assert!(out.contains("Search(99): NOT FOUND"));
    assert!(out.contains("Range query: [8, 25]"));
    assert!(out.contains("Results (6 keys):"));
    assert!(out.contains("Node still has enough keys (>= 2)"));
    assert!(out.contains("B+ tree: ordered insertion, point queries, range scans, leaf chains, deletion."));
}

#[test]
fn showcase575_pubsub() {
    let out = run_ore("showcase575.ore");
    assert!(out.contains("Pub/Sub Messaging System:"));
    assert!(out.contains("dashboard subscribed to 'events'"));
    assert!(out.contains("Publish to 'events': user.login"));
    assert!(out.contains("Deliver 'user.login' -> dashboard"));
    assert!(out.contains("DLQ size: 2"));
    assert!(out.contains("Total deliveries: 13"));
    assert!(out.contains("Pub/sub: topics, subscribers, publishing, delivery, filtering, dead letter queue."));
}

#[test]
fn showcase576_evolutionary_strategies() {
    let out = run_ore("showcase576.ore");
    assert!(out.contains("Evolutionary Strategies Optimization:"));
    assert!(out.contains("Individual 0: x=10, f(x)=1024"));
    assert!(out.contains("Tournament 0 vs 1: 30 wins (f=144)"));
    assert!(out.contains("Converged to optimal: x=42"));
    assert!(out.contains("Elite carry: 30 (f=144)"));
    assert!(out.contains("Evolutionary strategies: population, tournament selection, mutation, elite carry, sigma adaptation."));
}

#[test]
fn showcase577_tcp_state_machine() {
    let out = run_ore("showcase577.ore");
    assert!(out.contains("TCP State Machine:"));
    assert!(out.contains("Connection established!"));
    assert!(out.contains("Client -> [1000] 'Hello' -> Server"));
    assert!(out.contains("Server -> [ACK 1005] -> Client"));
    assert!(out.contains("Client: TIME_WAIT"));
    assert!(out.contains("Connection closed!"));
    assert!(out.contains("CLOSED --[active open]--> SYN_SENT"));
    assert!(out.contains("TCP state machine: handshake, data transfer, teardown, transitions, flow control."));
}

#[test]
fn showcase578_force_directed_layout() {
    let out = run_ore("showcase578.ore");
    assert!(out.contains("Force-Directed Graph Layout:"));
    assert!(out.contains("A -- B"));
    assert!(out.contains("A: repulsion"));
    assert!(out.contains("A: attraction"));
    assert!(out.contains("Energy decreased: layout stabilizing"));
    assert!(out.contains("Bounding box:"));
    assert!(out.contains("Force-directed layout: repulsion, attraction, cooling, convergence, quality metrics."));
}

#[test]
fn showcase579_compression_pipeline() {
    let out = run_ore("showcase579.ore");
    assert!(out.contains("File Compression Pipeline:"));
    assert!(out.contains("Input: AAABBCCCCADD"));
    assert!(out.contains("Encoded size: 10 (from 12)"));
    assert!(out.contains("Total bits: 22"));
    assert!(out.contains("Savings: 78%"));
    assert!(out.contains("match: (3,3) (2 bytes)"));
    assert!(out.contains("Final compression ratio: 51%"));
    assert!(out.contains("Compression pipeline: RLE, frequency analysis, Huffman coding, LZ77, dictionary, chaining."));
}

#[test]
fn showcase580_numerical_integration() {
    let out = run_ore("showcase580.ore");
    assert!(out.contains("Numerical Integration:"));
    assert!(out.contains("n=2, h=5: T=375, error=42"));
    assert!(out.contains("n=10, h=1: T=335, error=2"));
    assert!(out.contains("n=10, h=1: S=333, error=0"));
    assert!(out.contains("Simpson's achieves zero error for polynomials up to degree 3"));
    assert!(out.contains("Monte Carlo (10 points): 280"));
    assert!(out.contains("Numerical integration: trapezoidal, Simpson's, midpoint, Monte Carlo, convergence."));
}

#[test]
fn showcase581_container_orchestration() {
    let out = run_ore("showcase581.ore");
    assert!(out.contains("Container Orchestration:"));
    assert!(out.contains("Scheduled web-app -> node-1"));
    assert!(out.contains("Scheduled db-primary -> node-1"));
    assert!(out.contains("Scheduled cache -> node-2"));
    assert!(out.contains("node-1: CPU 8/8 (100%), MEM 28/32 (87%)"));
    assert!(out.contains("node-1: 3 pod(s)"));
    assert!(out.contains("Scheduled: 8"));
    assert!(out.contains("Failed: 0"));
    assert!(out.contains("Container orchestration: nodes, pods, first-fit scheduling, utilization, health checks."));
}

#[test]
fn showcase582_bloom_filter() {
    let out = run_ore("showcase582.ore");
    assert!(out.contains("Bloom Filter Applications:"));
    assert!(out.contains("Bits set: 24/32"));
    assert!(out.contains("'hello': probably correct"));
    assert!(out.contains("'appple': misspelled"));
    assert!(out.contains("Correct: 3, Misspelled: 3"));
    assert!(out.contains("New URLs: 5, Duplicates: 3"));
    assert!(out.contains("Fill ratio: 75%"));
    assert!(out.contains("Bloom filter: spell checker, URL dedup, false positive analysis."));
}

#[test]
fn showcase583_ssa_construction() {
    let out = run_ore("showcase583.ore");
    assert!(out.contains("SSA Construction:"));
    assert!(out.contains("x_5 = phi(x_3, x_4)"));
    assert!(out.contains("BB0 (entry):"));
    assert!(out.contains("BB3 has phi function:"));
    assert!(out.contains("Optimized values: x_2=3, y_1=9"));
    assert!(out.contains("SSA versions: 7"));
    assert!(out.contains("Phi functions: 1"));
    assert!(out.contains("SSA construction: renaming, basic blocks, phi functions, dominance, use-def chains, optimization."));
}

#[test]
fn showcase584_reaction_diffusion() {
    let out = run_ore("showcase584.ore");
    assert!(out.contains("Reaction-Diffusion System:"));
    assert!(out.contains("Diffusion rate U: 2"));
    assert!(out.contains("After diffusion step:"));
    assert!(out.contains("Feed rate: 4%"));
    assert!(out.contains("U average: 81"));
    assert!(out.contains("Pattern type: spots (activator dip in center)"));
    assert!(out.contains("Reaction-diffusion: grid setup, diffusion, reaction, evolution, pattern analysis."));
}

#[test]
fn showcase585_wal_transaction_log() {
    let out = run_ore("showcase585.ore");
    assert!(out.contains("Database Transaction Log (WAL):"));
    assert!(out.contains("[2] TXN-1 WRITE users id=1 -> name=Alice"));
    assert!(out.contains("TXN-1: committed"));
    assert!(out.contains("Checkpoint at LSN: 6"));
    assert!(out.contains("Redo LSN 7: WRITE accounts id=7"));
    assert!(out.contains("Redo operations: 1"));
    assert!(out.contains("BEGINs: 3"));
    assert!(out.contains("WRITEs: 4"));
    assert!(out.contains("COMMITs: 3"));
    assert!(out.contains("WAL: entries, transactions, checkpoint, recovery, compaction."));
}

#[test]
fn showcase586_quadtree() {
    let out = run_ore("showcase586.ore");
    assert!(out.contains("Spatial Indexing (Quadtree):"));
    assert!(out.contains("A (10,20) -> SW"));
    assert!(out.contains("NW: 4 points"));
    assert!(out.contains("SE: 3 points"));
    assert!(out.contains("Points in range: 5"));
    assert!(out.contains("Nearest: J (55,55)"));
    assert!(out.contains("Distance squared: 50"));
    assert!(out.contains("Quadtree: points, quadrants, subdivision, range query, nearest neighbor."));
}

#[test]
fn showcase587_message_queue() {
    let out = run_ore("showcase587.ore");
    assert!(out.contains("Message Queue System:"));
    assert!(out.contains("Enqueue: 'order-create' (priority=3), size=1"));
    assert!(out.contains("Dequeue: 'order-create' (priority=3), remaining=5"));
    assert!(out.contains("Consumed: 3"));
    assert!(out.contains("Highest priority in queue: 4"));
    assert!(out.contains("Drained: 6 messages"));
    assert!(out.contains("Total produced: 9"));
    assert!(out.contains("Total consumed: 9"));
    assert!(out.contains("Message queue: producers, consumers, circular buffer, priority processing, drain."));
}

#[test]
fn showcase588_abstract_interpretation() {
    let out = run_ore("showcase588.ore");
    assert!(out.contains("Abstract Interpretation - Constant Propagation:"));
    assert!(out.contains("c = const(8) (5 + 3)"));
    assert!(out.contains("e = const(16) (8 * 2)"));
    assert!(out.contains("h = TOP (phi of 10 and 20 -> different constants)"));
    assert!(out.contains("Constants found: 7"));
    assert!(out.contains("Optimization ratio: 70%"));
    assert!(out.contains("Abstract interpretation: constant propagation, lattice, widening, transfer functions."));
}

#[test]
fn showcase589_neural_architecture_search() {
    let out = run_ore("showcase589.ore");
    assert!(out.contains("Neural Architecture Search:"));
    assert!(out.contains("Pareto optimal: Arch-4 (acc=91%, params=280K, lat=10ms)"));
    assert!(out.contains("Pareto set size: 2"));
    assert!(out.contains("Tournament 2: Arch-4 wins (acc=91%)"));
    assert!(out.contains("Best: Arch-4"));
    assert!(out.contains("Accuracy: 91%"));
    assert!(out.contains("Avg parameters: 331K"));
    assert!(out.contains("Neural architecture search: candidates, Pareto frontier, tournament, efficiency scores."));
}

#[test]
fn showcase590_petri_net() {
    let out = run_ore("showcase590.ore");
    assert!(out.contains("Petri Net Simulation:"));
    assert!(out.contains("Step 1: fire 'start' (idle->ready)"));
    assert!(out.contains("Step 15: fire 'finish' (running->done)"));
    assert!(out.contains("done: 3 token(s)"));
    assert!(out.contains("finish: fired 3 times"));
    assert!(out.contains("Token conservation: PASSED"));
    assert!(out.contains("No enabled transitions - DEADLOCK or complete"));
    assert!(out.contains("Total steps: 15"));
    assert!(out.contains("Petri net: places, transitions, firing, reachability, conservation, deadlock detection."));
}

#[test]
fn showcase591_mvcc() {
    let out = run_ore("showcase591.ore");
    assert!(out.contains("Database MVCC Simulation:"));
    assert!(out.contains("TXN-10: row 1 updated 100 -> 150 (ver 2)"));
    assert!(out.contains("TXN-10: COMMITTED"));
    assert!(out.contains("Row 1: value=100 (historical ver 1)"));
    assert!(out.contains("Row 3: value=350 (visible)"));
    assert!(out.contains("CONFLICT: TXN-20 write to row 1 conflicts with TXN-10"));
    assert!(out.contains("TXN-20: ABORTED (write-write conflict)"));
    assert!(out.contains("Versions eligible for GC: 2"));
    assert!(out.contains("TXN-10: committed"));
    assert!(out.contains("MVCC: versioned rows, snapshot isolation, conflict detection, garbage collection."));
}

#[test]
fn showcase592_dataflow() {
    let out = run_ore("showcase592.ore");
    assert!(out.contains("Dataflow Analysis:"));
    assert!(out.contains("entry -> loop_head"));
    assert!(out.contains("Converged in 2 iterations"));
    assert!(out.contains("loop_body: reaches [x, y]"));
    assert!(out.contains("exit: live [x, z]"));
    assert!(out.contains("Dataflow: reaching definitions, live variables, dead code detection."));
}

#[test]
fn showcase593_game_ai() {
    let out = run_ore("showcase593.ore");
    assert!(out.contains("Game AI - Minimax with Alpha-Beta Pruning:"));
    assert!(out.contains("O X ."));
    assert!(out.contains("No winner yet"));
    assert!(out.contains("Move 7: WINNING MOVE (score=100)"));
    assert!(out.contains("Best move: position 7 (score=100)"));
    assert!(out.contains("Nodes evaluated: 31"));
    assert!(out.contains("Game AI: minimax search, alpha-beta pruning, position evaluation."));
}

#[test]
fn showcase594_distance_vector() {
    let out = run_ore("showcase594.ore");
    assert!(out.contains("Distance Vector Routing Protocol:"));
    assert!(out.contains("A -- B: cost 1"));
    assert!(out.contains("Iteration 1: 13 updates"));
    assert!(out.contains("Converged!"));
    assert!(out.contains("to D: cost=3 via B"));
    assert!(out.contains("A to D: A -> B -> D (cost=3, hops=2)"));
    assert!(out.contains("Simulating failure of link A-B"));
    assert!(out.contains("Distance vector: topology, Bellman-Ford, routing tables, path tracing, link failure."));
}

#[test]
fn showcase595_web_crawler() {
    let out = run_ore("showcase595.ore");
    assert!(out.contains("Web Crawler Simulation:"));
    assert!(out.contains("Disallow: /admin"));
    assert!(out.contains("BLOCKED: https://example.com/admin (matches robots.txt)"));
    assert!(out.contains("Crawl #1: https://example.com (priority=10)"));
    assert!(out.contains("NEW: https://example.com/blog/post3"));
    assert!(out.contains("DUPLICATE: https://example.com/about"));
    assert!(out.contains("Pages crawled: 7"));
    assert!(out.contains("Web crawler: URL frontier, robots.txt, priority crawling, link extraction, politeness."));
}

#[test]
fn showcase596_probability() {
    let out = run_ore("showcase596.ore");
    assert!(out.contains("Probability Distributions:"));
    assert!(out.contains("0! = 1"));
    assert!(out.contains("7! = 5040"));
    assert!(out.contains("1 7 21 35 35 21 7 1"));
    assert!(out.contains("3    120       2668"));
    assert!(out.contains("Lambda = n*p = 3"));
    assert!(out.contains("Most likely value: k=3"));
    assert!(out.contains("Probability: binomial coefficients, Pascal's triangle, distribution, CDF, Poisson approximation."));
}

#[test]
fn showcase597_virtual_memory() {
    let out = run_ore("showcase597.ore");
    assert!(out.contains("Virtual Memory - Page Replacement:"));
    assert!(out.contains("FIFO: 14 faults, 6 hits"));
    assert!(out.contains("LRU: 10 faults, 10 hits"));
    assert!(out.contains("Optimal: 8 faults, 12 hits"));
    assert!(out.contains("Clock: 14 faults, 6 hits"));
    assert!(out.contains("Working set fits in memory"));
    assert!(out.contains("Virtual memory: FIFO, LRU, optimal, clock replacement, working set analysis."));
}

#[test]
fn showcase598_register_allocation() {
    let out = run_ore("showcase598.ore");
    assert!(out.contains("Compiler Register Allocation - Linear Scan:"));
    assert!(out.contains("a: [0-5] ======......"));
    assert!(out.contains("Assign a -> R0 (live 0-5)"));
    assert!(out.contains("SPILL d (no register available)"));
    assert!(out.contains("Max pressure: 6 (registers: 3)"));
    assert!(out.contains("Spills: 3"));
    assert!(out.contains("Total interferences: 23"));
    assert!(out.contains("Register allocation: linear scan, live intervals, spilling, pressure analysis."));
}

#[test]
fn showcase599_dht() {
    let out = run_ore("showcase599.ore");
    assert!(out.contains("Distributed Hash Table (DHT):"));
    assert!(out.contains("Ring size: 64"));
    assert!(out.contains("Key 35 -> Node 42 (responsible)"));
    assert!(out.contains("Node 5 fingers:"));
    assert!(out.contains("Lookup key 35: Node 5 -> Node 30 -> Node 42"));
    assert!(out.contains("Hops: 2"));
    assert!(out.contains("Keys redistributed: 1"));
    assert!(out.contains("DHT: consistent hashing, finger tables, routing, replication, node join."));
}

#[test]
fn showcase600_grand_finale() {
    let out = run_ore("showcase600.ore");
    assert!(out.contains("=== Showcase 600: The Grand Finale ==="));
    assert!(out.contains("fibonacci(10) = 55"));
    assert!(out.contains("factorial(6) = 720"));
    assert!(out.contains("gcd(48, 18) = 6"));
    assert!(out.contains("Manhattan distance: 7"));
    assert!(out.contains("Color: red"));
    assert!(out.contains("Sum 1..20: 210"));
    assert!(out.contains("First 5 evens: 2, 4, 6, 8, 10"));
    assert!(out.contains("Keys: alice, bob, charlie, diana"));
    assert!(out.contains("Upper: 'HELLO, ORE PROGRAMMING LANGUAGE!'"));
    assert!(out.contains("Primes < 30: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29"));
    assert!(out.contains("Fib 0..9: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34"));
    assert!(out.contains("Sum of multiples of 15 in 1..100: 315"));
    assert!(out.contains("Capitalized: The Quick Brown Fox Jumps"));
    assert!(out.contains("Sorted: 1, 2, 3, 4, 5, 6, 7, 8, 9"));
    assert!(out.contains("Euler totient(12) = 4"));
    assert!(out.contains("Trace: 15"));
    assert!(out.contains("Primes via sieve < 30: 10"));
    assert!(out.contains("Features demonstrated: 18"));
    assert!(out.contains("Showcase 600: the grand finale of the Ore language!"));
}

#[test]
fn showcase601_symbolic_math() {
    let out = run_ore("showcase601.ore");
    assert!(out.contains("Symbolic Math Simplification:"));
    assert!(out.contains("Rule: x + 0 = x applied to expr 6"));
    assert!(out.contains("Rule: x * 1 = x applied to expr 7"));
    assert!(out.contains("Rule: x * 0 = 0 applied to expr 8"));
    assert!(out.contains("Rule: constant fold 3 + 5 = 8 for expr 9"));
    assert!(out.contains("Rule: constant fold 3 * 5 = 15 for expr 10"));
    assert!(out.contains("Total rules applied: 6"));
    assert!(out.contains("Expr 6 evaluates to: 7"));
    assert!(out.contains("Expr 9 evaluates to: 8"));
    assert!(out.contains("Expr 10 evaluates to: 15"));
    assert!(out.contains("Nodes saved: 10"));
    assert!(out.contains("Symbolic math: expression trees, simplification rules, constant folding, evaluation."));
}

#[test]
fn showcase602_chord_protocol() {
    let out = run_ore("showcase602.ore");
    assert!(out.contains("Chord Protocol Simulation:"));
    assert!(out.contains("Ring size: 64"));
    assert!(out.contains("Nodes: 3, 10, 21, 32, 45, 56"));
    assert!(out.contains("Node 3 -> successor: Node 10"));
    assert!(out.contains("Node 56 -> successor: Node 3"));
    assert!(out.contains("Key 5 -> responsible Node 10"));
    assert!(out.contains("Key 50 -> responsible Node 56"));
    assert!(out.contains("Hops taken: 5"));
    assert!(out.contains("Route: 3 -> 10 -> 21 -> 32 -> 45 -> 56"));
    assert!(out.contains("Joining Node 28"));
    assert!(out.contains("Position: between Node 21 and Node 32"));
    assert!(out.contains("Node 32 has failed!"));
    assert!(out.contains("Max arc gap: 13"));
    assert!(out.contains("Chord: ring topology, finger tables, key lookup, node join, failure detection."));
}

#[test]
fn showcase603_bytecode_verifier() {
    let out = run_ore("showcase603.ore");
    assert!(out.contains("Bytecode Verifier:"));
    assert!(out.contains("0: PUSH 10"));
    assert!(out.contains("9: HALT"));
    assert!(out.contains("Max stack depth: 2"));
    assert!(out.contains("Stack analysis: PASS"));
    assert!(out.contains("HALT found at position 9"));
    assert!(out.contains("Program has HALT: PASS"));
    assert!(out.contains("STORE to var[0] at 7: defined"));
    assert!(out.contains("LOAD from var[0] at 8: OK (defined)"));
    assert!(out.contains("Stack top: 180"));
    assert!(out.contains("Var[0]: 180"));
    assert!(out.contains("Verdict: VERIFIED"));
    assert!(out.contains("Bytecode verifier: stack analysis, type checking, control flow, variable tracking."));
}

#[test]
fn showcase604_reservoir_computing() {
    let out = run_ore("showcase604.ore");
    assert!(out.contains("Reservoir Computing Simulation:"));
    assert!(out.contains("Reservoir size: 6"));
    assert!(out.contains("t=0: state=[2, -1, 3, 0, -2, 1]"));
    assert!(out.contains("Readout weights: 3, -2, 1, 4, -1, 2"));
    assert!(out.contains("Mean absolute error: 4"));
    assert!(out.contains("Non-zero connections: 18"));
    assert!(out.contains("Sparsity: 50%"));
    assert!(out.contains("Reservoir computing: fixed reservoir, state evolution, readout training, memory capacity."));
}

#[test]
fn showcase605_lsm_tree() {
    let out = run_ore("showcase605.ore");
    assert!(out.contains("LSM Tree Simulation:"));
    assert!(out.contains("Insert key=15 val=100 -> memtable size=1"));
    assert!(out.contains("Flushed 4 entries to SSTable-0"));
    assert!(out.contains("SSTable-0 range: [3, 23]"));
    assert!(out.contains("Lookup key=7: found val=200 in SSTable-0"));
    assert!(out.contains("Lookup key=99: NOT FOUND"));
    assert!(out.contains("Lookup key=18: found val=800 in SSTable-1"));
    assert!(out.contains("Merged SSTable (level 1): 8 entries"));
    assert!(out.contains("Bloom filter: 12/16 bits set"));
    assert!(out.contains("Key range: 3 to 25 (span=22)"));
    assert!(out.contains("LSM tree: memtable, SSTable flush, compaction, bloom filter, point lookup."));
}

#[test]
fn showcase606_model_checker() {
    let out = run_ore("showcase606.ore");
    assert!(out.contains("Model Checker Simulation:"));
    assert!(out.contains("Reachable states: 4/4"));
    assert!(out.contains("Safety: NS and EW never both green -> PASS"));
    assert!(out.contains("Safety: At least one direction always red -> PASS"));
    assert!(out.contains("Liveness: All states reachable from initial state -> PASS"));
    assert!(out.contains("Liveness: Returns to initial state in 4 steps -> PASS"));
    assert!(out.contains("AG(!(ns_green & ew_green)): SATISFIED"));
    assert!(out.contains("EF(ns_green): SATISFIED"));
    assert!(out.contains("Mutual exclusion: VERIFIED"));
    assert!(out.contains("Properties checked: 6"));
    assert!(out.contains("Properties satisfied: 6"));
    assert!(out.contains("Properties violated: 0"));
    assert!(out.contains("Model checker: state space, safety, liveness, CTL, mutual exclusion."));
}

#[test]
fn showcase607_wavelet_tree() {
    let out = run_ore("showcase607.ore");
    assert!(out.contains("Wavelet Tree Simulation:"));
    assert!(out.contains("Sequence: 3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9"));
    assert!(out.contains("rank(1, 4) = 2"));
    assert!(out.contains("rank(5, 10) = 2"));
    assert!(out.contains("rank(9, 15) = 3"));
    assert!(out.contains("select(5, 2) = position 8"));
    assert!(out.contains("select(9, 3) = position 14"));
    assert!(out.contains("access(0) = 3 (original: 3)"));
    assert!(out.contains("access(5) = 9 (original: 9)"));
    assert!(out.contains("freq(5, [4,12)) = 3"));
    assert!(out.contains("quantile(1, [2,10)) = 1"));
    assert!(out.contains("Distinct values: 9"));
    assert!(out.contains("Most frequent: 5 (3 times)"));
    assert!(out.contains("Wavelet tree: rank, select, access, range frequency, quantile queries."));
}

#[test]
fn showcase608_proof_assistant() {
    let out = run_ore("showcase608.ore");
    assert!(out.contains("Proof Assistant Simulation:"));
    assert!(out.contains("Verification: VALID (checked all 8 valuations)"));
    assert!(out.contains("De Morgan's Law: VERIFIED"));
    assert!(out.contains("Contrapositive: VERIFIED"));
    assert!(out.contains("Resolution proof: VALID"));
    assert!(out.contains("Theorems proved: 4"));
    assert!(out.contains("Proof assistant: propositional logic, truth tables, modus ponens, De Morgan, resolution."));
}

#[test]
fn showcase609_crdts() {
    let out = run_ore("showcase609.ore");
    assert!(out.contains("CRDT Simulation:"));
    assert!(out.contains("Merged state: [3, 5, 2] total=10"));
    assert!(out.contains("Counter value: 6"));
    assert!(out.contains("Winner: Replica 1 with value=\"world\" at t=25"));
    assert!(out.contains("Merged set: apple, banana, cherry, date"));
    assert!(out.contains("Alive elements: 3"));
    assert!(out.contains("A || B (concurrent)"));
    assert!(out.contains("merge(A,B): [3, 4, 1]"));
    assert!(out.contains("merge(A,B) == C: true"));
    assert!(out.contains("All CRDTs converge without coordination"));
    assert!(out.contains("CRDTs: G-Counter, PN-Counter, LWW-Register, G-Set, OR-Set, vector clocks."));
}

#[test]
fn showcase610_quantum_computing() {
    let out = run_ore("showcase610.ore");
    assert!(out.contains("Quantum Computing Simulation:"));
    assert!(out.contains("X|0> = (0,0)|0> + (1000,0)|1> = |1>"));
    assert!(out.contains("Z|+> = (707,0)|0> + (-707,0)|1> = |->"));
    assert!(out.contains("H|0> = (707,0)|0> + (707,0)|1> ~ |+>"));
    assert!(out.contains("P(|00>) = 49%"));
    assert!(out.contains("P(|11>) = 49%"));
    assert!(out.contains("This is a Bell state: (|00> + |11>)/sqrt(2)"));
    assert!(out.contains("Entangled: measuring one qubit determines the other"));
    assert!(out.contains("Norm check: 1000000 (should be 1000000)"));
    assert!(out.contains("Quantum computing: qubits, gates, measurement, entanglement, teleportation."));
}

#[test]
fn showcase611_sat_solver() {
    let out = run_ore("showcase611.ore");
    assert!(out.contains("SAT Solver (DPLL Algorithm):"));
    assert!(out.contains("Clause 0: SATISFIED"));
    assert!(out.contains("Clause 3: SATISFIED"));
    assert!(out.contains("Result: SATISFIABLE"));
    assert!(out.contains("Result: UNSATISFIABLE (no assignment works)"));
    assert!(out.contains("Clause 2 conflict! Backtracking..."));
    assert!(out.contains("Solution: x1=true, x2=false"));
    assert!(out.contains("SAT solver: CNF encoding, unit propagation, DPLL backtracking, verification."));
}

#[test]
fn showcase612_suffix_array() {
    let out = run_ore("showcase612.ore");
    assert!(out.contains("Suffix Array Construction:"));
    assert!(out.contains("Suffix array: [6, 5, 3, 1, 0, 4, 2]"));
    assert!(out.contains("SA[0] = 6: $"));
    assert!(out.contains("SA[4] = 0: banana$"));
    assert!(out.contains("LCP array: [0, 0, 1, 3, 0, 0, 2]"));
    assert!(out.contains("Total occurrences: 2"));
    assert!(out.contains("Longest repeated substring: ana (length 3)"));
    assert!(out.contains("Distinct substrings: 22"));
    assert!(out.contains("Suffix array: construction, LCP, pattern search, distinct substrings, longest repeat."));
}

#[test]
fn showcase613_van_emde_boas() {
    let out = run_ore("showcase613.ore");
    assert!(out.contains("Van Emde Boas Tree Concepts:"));
    assert!(out.contains("Size: 8, Min: 0, Max: 14"));
    assert!(out.contains("Member(0): YES"));
    assert!(out.contains("Member(1): NO"));
    assert!(out.contains("Successor(0): 2"));
    assert!(out.contains("Predecessor(3): 2"));
    assert!(out.contains("Delete 3: size now 7"));
    assert!(out.contains("After deletions: Min=0, Max=14, Size=6"));
    assert!(out.contains("Van Emde Boas: clusters, summary, O(log log u) operations, successor, predecessor."));
}

#[test]
fn showcase614_garbage_collection() {
    let out = run_ore("showcase614.ore");
    assert!(out.contains("Garbage Collection (Copying Collector):"));
    assert!(out.contains("Total allocated: 8"));
    assert!(out.contains("Copy obj4 -> to[0]"));
    assert!(out.contains("Objects copied: 6"));
    assert!(out.contains("Objects collected: 2"));
    assert!(out.contains("to[3]: Int(42)"));
    assert!(out.contains("to[4]: Int(55)"));
    assert!(out.contains("Before GC: 8/16 slots used"));
    assert!(out.contains("Freed: 2 slots"));
    assert!(out.contains("GC copying collector: semispace, root tracing, forwarding, compaction."));
}

#[test]
fn showcase615_tarjan_lca() {
    let out = run_ore("showcase615.ore");
    assert!(out.contains("Tarjan's Offline LCA:"));
    assert!(out.contains("LCA(6, 4) = 1"));
    assert!(out.contains("LCA(7, 8) = 5"));
    assert!(out.contains("LCA(3, 5) = 0"));
    assert!(out.contains("Leaf nodes: 4"));
    assert!(out.contains("Tree height: 3"));
    assert!(out.contains("Path(6,4): length=3 via LCA=1"));
    assert!(out.contains("Path(7,8): length=2 via LCA=5"));
    assert!(out.contains("Tarjan LCA: tree structure, union-find, offline queries, path lengths."));
}

#[test]
fn showcase616_decompiler() {
    let out = run_ore("showcase616.ore");
    assert!(out.contains("Decompiler Concepts:"));
    assert!(out.contains("Total basic blocks: 4"));
    assert!(out.contains("Back edge: 16 -> 4 (loop detected)"));
    assert!(out.contains("Loops found: 1"));
    assert!(out.contains("Variable initializations: 2"));
    assert!(out.contains("Execution result: 120"));
    assert!(out.contains("Decompiler: bytecode, control flow, loop detection, pattern matching, pseudocode."));
}

#[test]
fn showcase617_bipartite_matching() {
    let out = run_ore("showcase617.ore");
    assert!(out.contains("Maximal Matching in Bipartite Graphs:"));
    assert!(out.contains("Greedy matching size: 4"));
    assert!(out.contains("Maximum matching size: 4"));
    assert!(out.contains("Matching is valid"));
    assert!(out.contains("Minimum vertex cover: 4 (by Konig's theorem)"));
    assert!(out.contains("Maximum independent set: 5"));
    assert!(out.contains("Total edges: 9"));
    assert!(out.contains("Bipartite matching: augmenting paths, maximum matching, Konig's theorem."));
}

#[test]
fn showcase618_cache_coherence() {
    let out = run_ore("showcase618.ore");
    assert!(out.contains("Cache Coherence Protocol (MESI):"));
    assert!(out.contains("P0 reads addr 0: miss -> Exclusive, data=100"));
    assert!(out.contains("P0 has Exclusive -> both Shared"));
    assert!(out.contains("Shared -> Modified, invalidate P1"));
    assert!(out.contains("P0 Modified -> writeback (150), both Shared"));
    assert!(out.contains("Modified: 1"));
    assert!(out.contains("Note: addr 2 dirty in P1 cache (350 vs memory 300)"));
    assert!(out.contains("MESI protocol: cache states, snooping, invalidation, writeback coherence."));
}

#[test]
fn showcase619_persistent_data_structures() {
    let out = run_ore("showcase619.ore");
    assert!(out.contains("Persistent Data Structures (Path Copying):"));
    assert!(out.contains("Version 0: key 5 -> value 50"));
    assert!(out.contains("Version 1: key 5 -> value 55"));
    assert!(out.contains("Version 0: key 12 -> NOT FOUND"));
    assert!(out.contains("Version 2: key 12 -> FOUND"));
    assert!(out.contains("Nodes saved by sharing: 6"));
    assert!(out.contains("Version 2: 6 reachable nodes"));
    assert!(out.contains("Persistent BST: path copying, version history, structural sharing, immutable snapshots."));
}

#[test]
fn showcase620_theorem_proving() {
    let out = run_ore("showcase620.ore");
    assert!(out.contains("Theorem Proving (Resolution Refutation):"));
    assert!(out.contains("EMPTY CLAUSE derived!"));
    assert!(out.contains("Negation is unsatisfiable -> theorem is VALID"));
    assert!(out.contains("Theorem is VALID"));
    assert!(out.contains("Claim is NOT VALID"));
    assert!(out.contains("Step 4: [S] + [~S] -> EMPTY"));
    assert!(out.contains("Total resolution steps: 9"));
    assert!(out.contains("Resolution refutation: CNF, complementary literals, empty clause, theorem validity."));
}

#[test]
fn showcase621_gc_mark_compact() {
    let out = run_ore("showcase621.ore");
    assert!(out.contains("Garbage Collection Mark-Compact:"));
    assert!(out.contains("Marked: 7, Garbage: 1"));
    assert!(out.contains("GARBAGE (will be collected)"));
    assert!(out.contains("Compacted heap size: 36"));
    assert!(out.contains("Freed: 2"));
    assert!(out.contains("Fragmentation after compaction: 0%"));
    assert!(out.contains("Compaction ratio: 36 / 38"));
    assert!(out.contains("GC mark-compact: root tracing, mark phase, forwarding addresses, reference update, compaction."));
}

#[test]
fn showcase622_parallel_prefix_sum() {
    let out = run_ore("showcase622.ore");
    assert!(out.contains("Parallel Prefix Sum (Scan):"));
    assert!(out.contains("Inclusive scan: [3, 4, 11, 11, 15, 16, 22, 25]"));
    assert!(out.contains("Exclusive scan: [0, 3, 4, 11, 11, 15, 16, 22]"));
    assert!(out.contains("Root contains total sum: 25"));
    assert!(out.contains("Blelloch result matches sequential: PASS"));
    assert!(out.contains("Compacted elements: 4 out of 8"));
    assert!(out.contains("Parallel prefix sum: inclusive/exclusive scan, Blelloch algorithm, stream compaction."));
}

#[test]
fn showcase623_instruction_scheduling() {
    let out = run_ore("showcase623.ore");
    assert!(out.contains("Instruction Scheduling:"));
    assert!(out.contains("I2 depends on I0 (RAW: r0)"));
    assert!(out.contains("I3 depends on I2 (RAW: r2)"));
    assert!(out.contains("Cycle 0: schedule I0 (LOAD)"));
    assert!(out.contains("Cycle 3: schedule I2 (ADD)"));
    assert!(out.contains("Total cycles: 6"));
    assert!(out.contains("Instruction scheduling: dependency graph, list scheduling, pipeline stalls, ILP analysis."));
}

#[test]
fn showcase624_interval_graph_coloring() {
    let out = run_ore("showcase624.ore");
    assert!(out.contains("Interval Graph Coloring:"));
    assert!(out.contains("Total overlapping pairs: 9"));
    assert!(out.contains("Chromatic number: 3"));
    assert!(out.contains("Equals chromatic number: VERIFIED (perfect graph)"));
    assert!(out.contains("Maximum independent set size: 5"));
    assert!(out.contains("Interval graph coloring: overlap detection, greedy coloring, sweep line, independent set."));
}

#[test]
fn showcase625_mvc_pattern() {
    let out = run_ore("showcase625.ore");
    assert!(out.contains("Model-View-Controller Pattern:"));
    assert!(out.contains("Toggled 'Buy groceries' -> done"));
    assert!(out.contains("Completed: 3/5"));
    assert!(out.contains("Total pending: 2"));
    assert!(out.contains("High priority items: 2"));
    assert!(out.contains("Separation of concerns: data, presentation, logic"));
    assert!(out.contains("MVC pattern: model data store, view rendering, controller actions, filtered views."));
}

#[test]
fn showcase626_network_flow_min_cut() {
    let out = run_ore("showcase626.ore");
    assert!(out.contains("Network Flow Minimum Cut:"));
    assert!(out.contains("Total flow after 3 augmentations: 23"));
    assert!(out.contains("Min-cut capacity: 23"));
    assert!(out.contains("Max-flow = Min-cut: VERIFIED"));
    assert!(out.contains("Edge 1->3: flow=12/12 (100%)"));
    assert!(out.contains("Network flow min-cut: Ford-Fulkerson, residual graph, BFS reachability, cut edges."));
}

#[test]
fn showcase627_interpreter_with_closures() {
    let out = run_ore("showcase627.ore");
    assert!(out.contains("Interpreter with Closures:"));
    assert!(out.contains("Output: 14"));
    assert!(out.contains("Output: 30"));
    assert!(out.contains("Output: 200"));
    assert!(out.contains("adder5(3) = 8"));
    assert!(out.contains("adder10(7) = 17"));
    assert!(out.contains("make_multiplier(2)(3)(4) = 24"));
    assert!(out.contains("Counter 0 after 5 increments: 5"));
    assert!(out.contains("Interpreter with closures: bytecode execution, variable binding, closure capture, nested environments."));
}

#[test]
fn showcase628_boyer_moore() {
    let out = run_ore("showcase628.ore");
    assert!(out.contains("Boyer-Moore String Matching:"));
    assert!(out.contains("Total matches: 3"));
    assert!(out.contains("Pattern 'ABC' found 3 times"));
    assert!(out.contains("Matches: 4"));
    assert!(out.contains("Comparisons: 3 (best case: n/m)"));
    assert!(out.contains("Boyer-Moore is faster"));
    assert!(out.contains("Boyer-Moore: bad character table, right-to-left scan, shift heuristics, pattern matching."));
}

#[test]
fn showcase629_concurrent_data_structures() {
    let out = run_ore("showcase629.ore");
    assert!(out.contains("Concurrent Data Structure Concepts:"));
    assert!(out.contains("Pop: 50 (from node 4)"));
    assert!(out.contains("CAS(100->200): SUCCESS, value now 200"));
    assert!(out.contains("CAS(100->300): FAILED, value is 200"));
    assert!(out.contains("Final counter: 12"));
    assert!(out.contains("ABA detected! CAS rejected"));
    assert!(out.contains("Dequeue: 100"));
    assert!(out.contains("Concurrent data structures: CAS operations, lock-free stack/queue, ABA problem, linearizability."));
}

#[test]
fn showcase630_symbolic_regression() {
    let out = run_ore("showcase630.ore");
    assert!(out.contains("Symbolic Regression:"));
    assert!(out.contains("Sum squared error: 152"));
    assert!(out.contains("PERFECT FIT!"));
    assert!(out.contains("Formulas tried: 125"));
    assert!(out.contains("Best: y = 2*x^2 + 3*x + 1"));
    assert!(out.contains("Found: y = 1*x^2 + 0*x + 1"));
    assert!(out.contains("Symbolic regression: data fitting, expression search, coefficient optimization, formula discovery."));
}

#[test]
fn showcase631_escape_analysis() {
    let out = run_ore("showcase631.ore");
    assert!(out.contains("Escape Analysis:"));
    assert!(out.contains("'ptr': ESCAPES (used in outer scope 0)"));
    assert!(out.contains("Escaping: 1, Local: 7"));
    assert!(out.contains("'ptr' -> HEAP allocation (escapes scope)"));
    assert!(out.contains("'counter' captured by reference -> must heap allocate"));
    assert!(out.contains("Stack optimization rate: 87%"));
    assert!(out.contains("Escape analysis: scope tracking, allocation decisions, closure captures, heap vs stack."));
}

#[test]
fn showcase632_fibonacci_heap() {
    let out = run_ore("showcase632.ore");
    assert!(out.contains("Fibonacci Heap Concepts:"));
    assert!(out.contains("Min: 1"));
    assert!(out.contains("Heap size: 11, Min: 0"));
    assert!(out.contains("Merges performed: 5"));
    assert!(out.contains("decrease-key: O(1) amortized"));
    assert!(out.contains("Extract #1: 1"));
    assert!(out.contains("Extract #4: 10"));
    assert!(out.contains("Fibonacci heap: lazy merging, O(1) insert and decrease-key, consolidation on extract-min."));
}

#[test]
fn showcase633_program_verifier() {
    let out = run_ore("showcase633.ore");
    assert!(out.contains("Simple Program Verifier:"));
    assert!(out.contains("VERIFIED: x + y >= -10"));
    assert!(out.contains("Loop invariant verified for all iterations"));
    assert!(out.contains("Final sum: 55"));
    assert!(out.contains("Safe: 8, Unsafe: 0"));
    assert!(out.contains("Violations: 0"));
    assert!(out.contains("FAIL: x > 0 (counterexample possible)"));
    assert!(out.contains("Passed: 4, Failed: 1"));
    assert!(out.contains("Program verifier: range analysis, loop invariants, bounds checking, type state, assertions."));
}

#[test]
fn showcase634_sparse_table() {
    let out = run_ore("showcase634.ore");
    assert!(out.contains("Sparse Table (RMQ):"));
    assert!(out.contains("RMQ(0, 7) = 1 (full array)"));
    assert!(out.contains("RMQ(0, 3) = 2"));
    assert!(out.contains("RMQ(1, 4) = 1"));
    assert!(out.contains("RMQ(5, 7) = 3"));
    assert!(out.contains("Brute force min(0,7) = 1, sparse table = 1, match = true"));
    assert!(out.contains("Levels built: 4"));
    assert!(out.contains("Sparse table: O(1) range minimum queries, log-level preprocessing, idempotent overlap."));
}

#[test]
fn showcase635_gc_tricolor() {
    let out = run_ore("showcase635.ore");
    assert!(out.contains("GC Tri-Color Marking:"));
    assert!(out.contains("Mark root A gray"));
    assert!(out.contains("Mark root G gray"));
    assert!(out.contains("A: BLACK (reachable)"));
    assert!(out.contains("H: WHITE (garbage)"));
    assert!(out.contains("J: WHITE (garbage)"));
    assert!(out.contains("Free H"));
    assert!(out.contains("Freed 2 objects"));
    assert!(out.contains("Retained 8 objects"));
    assert!(out.contains("Reclaimed: 20%"));
    assert!(out.contains("Tri-color GC: root tracing, gray worklist, mark-and-sweep, unreachable detection."));
}

#[test]
fn showcase636_cartesian_tree() {
    let out = run_ore("showcase636.ore");
    assert!(out.contains("Cartesian Tree:"));
    assert!(out.contains("Root: arr[3] = 1"));
    assert!(out.contains("Min-heap property: VERIFIED"));
    assert!(out.contains("BST property: VERIFIED (in-order = 0..8)"));
    assert!(out.contains("RMQ(0,8) = arr[3] = 1 (root = global min)"));
    assert!(out.contains("Cartesian tree: min-heap by value, BST by index, stack-based O(n) construction."));
}

#[test]
fn showcase637_program_slicing() {
    let out = run_ore("showcase637.ore");
    assert!(out.contains("Program Slicing:"));
    assert!(out.contains("Tracing dependency on 'v'"));
    assert!(out.contains("Include [5] v := z * w"));
    assert!(out.contains("Slice size: 7/7 statements"));
    assert!(out.contains("Forward slice size: 5/7 statements"));
    assert!(out.contains("Dead statements: 0"));
    assert!(out.contains("Program slicing: backward and forward slicing, dependency tracking, dead code detection."));
}

#[test]
fn showcase638_heavy_light_decomposition() {
    let out = run_ore("showcase638.ore");
    assert!(out.contains("Heavy-Light Decomposition:"));
    assert!(out.contains("Node 0: heavy child = 1"));
    assert!(out.contains("Edge 0-1: heavy"));
    assert!(out.contains("Total chains: 4"));
    assert!(out.contains("LCA(6, 8) = 0"));
    assert!(out.contains("Heavy edges: 5"));
    assert!(out.contains("Light edges: 3"));
    assert!(out.contains("HLD: heavy chains, light edges, O(log n) path queries, subtree decomposition."));
}

#[test]
fn showcase639_abstract_machine() {
    let out = run_ore("showcase639.ore");
    assert!(out.contains("Simple Abstract Machine:"));
    assert!(out.contains("PUSH 3  -> stack top: 3"));
    assert!(out.contains("MUL        -> 3 * 4 = 12"));
    assert!(out.contains("Result: 17, Steps: 7"));
    assert!(out.contains("5! = 120, Steps: 11"));
    assert!(out.contains("Stack machine sum(1..10) = 55"));
    assert!(out.contains("Abstract machine: stack-based VM, instruction decoding, arithmetic operations, program execution."));
}

#[test]
fn showcase640_fenwick_tree() {
    let out = run_ore("showcase640.ore");
    assert!(out.contains("Fenwick Tree (BIT):"));
    assert!(out.contains("prefix_sum(0..0) = 3 (brute force: 3) OK"));
    assert!(out.contains("prefix_sum(0..9) = 28 (brute force: 28) OK"));
    assert!(out.contains("sum(2..5) = 14 (verify: 14)"));
    assert!(out.contains("sum(0..9) = 28 (verify: 28)"));
    assert!(out.contains("After update - total sum: brute=32, BIT=32"));
    assert!(out.contains("Fenwick tree: binary indexed tree, prefix sums, point updates, range queries, O(log n)."));
}

#[test]
fn cli_check_valid() {
    let path = fixtures_dir().join("showcase80.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["check", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore check");
    assert!(output.status.success(), "ore check should pass for valid code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ok"));
}

#[test]
fn cli_check_type_error() {
    let path = fixtures_dir().join("check_error.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["check", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore check");
    assert!(!output.status.success(), "ore check should fail for type errors");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("expects Int, got Str"));
}

#[test]
fn cli_fmt_output() {
    let path = fixtures_dir().join("showcase10.ore");
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["fmt", path.to_str().unwrap()])
        .output()
        .expect("failed to execute ore fmt");
    assert!(output.status.success(), "ore fmt should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let out = String::from_utf8(output.stdout).unwrap();
    assert!(out.contains("fn main"));
    assert!(out.contains("print"));
}

#[test]
fn cli_eval_expression() {
    let output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["eval", "2 + 3 * 4"])
        .output()
        .expect("failed to execute ore eval");
    assert!(output.status.success(), "ore eval should succeed");
    let out = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert_eq!(out, "14");
}

#[test]
fn build_and_run_binary() {
    // Test `ore build` produces a working native binary
    let path = fixtures_dir().join("showcase36.ore");
    let tmp_bin = std::env::temp_dir().join("ore_test_build_binary");

    let build_output = Command::new(env!("CARGO_BIN_EXE_ore"))
        .args(["build", path.to_str().unwrap(), "-o", tmp_bin.to_str().unwrap()])
        .output()
        .expect("failed to execute ore build");

    assert!(build_output.status.success(),
        "ore build failed: {}", String::from_utf8_lossy(&build_output.stderr));

    // Run the compiled binary
    let run_output = Command::new(&tmp_bin)
        .output()
        .expect("failed to run compiled binary");

    assert!(run_output.status.success(), "compiled binary failed");
    let out = String::from_utf8(run_output.stdout).unwrap();
    assert!(out.contains("10! = 3628800"));
    assert!(out.contains("Perfect numbers < 500: 6, 28, 496"));

    // Clean up
    let _ = std::fs::remove_file(&tmp_bin);
}
