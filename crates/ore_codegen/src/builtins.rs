use super::*;
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::IntPredicate;

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn compile_typed_list_print(
        &mut self,
        list_ptr: PointerValue<'ctx>,
        elem_kind: &ValKind,
    ) -> Result<(), CodeGenError> {
        // Print "[" using ore_str_print
        let open_bracket = self.compile_string_literal("[")?;
        let str_print = self.rt("ore_str_print_no_newline")?;
        bld!(self.builder.build_call(str_print, &[open_bracket.into()], ""))?;
        let release = self.rt("ore_str_release")?;
        bld!(self.builder.build_call(release, &[open_bracket.into()], ""))?;

        let list_len = self.rt("ore_list_len")?;
        let list_get = self.rt("ore_list_get")?;

        let len_result = bld!(self.builder.build_call(list_len, &[list_ptr.into()], "len"))?;
        let len = self.call_result_to_value(len_result)?.into_int_value();

        let current_fn = self.current_fn()?;

        // Loop: for i in 0..len
        let idx_alloca = bld!(self.builder.build_alloca(self.context.i64_type(), "idx"))?;
        bld!(self.builder.build_store(idx_alloca, self.context.i64_type().const_int(0, false)))?;

        let loop_check = self.context.append_basic_block(current_fn, "list_print_check");
        let loop_body = self.context.append_basic_block(current_fn, "list_print_body");
        let loop_end = self.context.append_basic_block(current_fn, "list_print_end");

        bld!(self.builder.build_unconditional_branch(loop_check))?;

        self.builder.position_at_end(loop_check);
        let i = bld!(self.builder.build_load(self.context.i64_type(), idx_alloca, "i"))?.into_int_value();
        let cond = bld!(self.builder.build_int_compare(IntPredicate::SLT, i, len, "cmp"))?;
        bld!(self.builder.build_conditional_branch(cond, loop_body, loop_end))?;

        self.builder.position_at_end(loop_body);
        let i = bld!(self.builder.build_load(self.context.i64_type(), idx_alloca, "i"))?.into_int_value();

        // Print ", " if not first element
        let is_first = bld!(self.builder.build_int_compare(IntPredicate::EQ, i, self.context.i64_type().const_int(0, false), "first"))?;
        let sep_bb = self.context.append_basic_block(current_fn, "print_sep");
        let elem_bb = self.context.append_basic_block(current_fn, "print_elem");
        bld!(self.builder.build_conditional_branch(is_first, elem_bb, sep_bb))?;

        self.builder.position_at_end(sep_bb);
        let sep = self.compile_string_literal(", ")?;
        bld!(self.builder.build_call(str_print, &[sep.into()], ""))?;
        bld!(self.builder.build_call(release, &[sep.into()], ""))?;
        bld!(self.builder.build_unconditional_branch(elem_bb))?;

        self.builder.position_at_end(elem_bb);
        let i = bld!(self.builder.build_load(self.context.i64_type(), idx_alloca, "i"))?.into_int_value();

        // Get element
        let elem_result = bld!(self.builder.build_call(list_get, &[list_ptr.into(), i.into()], "elem"))?;
        let elem_i64 = self.call_result_to_value(elem_result)?.into_int_value();

        // Convert and print based on element kind
        match elem_kind {
            ValKind::Str => {
                let elem_ptr = bld!(self.builder.build_int_to_ptr(elem_i64, self.ptr_type(), "str_ptr"))?;
                bld!(self.builder.build_call(str_print, &[elem_ptr.into()], ""))?;
            }
            ValKind::Float => {
                let f = bld!(self.builder.build_bit_cast(elem_i64, self.context.f64_type(), "f"))?.into_float_value();
                let print_float = self.rt("ore_print_float_no_newline")?;
                bld!(self.builder.build_call(print_float, &[f.into()], ""))?;
            }
            ValKind::Bool => {
                let b = bld!(self.builder.build_int_truncate(elem_i64, self.context.i8_type(), "b"))?;
                let print_bool = self.rt("ore_print_bool_no_newline")?;
                bld!(self.builder.build_call(print_bool, &[b.into()], ""))?;
            }
            _ => {
                let print_int = self.rt("ore_print_int_no_newline")?;
                bld!(self.builder.build_call(print_int, &[elem_i64.into()], ""))?;
            }
        }

        // Increment
        let next_i = bld!(self.builder.build_int_add(i, self.context.i64_type().const_int(1, false), "next_i"))?;
        bld!(self.builder.build_store(idx_alloca, next_i))?;
        bld!(self.builder.build_unconditional_branch(loop_check))?;

        self.builder.position_at_end(loop_end);
        // Print "]\n"
        let close_str = self.compile_string_literal("]")?;
        let print_str_fn = self.rt("ore_str_print")?;
        bld!(self.builder.build_call(print_str_fn, &[close_str.into()], ""))?;
        bld!(self.builder.build_call(release, &[close_str.into()], ""))?;

        Ok(())
    }

    pub(crate) fn compile_print(
        &mut self,
        val: BasicValueEnum<'ctx>,
        kind: ValKind,
    ) -> Result<(), CodeGenError> {
        match kind {
            ValKind::Str => {
                let pf = self.rt("ore_str_print")?;
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
            ValKind::Bool => {
                let pf = self.rt("ore_print_bool")?;
                let int_val = val.into_int_value();
                let ext = {
                    let bw = int_val.get_type().get_bit_width();
                    if bw < 8 {
                        bld!(self.builder.build_int_z_extend(int_val, self.context.i8_type(), "zext"))?
                    } else if bw > 8 {
                        bld!(self.builder.build_int_truncate(int_val, self.context.i8_type(), "trunc"))?
                    } else {
                        int_val
                    }
                };
                bld!(self.builder.build_call(pf, &[ext.into()], ""))?;
            }
            ValKind::Float => {
                let pf = self.rt("ore_print_float")?;
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
            ValKind::List(_) => {
                let pf = self.rt("ore_list_print")?;
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
            ValKind::Map => {
                let pf = self.rt("ore_map_print")?;
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
            ValKind::Record(ref name) => {
                let s = self.record_to_str(val, name)?;
                let pf = self.rt("ore_str_print")?;
                bld!(self.builder.build_call(pf, &[s.into()], ""))?;
                let release = self.rt("ore_str_release")?;
                bld!(self.builder.build_call(release, &[s.into()], ""))?;
            }
            ValKind::Enum(ref name) => {
                let s = self.enum_to_str(val, name)?;
                let pf = self.rt("ore_str_print")?;
                bld!(self.builder.build_call(pf, &[s.into()], ""))?;
                let release = self.rt("ore_str_release")?;
                bld!(self.builder.build_call(release, &[s.into()], ""))?;
            }
            _ => {
                let pf = self.rt("ore_print_int")?;
                bld!(self.builder.build_call(pf, &[val.into()], ""))?;
            }
        }
        Ok(())
    }

}
