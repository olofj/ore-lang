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
