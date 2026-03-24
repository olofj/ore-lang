//! Direct LLVM IR generation for enum-derived methods that can't be expressed
//! cleanly as AST (specifically enum Eq, which requires nested match and causes
//! LLVM register allocation issues).

use super::*;
use inkwell::IntPredicate;
use inkwell::values::BasicValueEnum;

impl<'ctx> CodeGen<'ctx> {
    /// Generate and compile the `TypeName_eq(self, other) -> Bool` function
    /// for an enum type directly as LLVM IR.
    ///
    /// Strategy:
    /// 1. Compare tags; if different, return false
    /// 2. If tags match, switch on tag and compare payload fields
    pub(crate) fn compile_enum_eq_derive(
        &mut self,
        enum_name: &str,
    ) -> Result<(), CodeGenError> {
        let info = self.enums.get(enum_name)
            .ok_or_else(|| self.err(format!("enum '{}' not registered", enum_name)))?;
        let enum_type = info.enum_type;
        let variants: Vec<_> = info.variants.iter().map(|v| {
            (v.name.clone(), v.tag, v.payload_type, v.field_names.clone(), v.field_kinds.clone())
        }).collect();

        let fn_name = format!("{}_eq", enum_name);

        // Declare the function: fn(enum_type, enum_type) -> bool
        let bool_type = self.context.bool_type();
        let fn_type = bool_type.fn_type(
            &[enum_type.into(), enum_type.into()],
            false,
        );
        let func = self.module.add_function(&fn_name, fn_type, None);
        self.functions.insert(fn_name.clone(), (func, ValKind::Bool));

        // Create entry block
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        // Store params to allocas
        let self_val = func.get_nth_param(0).unwrap();
        let other_val = func.get_nth_param(1).unwrap();

        let self_alloca = bld!(self.builder.build_alloca(enum_type, "self_alloca"))?;
        bld!(self.builder.build_store(self_alloca, self_val))?;
        let other_alloca = bld!(self.builder.build_alloca(enum_type, "other_alloca"))?;
        bld!(self.builder.build_store(other_alloca, other_val))?;

        // Load tags
        let self_tag = self.load_tag(enum_type, self_alloca)?;
        let other_tag = self.load_tag(enum_type, other_alloca)?;

        // Compare tags
        let tags_equal = bld!(self.builder.build_int_compare(
            IntPredicate::EQ, self_tag, other_tag, "tags_eq"
        ))?;

        let tags_differ_bb = self.context.append_basic_block(func, "tags_differ");
        let tags_match_bb = self.context.append_basic_block(func, "tags_match");
        let merge_bb = self.context.append_basic_block(func, "merge");

        bld!(self.builder.build_conditional_branch(tags_equal, tags_match_bb, tags_differ_bb))?;

        // Tags differ -> return false
        self.builder.position_at_end(tags_differ_bb);
        let false_val = bool_type.const_int(0, false);
        bld!(self.builder.build_unconditional_branch(merge_bb))?;

        // Tags match -> compare payloads per variant
        self.builder.position_at_end(tags_match_bb);

        if variants.is_empty() || variants.iter().all(|v| v.3.is_empty()) {
            // No variants or all variants have no payload -> always equal when tags match
            let true_val = bool_type.const_int(1, false);
            bld!(self.builder.build_unconditional_branch(merge_bb))?;

            // Build phi
            self.builder.position_at_end(merge_bb);
            let phi = bld!(self.builder.build_phi(bool_type, "result"))?;
            phi.add_incoming(&[
                (&false_val, tags_differ_bb),
                (&true_val, tags_match_bb),
            ]);
            bld!(self.builder.build_return(Some(&phi.as_basic_value())))?;
        } else {
            // Switch on self_tag to compare variant-specific payloads
            let default_bb = self.context.append_basic_block(func, "default");
            let mut case_blocks = Vec::new();
            let mut variant_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();

            for (_, vtag, payload_type, field_names, field_kinds) in &variants {
                let case_bb = self.context.append_basic_block(func, &format!("case_{}", vtag));
                let tag_const = self.context.i8_type().const_int(*vtag as u64, false);
                case_blocks.push((tag_const, case_bb));

                self.builder.position_at_end(case_bb);

                if field_names.is_empty() {
                    // No payload, tags already match -> true
                    let true_val: BasicValueEnum = bool_type.const_int(1, false).into();
                    bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    variant_results.push((true_val, case_bb));
                } else {
                    // Get data pointers
                    let self_data_ptr = bld!(self.builder.build_struct_gep(
                        enum_type, self_alloca, 1, "self_data"
                    ))?;
                    let self_payload_ptr = bld!(self.builder.build_pointer_cast(
                        self_data_ptr, self.ptr_type(), "self_payload"
                    ))?;

                    let other_data_ptr = bld!(self.builder.build_struct_gep(
                        enum_type, other_alloca, 1, "other_data"
                    ))?;
                    let other_payload_ptr = bld!(self.builder.build_pointer_cast(
                        other_data_ptr, self.ptr_type(), "other_payload"
                    ))?;

                    // Compare each field
                    let mut all_eq: inkwell::values::IntValue<'ctx> = bool_type.const_int(1, false);

                    for (i, (_fname, fkind)) in field_names.iter().zip(field_kinds.iter()).enumerate() {
                        let field_ty = self.kind_to_llvm_type(fkind);

                        let self_field_ptr = bld!(self.builder.build_struct_gep(
                            *payload_type, self_payload_ptr, i as u32, "sf"
                        ))?;
                        let self_field = bld!(self.builder.build_load(
                            field_ty, self_field_ptr, "sf_val"
                        ))?;

                        let other_field_ptr = bld!(self.builder.build_struct_gep(
                            *payload_type, other_payload_ptr, i as u32, "of"
                        ))?;
                        let other_field = bld!(self.builder.build_load(
                            field_ty, other_field_ptr, "of_val"
                        ))?;

                        let field_eq = self.compile_field_eq(self_field, other_field, fkind)?;
                        all_eq = bld!(self.builder.build_and(all_eq, field_eq, "and_eq"))?;
                    }

                    let end_bb = self.current_block()?;
                    bld!(self.builder.build_unconditional_branch(merge_bb))?;
                    variant_results.push((all_eq.into(), end_bb));
                }
            }

            // Default case (shouldn't happen) -> true
            self.builder.position_at_end(default_bb);
            let true_val: BasicValueEnum = bool_type.const_int(1, false).into();
            bld!(self.builder.build_unconditional_branch(merge_bb))?;

            // Build switch
            self.builder.position_at_end(tags_match_bb);
            bld!(self.builder.build_switch(
                self_tag,
                default_bb,
                &case_blocks.iter().map(|(v, bb)| (*v, *bb)).collect::<Vec<_>>()
            ))?;

            // Build phi at merge
            self.builder.position_at_end(merge_bb);
            let phi = bld!(self.builder.build_phi(bool_type, "result"))?;
            phi.add_incoming(&[(&false_val, tags_differ_bb)]);
            phi.add_incoming(&[(&true_val, default_bb)]);
            for (val, bb) in &variant_results {
                phi.add_incoming(&[(val, *bb)]);
            }
            bld!(self.builder.build_return(Some(&phi.as_basic_value())))?;
        }

        Ok(())
    }

    /// Compare two values for equality, returning an i1 bool.
    fn compile_field_eq(
        &self,
        a: BasicValueEnum<'ctx>,
        b: BasicValueEnum<'ctx>,
        kind: &ValKind,
    ) -> Result<inkwell::values::IntValue<'ctx>, CodeGenError> {
        match kind {
            ValKind::Str => {
                let i8_val = self.call_rt(
                    "ore_str_eq",
                    &[a.into(), b.into()],
                    "seq",
                )?.into_int_value();
                self.i8_to_bool(i8_val)
            }
            ValKind::Float => {
                Ok(bld!(self.builder.build_float_compare(
                    inkwell::FloatPredicate::OEQ,
                    a.into_float_value(),
                    b.into_float_value(),
                    "feq"
                ))?)
            }
            ValKind::Bool => {
                Ok(bld!(self.builder.build_int_compare(
                    IntPredicate::EQ,
                    a.into_int_value(),
                    b.into_int_value(),
                    "beq"
                ))?)
            }
            _ => {
                // Int and other types: integer comparison
                Ok(bld!(self.builder.build_int_compare(
                    IntPredicate::EQ,
                    a.into_int_value(),
                    b.into_int_value(),
                    "ieq"
                ))?)
            }
        }
    }
}
