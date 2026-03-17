use super::*;

impl CCodeGen {
    pub(crate) fn compile_builtin_call(&mut self, name: &str, args: &[Expr]) -> Result<std::option::Option<(String, ValKind)>, CCodeGenError> {
        match name {
            "abs" => {
                self.check_arity("abs", args, 1)?;
                let (val, kind) = self.compile_expr(&args[0])?;
                match kind {
                    ValKind::Int => Ok(Some((format!("(({0}) < 0 ? -({0}) : ({0}))", val), ValKind::Int))),
                    ValKind::Float => Ok(Some((format!("ore_math_abs({})", val), ValKind::Float))),
                    _ => Err(self.err("abs requires Int or Float")),
                }
            }
            "min" | "max" => {
                self.check_arity(name, args, 2)?;
                let (a, ak) = self.compile_expr(&args[0])?;
                let (b, _) = self.compile_expr(&args[1])?;
                let op = if name == "min" { "<" } else { ">" };
                let kind = if ak == ValKind::Float { ValKind::Float } else { ValKind::Int };
                Ok(Some((format!("(({0}) {2} ({1}) ? ({0}) : ({1}))", a, b, op), kind)))
            }
            "channel" => Ok(Some(("ore_channel_new()".to_string(), ValKind::Channel))),
            "readln" | "input" => {
                if args.len() == 1 {
                    let (prompt, _) = self.compile_expr(&args[0])?;
                    self.emit(&format!("ore_str_print_no_newline({});", prompt));
                }
                Ok(Some(("ore_readln()".to_string(), ValKind::Str)))
            }
            "file_read" => {
                self.check_arity("file_read", args, 1)?;
                let (path, _) = self.compile_expr(&args[0])?;
                Ok(Some((format!("ore_file_read({})", path), ValKind::Str)))
            }
            "file_read_lines" => {
                self.check_arity("file_read_lines", args, 1)?;
                let (path, _) = self.compile_expr(&args[0])?;
                Ok(Some((format!("ore_file_read_lines({})", path), ValKind::list_of(ValKind::Str))))
            }
            "file_write" | "file_append" => {
                self.check_arity(name, args, 2)?;
                let (path, _) = self.compile_expr(&args[0])?;
                let (content, _) = self.compile_expr(&args[1])?;
                Ok(Some((format!("ore_{}({}, {})", name, path, content), ValKind::Bool)))
            }
            "file_exists" => {
                self.check_arity("file_exists", args, 1)?;
                let (path, _) = self.compile_expr(&args[0])?;
                Ok(Some((format!("(ore_file_exists({}) != 0)", path), ValKind::Bool)))
            }
            "env_get" => {
                self.check_arity("env_get", args, 1)?;
                let (key, _) = self.compile_expr(&args[0])?;
                Ok(Some((format!("ore_env_get({})", key), ValKind::Str)))
            }
            "env_set" => {
                self.check_arity("env_set", args, 2)?;
                let (key, _) = self.compile_expr(&args[0])?;
                let (val, _) = self.compile_expr(&args[1])?;
                self.emit(&format!("ore_env_set({}, {});", key, val));
                Ok(Some(("0".to_string(), ValKind::Void)))
            }
            "args" => Ok(Some(("ore_args()".to_string(), ValKind::list_of(ValKind::Str)))),
            "eprint" => {
                self.check_arity("eprint", args, 1)?;
                let (val, kind) = self.compile_expr(&args[0])?;
                let rt = match kind {
                    ValKind::Str => "ore_eprint_str",
                    ValKind::Float => "ore_eprint_float",
                    ValKind::Bool => "ore_eprint_bool",
                    _ => "ore_eprint_int",
                };
                self.emit(&format!("{}({});", rt, val));
                Ok(Some(("0".to_string(), ValKind::Void)))
            }
            "exit" => {
                self.check_arity("exit", args, 1)?;
                let (code, _) = self.compile_expr(&args[0])?;
                self.emit(&format!("ore_exit({});", code));
                Ok(Some(("0".to_string(), ValKind::Void)))
            }
            "exec" => {
                self.check_arity("exec", args, 1)?;
                let (cmd, _) = self.compile_expr(&args[0])?;
                Ok(Some((format!("ore_exec({})", cmd), ValKind::Str)))
            }
            "str" => {
                self.check_arity("str", args, 1)?;
                let (val, kind) = self.compile_expr(&args[0])?;
                let result = self.value_to_str_expr(&val, &kind);
                Ok(Some((result, ValKind::Str)))
            }
            "int" => {
                self.check_arity("int", args, 1)?;
                let (val, kind) = self.compile_expr(&args[0])?;
                match kind {
                    ValKind::Int => Ok(Some((val, ValKind::Int))),
                    ValKind::Float => Ok(Some((format!("(int64_t)({})", val), ValKind::Int))),
                    ValKind::Bool => Ok(Some((format!("(int64_t)({})", val), ValKind::Int))),
                    ValKind::Str => Ok(Some((format!("ore_str_to_int({})", val), ValKind::Int))),
                    _ => Err(self.err("int() cannot convert this type")),
                }
            }
            "float" => {
                self.check_arity("float", args, 1)?;
                let (val, kind) = self.compile_expr(&args[0])?;
                match kind {
                    ValKind::Float => Ok(Some((val, ValKind::Float))),
                    ValKind::Int => Ok(Some((format!("(double)({})", val), ValKind::Float))),
                    ValKind::Str => Ok(Some((format!("ore_str_to_float({})", val), ValKind::Float))),
                    _ => Err(self.err("float() cannot convert this type")),
                }
            }
            "ord" => {
                self.check_arity("ord", args, 1)?;
                let (val, _) = self.compile_expr(&args[0])?;
                Ok(Some((format!("ore_ord({})", val), ValKind::Int)))
            }
            "chr" => {
                self.check_arity("chr", args, 1)?;
                let (val, _) = self.compile_expr(&args[0])?;
                Ok(Some((format!("ore_chr({})", val), ValKind::Str)))
            }
            "type_of" | "typeof" => {
                self.check_arity(name, args, 1)?;
                let (_, kind) = self.compile_expr(&args[0])?;
                let type_name = match &kind {
                    ValKind::Int => "Int", ValKind::Float => "Float", ValKind::Bool => "Bool",
                    ValKind::Str => "Str", ValKind::List(_) => "List", ValKind::Map(_) => "Map",
                    ValKind::Option => "Option", ValKind::Result => "Result",
                    ValKind::Channel => "Channel", ValKind::Void => "Void",
                    ValKind::Record(n) | ValKind::Enum(n) => n,
                };
                let str_val = self.compile_string_literal(type_name);
                Ok(Some((str_val, ValKind::Str)))
            }
            "rand_int" => {
                self.check_arity("rand_int", args, 2)?;
                let (lo, _) = self.compile_expr(&args[0])?;
                let (hi, _) = self.compile_expr(&args[1])?;
                Ok(Some((format!("ore_rand_int({}, {})", lo, hi), ValKind::Int)))
            }
            "time_now" | "time_ms" => {
                Ok(Some((format!("ore_{}()", name), ValKind::Int)))
            }
            "json_parse" => {
                self.check_arity("json_parse", args, 1)?;
                let (val, _) = self.compile_expr(&args[0])?;
                Ok(Some((format!("ore_json_parse({})", val), ValKind::Map(None))))
            }
            "json_stringify" => {
                self.check_arity("json_stringify", args, 1)?;
                let (val, _) = self.compile_expr(&args[0])?;
                Ok(Some((format!("ore_json_stringify({})", val), ValKind::Str)))
            }
            "repeat" => {
                self.check_arity("repeat", args, 2)?;
                let (val, kind) = self.compile_expr(&args[0])?;
                let i64_val = self.value_to_i64_expr(&val, &kind);
                let (count, _) = self.compile_expr(&args[1])?;
                Ok(Some((format!("ore_list_repeat({}, {})", i64_val, count), ValKind::list_of(kind))))
            }
            "range" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(self.err("range takes 2-3 arguments"));
                }
                let (start, _) = self.compile_expr(&args[0])?;
                let (end, _) = self.compile_expr(&args[1])?;
                if args.len() == 3 {
                    let (step, _) = self.compile_expr(&args[2])?;
                    Ok(Some((format!("ore_range_step({}, {}, {})", start, end, step), ValKind::list_of(ValKind::Int))))
                } else {
                    Ok(Some((format!("ore_range({}, {})", start, end), ValKind::list_of(ValKind::Int))))
                }
            }
            "len" => {
                self.check_arity("len", args, 1)?;
                let (val, kind) = self.compile_expr(&args[0])?;
                match kind {
                    ValKind::Str => Ok(Some((format!("ore_str_len({})", val), ValKind::Int))),
                    ValKind::List(_) => Ok(Some((format!("ore_list_len({})", val), ValKind::Int))),
                    ValKind::Map(_) => Ok(Some((format!("ore_map_len({})", val), ValKind::Int))),
                    _ => Err(self.err("len() not supported on this type")),
                }
            }
            "assert" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(self.err("assert takes 1-2 arguments"));
                }
                let (cond, _) = self.compile_expr(&args[0])?;
                if args.len() == 2 {
                    let (msg, _) = self.compile_expr(&args[1])?;
                    self.emit(&format!("if (!({cond})) ore_assert_fail({msg});"));
                } else {
                    let msg = self.compile_string_literal(&format!("assertion failed at line {}", self.current_line));
                    self.emit(&format!("if (!({cond})) ore_assert_fail({msg});"));
                }
                Ok(Some(("0".to_string(), ValKind::Void)))
            }
            // Math functions
            "sqrt" | "sin" | "cos" | "tan" | "log" | "log10" | "exp" | "floor" | "ceil" | "round"
            | "math_abs" | "math_floor" | "math_ceil" | "math_round" => {
                if (name == "round" || name == "math_round") && args.len() == 2 {
                    let (val, kind) = self.compile_expr(&args[0])?;
                    let f_val = if kind == ValKind::Int { format!("(double)({})", val) } else { val };
                    let (dec, _) = self.compile_expr(&args[1])?;
                    return Ok(Some((format!("ore_float_round_to({}, {})", f_val, dec), ValKind::Float)));
                }
                self.check_arity(name, args, 1)?;
                let (val, kind) = self.compile_expr(&args[0])?;
                let f_val = if kind == ValKind::Int { format!("(double)({})", val) } else { val };
                let rt_name = name.strip_prefix("math_").unwrap_or(name);
                Ok(Some((format!("ore_math_{}({})", rt_name, f_val), ValKind::Float)))
            }
            "pow" | "atan2" => {
                self.check_arity(name, args, 2)?;
                let (a, ak) = self.compile_expr(&args[0])?;
                let (b, bk) = self.compile_expr(&args[1])?;
                let af = if ak == ValKind::Int { format!("(double)({})", a) } else { a };
                let bf = if bk == ValKind::Int { format!("(double)({})", b) } else { b };
                Ok(Some((format!("ore_math_{}({}, {})", name, af, bf), ValKind::Float)))
            }
            "pi" | "euler" | "e" => {
                let rt = if name == "pi" { "ore_math_pi" } else { "ore_math_e" };
                Ok(Some((format!("{}()", rt), ValKind::Float)))
            }
            "__range" => {
                // Internal function used by list comprehension parser
                self.check_arity("__range", args, 2)?;
                let (start, _) = self.compile_expr(&args[0])?;
                let (end, _) = self.compile_expr(&args[1])?;
                Ok(Some((format!("ore_range({}, {})", start, end), ValKind::list_of(ValKind::Int))))
            }
            _ => Ok(None),
        }
    }
}
