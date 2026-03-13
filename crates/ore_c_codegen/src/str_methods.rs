use super::*;

impl CCodeGen {
    pub(crate) fn compile_str_method(&mut self, val: &str, method: &str, args: &[Expr]) -> Result<(String, ValKind), CCodeGenError> {
        match method {
            "len" => Ok((format!("ore_str_len({})", val), ValKind::Int)),
            "is_empty" => Ok((format!("(ore_str_len({}) == 0)", val), ValKind::Bool)),
            "contains" => {
                let (needle, _) = self.compile_expr(&args[0])?;
                Ok((format!("(ore_str_contains({}, {}) != 0)", val, needle), ValKind::Bool))
            }
            "trim" | "trim_start" | "trim_end" => {
                Ok((format!("ore_str_{}({})", method, val), ValKind::Str))
            }
            "words" => Ok((format!("ore_str_split_whitespace({})", val), ValKind::list_of(ValKind::Str))),
            "lines" => Ok((format!("ore_str_lines({})", val), ValKind::list_of(ValKind::Str))),
            "chars" => Ok((format!("ore_str_chars({})", val), ValKind::list_of(ValKind::Str))),
            "split" => {
                if args.is_empty() {
                    return Ok((format!("ore_str_split_whitespace({})", val), ValKind::list_of(ValKind::Str)));
                }
                let (delim, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_str_split({}, {})", val, delim), ValKind::list_of(ValKind::Str)))
            }
            "to_int" | "parse_int" => Ok((format!("ore_str_to_int({})", val), ValKind::Int)),
            "to_float" | "parse_float" => Ok((format!("ore_str_to_float({})", val), ValKind::Float)),
            "replace" => {
                let (from, _) = self.compile_expr(&args[0])?;
                let (to, _) = self.compile_expr(&args[1])?;
                Ok((format!("ore_str_replace({}, {}, {})", val, from, to), ValKind::Str))
            }
            "starts_with" | "ends_with" => {
                let (arg, _) = self.compile_expr(&args[0])?;
                Ok((format!("(ore_str_{}({}, {}) != 0)", method, val, arg), ValKind::Bool))
            }
            "to_upper" | "to_lower" | "capitalize" | "reverse" => {
                Ok((format!("ore_str_{}({})", method, val), ValKind::Str))
            }
            "substr" => {
                let (start, _) = self.compile_expr(&args[0])?;
                let (len, _) = self.compile_expr(&args[1])?;
                Ok((format!("ore_str_substr({}, {}, {})", val, start, len), ValKind::Str))
            }
            "char_at" => {
                let (idx, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_str_char_at({}, {})", val, idx), ValKind::Str))
            }
            "index_of" | "find" => {
                let (needle, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_str_index_of({}, {})", val, needle), ValKind::Int))
            }
            "slice" => {
                let (start, _) = self.compile_expr(&args[0])?;
                let (end, _) = self.compile_expr(&args[1])?;
                Ok((format!("ore_str_slice({}, {}, {})", val, start, end), ValKind::Str))
            }
            "repeat" => {
                let (count, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_str_repeat({}, {})", val, count), ValKind::Str))
            }
            "count" => {
                let (needle, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_str_count({}, {})", val, needle), ValKind::Int))
            }
            "strip_prefix" | "strip_suffix" => {
                let (arg, _) = self.compile_expr(&args[0])?;
                Ok((format!("ore_str_{}({}, {})", method, val, arg), ValKind::Str))
            }
            "pad_left" | "pad_right" => {
                let (width, _) = self.compile_expr(&args[0])?;
                let pad = if args.len() > 1 {
                    let (p, _) = self.compile_expr(&args[1])?;
                    p
                } else {
                    self.compile_string_literal(" ")
                };
                Ok((format!("ore_str_{}({}, {}, {})", method, val, width, pad), ValKind::Str))
            }
            _ => Err(self.err(format!("unknown Str method '{}'", method))),
        }
    }
}
