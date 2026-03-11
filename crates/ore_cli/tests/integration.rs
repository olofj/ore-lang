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
