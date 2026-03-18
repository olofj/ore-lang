// Runtime declarations
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

// Tagged union for Option/Result
typedef struct { int8_t tag; int64_t value; } OreTaggedUnion;

extern void ore_print_int(int64_t);
extern void ore_print_bool(int8_t);
extern void ore_print_float(double);
extern void ore_print_int_no_newline(int64_t);
extern void ore_print_float_no_newline(double);
extern void ore_print_bool_no_newline(int8_t);
extern void* ore_str_new(const char*, int32_t);
extern void* ore_str_concat(void*, void*);
extern void ore_str_print(void*);
extern void ore_str_print_no_newline(void*);
extern void ore_str_retain(void*);
extern void ore_str_release(void*);
extern void* ore_int_to_str(int64_t);
extern void* ore_bool_to_str(int8_t);
extern void* ore_float_to_str(double);
extern void* ore_dynamic_to_str(int64_t, int8_t);
extern int64_t ore_str_len(void*);
extern int8_t ore_str_eq(void*, void*);
extern int64_t ore_str_cmp(void*, void*);
extern int8_t ore_str_contains(void*, void*);
extern void* ore_str_trim(void*);
extern void* ore_str_split(void*, void*);
extern void* ore_str_replace(void*, void*, void*);
extern int8_t ore_str_starts_with(void*, void*);
extern int8_t ore_str_ends_with(void*, void*);
extern void* ore_str_to_upper(void*);
extern void* ore_str_to_lower(void*);
extern void* ore_str_substr(void*, int64_t, int64_t);
extern void* ore_str_chars(void*);
extern void* ore_str_repeat(void*, int64_t);
extern int64_t ore_str_to_int(void*);
extern double ore_str_to_float(void*);
extern void* ore_str_reverse(void*);
extern void* ore_str_char_at(void*, int64_t);
extern int64_t ore_ord(void*);
extern void* ore_chr(int64_t);
extern void ore_assert(int8_t, void*, int64_t);
extern void ore_assert_eq_int(int64_t, int64_t, void*, int64_t);
extern void ore_assert_eq_float(double, double, void*, int64_t);
extern void ore_assert_eq_str(void*, void*, void*, int64_t);
extern void ore_assert_ne_int(int64_t, int64_t, void*, int64_t);
extern void ore_assert_ne_str(void*, void*, void*, int64_t);
extern void* ore_list_new(void);
extern void ore_list_push(void*, int64_t);
extern int64_t ore_list_pop(void*);
extern void ore_list_clear(void*);
extern int64_t ore_list_get(void*, int64_t);
extern int8_t ore_list_get_kind(void*, int64_t);
extern void ore_list_set(void*, int64_t, int64_t);
extern int64_t ore_list_len(void*);
extern void ore_list_print(void*);
extern void ore_list_print_str(void*);
extern void ore_list_print_float(void*);
extern void ore_list_print_bool(void*);
extern void* ore_list_sort(void*);
extern void ore_list_reverse(void*);
extern int8_t ore_list_contains(void*, int64_t);
extern int8_t ore_list_contains_str(void*, void*);
extern void* ore_list_concat(void*, void*);
extern void* ore_list_join(void*, void*);
extern void* ore_list_join_str(void*, void*);
extern void* ore_list_take(void*, int64_t);
extern void* ore_list_skip(void*, int64_t);
extern void* ore_list_slice(void*, int64_t, int64_t);
extern void* ore_list_flatten(void*);
extern void* ore_list_unique(void*);
extern int64_t ore_list_sum(void*);
extern int64_t ore_list_min(void*);
extern int64_t ore_list_max(void*);
extern double ore_list_average(void*);
extern void* ore_list_map(void*, void*, void*);
extern void* ore_list_filter(void*, void*, void*);
extern void ore_list_each(void*, void*, void*);
extern int64_t ore_list_fold(void*, int64_t, void*, void*);
extern int64_t ore_list_reduce1(void*, void*, void*);
extern int8_t ore_list_any(void*, void*, void*);
extern int8_t ore_list_all(void*, void*, void*);
extern void* ore_list_enumerate(void*);
extern void* ore_range(int64_t, int64_t);
extern void* ore_range_step(int64_t, int64_t, int64_t);
extern void* ore_map_new(void);
extern void ore_map_set(void*, void*, int64_t);
extern int64_t ore_map_get(void*, void*);
extern int64_t ore_map_get_or(void*, void*, int64_t);
extern int8_t ore_map_contains(void*, void*);
extern int64_t ore_map_len(void*);
extern int64_t ore_map_remove(void*, void*);
extern void* ore_map_keys(void*);
extern void* ore_map_values(void*);
extern void* ore_map_entries(void*);
extern void ore_map_print(void*);
extern void* ore_map_merge(void*, void*);
extern void ore_map_clear(void*);
extern int64_t ore_int_pow(int64_t, int64_t);
extern double ore_math_sqrt(double);
extern double ore_math_sin(double);
extern double ore_math_cos(double);
extern double ore_math_log(double);
extern double ore_math_pow(double, double);
extern double ore_math_abs(double);
extern double ore_math_floor(double);
extern double ore_math_ceil(double);
extern double ore_math_round(double);
extern double ore_math_pi(void);
extern double ore_math_e(void);
extern double ore_float_round_to(double, int64_t);
extern void* ore_float_format(double, int64_t);
extern void* ore_readln(void);
extern void* ore_file_read(void*);
extern void* ore_file_read_lines(void*);
extern int8_t ore_file_write(void*, void*);
extern int8_t ore_file_exists(void*);
extern void* ore_args(void);
extern void ore_exit(int64_t);
extern void* ore_exec(void*);
extern void* ore_type_of(int8_t);
extern void* ore_env_get(void*);
extern void ore_env_set(void*, void*);
extern int64_t ore_time_now(void);
extern int64_t ore_time_ms(void);
extern int64_t ore_rand_int(int64_t, int64_t);
extern void ore_spawn(void*);
extern void ore_spawn_with_arg(void*, int64_t);
extern void ore_thread_join_all(void);
extern void ore_sleep(int64_t);
extern void* ore_channel_new(void);
extern void ore_channel_send(void*, int64_t);
extern int64_t ore_channel_recv(void*);
extern void ore_div_by_zero(void);

// Enum: Token
struct ore_enum_Token { int8_t tag; int64_t data[1]; };

// Record: SpannedToken
struct ore_rec_SpannedToken { struct ore_enum_Token token; int64_t line; int64_t col; };

// Record: LexerState
struct ore_rec_LexerState { void* source; int64_t pos; int64_t line; int64_t col; };

// Enum: TypeExpr
struct ore_enum_TypeExpr { int8_t tag; int64_t data[2]; };

// Enum: BinOp
struct ore_enum_BinOp { int8_t tag; };

// Enum: StringPart
struct ore_enum_StringPart { int8_t tag; int64_t data[1]; };

// Enum: Pattern
struct ore_enum_Pattern { int8_t tag; int64_t data[2]; };

// Record: MatchArm
struct ore_rec_MatchArm { struct ore_enum_Pattern pattern; int64_t guard; int64_t body; };

// Record: ParamDef
struct ore_rec_ParamDef { void* name; struct ore_enum_TypeExpr ty; int64_t default_expr; };

// Record: FieldDef
struct ore_rec_FieldDef { void* name; struct ore_enum_TypeExpr ty; };

// Record: TypeParamDef
struct ore_rec_TypeParamDef { void* name; void* bound; };

// Record: VariantDef
struct ore_rec_VariantDef { void* name; void* fields; };

// Record: SpannedStmt
struct ore_rec_SpannedStmt { int64_t stmt_id; int64_t line; };

// Record: Block
struct ore_rec_Block { void* stmts; };

// Enum: Expr
struct ore_enum_Expr { int8_t tag; int64_t data[4]; };

// Enum: Stmt
struct ore_enum_Stmt { int8_t tag; int64_t data[5]; };

// Record: FnDef
struct ore_rec_FnDef { void* name; void* type_params; void* params; struct ore_enum_TypeExpr ret_type; struct ore_rec_Block body; };

// Record: EnumDefNode
struct ore_rec_EnumDefNode { void* name; void* variants; };

// Record: TypeDefNode
struct ore_rec_TypeDefNode { void* name; void* type_params; void* fields; };

// Record: TraitMethodDef
struct ore_rec_TraitMethodDef { void* name; void* params; struct ore_enum_TypeExpr ret_type; };

// Record: TraitDefNode
struct ore_rec_TraitDefNode { void* name; void* methods; };

// Enum: Item
struct ore_enum_Item { int8_t tag; int64_t data[3]; };

// Record: Program
struct ore_rec_Program { void* items; };

// Record: ParseResult
struct ore_rec_ParseResult { struct ore_rec_Program program; void* exprs; void* stmts; };

// Enum: OreType
struct ore_enum_OreType { int8_t tag; int64_t data[2]; };

// Forward declarations
int8_t is_layout(struct ore_enum_Token tok);
int8_t is_keyword(struct ore_enum_Token tok);
int64_t push_token(void* pool, struct ore_enum_Token t);
struct ore_enum_Token get_token(void* pool, int64_t idx);
void* token_to_str(struct ore_enum_Token tok);
struct ore_rec_LexerState new_lexer(void* source);
int8_t has_more(struct ore_rec_LexerState ls);
int64_t peek_char(struct ore_rec_LexerState ls);
int64_t peek_char2(struct ore_rec_LexerState ls);
int8_t is_digit(int64_t c);
int8_t is_alpha(int64_t c);
int8_t is_alnum(int64_t c);
int8_t is_space(int64_t c);
OreTaggedUnion lex(void* source);
struct ore_rec_LexerState advance(struct ore_rec_LexerState ls);
int8_t last_is_layout(void* tokens);
int8_t is_pipe_tok(struct ore_enum_Token t);
int8_t is_dot_tok(struct ore_enum_Token t);
OreTaggedUnion handle_indentation(struct ore_rec_LexerState ls, void* tokens, void* indent_stack);
struct ore_rec_LexerState lex_token(struct ore_rec_LexerState ls, void* tokens, void* errors);
struct ore_rec_LexerState lex_number(struct ore_rec_LexerState ls, void* tokens);
struct ore_rec_LexerState lex_string(struct ore_rec_LexerState ls, void* tokens, void* errors);
struct ore_rec_LexerState lex_triple_string(struct ore_rec_LexerState ls, void* tokens, void* errors, int64_t start_line, int64_t start_col);
struct ore_rec_LexerState lex_ident(struct ore_rec_LexerState ls, void* tokens);
struct ore_enum_Token keyword_lookup(void* text);
void* fixup_multiline_pipes(void* tokens);
void* lex_split(void* source);
int64_t alloc_expr(void* pool, struct ore_enum_Expr e);
struct ore_enum_Expr get_expr(void* pool, int64_t id);
int64_t alloc_stmt(void* pool, struct ore_enum_Stmt s);
struct ore_enum_Stmt get_stmt(void* pool, int64_t id);
int64_t alloc_pattern(void* pool, struct ore_enum_Pattern p);
struct ore_enum_Pattern get_pattern(void* pool, int64_t id);
int64_t alloc_item(void* pool, struct ore_enum_Item i);
struct ore_enum_Item get_item(void* pool, int64_t id);
int64_t alloc_type(void* pool, struct ore_enum_TypeExpr t);
struct ore_enum_TypeExpr get_type(void* pool, int64_t id);
struct ore_rec_SpannedStmt get_sstmt(void* pool, int64_t id);
int64_t no_node();
int8_t has_node(int64_t id);
struct ore_enum_StringPart get_string_part(void* pool, int64_t id);
struct ore_rec_MatchArm get_match_arm(void* pool, int64_t id);
struct ore_rec_FnDef get_fn_def(void* pool, int64_t id);
struct ore_rec_FieldDef get_field_def(void* pool, int64_t id);
struct ore_rec_VariantDef get_variant_def(void* pool, int64_t id);
struct ore_rec_ParamDef get_param_def(void* pool, int64_t id);
struct ore_rec_TypeParamDef get_type_param_def(void* pool, int64_t id);
void* s_get_list(void* s, int64_t idx);
int64_t s_pos(void* s);
int64_t s_set_pos(void* s, int64_t val);
void* s_tokens(void* s);
void* s_exprs(void* s);
void* s_stmts(void* s);
void* s_errors(void* s);
void* s_items(void* s);
void* s_types(void* s);
void* s_patterns(void* s);
int8_t s_has_error(void* s);
int64_t s_error(void* s, void* msg);
int64_t s_alloc_expr(void* s, struct ore_enum_Expr e);
int64_t s_alloc_stmt(void* s, struct ore_enum_Stmt st);
int64_t s_alloc_item(void* s, struct ore_enum_Item it);
int64_t s_alloc_type(void* s, struct ore_enum_TypeExpr te);
int64_t s_alloc_pat(void* s, struct ore_enum_Pattern p);
struct ore_enum_Token p_peek(void* s);
int64_t p_peek_line(void* s);
int64_t p_peek_col(void* s);
struct ore_enum_Token p_peek_at(void* s, int64_t offset);
int64_t p_skip(void* s);
int8_t p_is_eof(struct ore_enum_Token tok);
int8_t p_is_dedent(struct ore_enum_Token tok);
int8_t p_is_newline(struct ore_enum_Token tok);
int8_t p_is_indent(struct ore_enum_Token tok);
int8_t p_at_block_end(void* s);
int64_t p_check_delim(struct ore_enum_Token tok, void* tag);
int64_t p_check_op(struct ore_enum_Token tok, void* tag);
int64_t p_check_kw1(struct ore_enum_Token tok, void* tag);
int64_t p_check_kw2(struct ore_enum_Token tok, void* tag);
int8_t p_check_tok(struct ore_enum_Token tok, void* tag);
int8_t p_at(void* s, void* tag);
int64_t p_expect(void* s, void* tag);
void* p_expect_ident(void* s, void* ctx);
int64_t p_skip_nl(void* s);
int64_t p_skip_ws(void* s);
int8_t is_compound(struct ore_enum_Token t);
struct ore_enum_BinOp compound_op(struct ore_enum_Token t);
int64_t tok_to_op(struct ore_enum_Token tok);
struct ore_enum_BinOp int_to_op(int64_t n);
int64_t bp_l(int64_t n);
int64_t bp_r(int64_t n);
int64_t parse_type_expr(void* s);
int64_t parse_param(void* s, void* out);
void* parse_call_args(void* s);
int64_t parse_block(void* s, void* out);
void* parse_optional_msg(void* s);
int64_t parse_stmt(void* s);
int64_t parse_for_stmt(void* s);
int64_t parse_ident_stmt(void* s);
int64_t parse_expr(void* s, int64_t min_bp);
int64_t parse_prefix(void* s);
int64_t parse_string_interp(void* s, void* start);
int64_t parse_if_expr(void* s);
int64_t parse_paren_expr(void* s);
int64_t try_lambda(void* s);
int64_t parse_lambda_body(void* s);
int64_t parse_list_lit(void* s);
int64_t parse_map_lit(void* s);
int64_t parse_ident_expr(void* s);
void* parse_match_arms(void* s);
struct ore_rec_MatchArm parse_match_arm(void* s);
int64_t parse_pattern(void* s);
int64_t parse_variant_pat(void* s, void* name);
int64_t parse_fn_def(void* s, void* out);
int64_t parse_type_or_enum(void* s);
int64_t parse_use_item(void* s);
int64_t parse_test_def(void* s);
int64_t parse_item(void* s);
void* s_lines(void* s);
void* s_cols(void* s);
void* make_parse_state_split(void* split_data);
int8_t parse_to_lists(void* split_data, void* result_holder);
struct ore_rec_Program parse(void* split_data);
void* type_to_str(struct ore_enum_OreType t);
struct ore_enum_OreType type_expr_to_ore_type(struct ore_enum_TypeExpr te);
int8_t types_equal(struct ore_enum_OreType a, struct ore_enum_OreType b);
struct ore_rec_VariantDef get_variant_def(void* pool, int64_t idx);
struct ore_rec_ParamDef get_param_def(void* pool, int64_t idx);
struct ore_rec_FnDef get_fn_def(void* pool, int64_t idx);
struct ore_rec_SpannedStmt get_sspanned(void* pool, int64_t idx);
struct ore_rec_MatchArm get_match_arm_typed(void* pool, int64_t idx);
struct ore_enum_Item get_item_typed(void* pool, int64_t idx);
struct ore_enum_StringPart get_string_part(void* pool, int64_t idx);
void* int_to_str(int64_t n);
void* make_method_set();
void* make_builtin_set();
int64_t scope_push(void* scopes);
int64_t scope_pop(void* scopes);
int64_t scope_define(void* scopes, void* name, void* type_name);
void* scope_lookup(void* scopes, void* name);
void* tc_new();
void* tc_fns(void* tc);
void* tc_fn_req(void* tc);
void* tc_records(void* tc);
void* tc_enums(void* tc);
void* tc_variant_to_enum(void* tc);
void* tc_errors(void* tc);
void* tc_scopes(void* tc);
void* tc_methods(void* tc);
void* tc_builtins(void* tc);
int64_t tc_add_error(void* tc, void* msg);
int8_t is_line_end(struct ore_enum_Token t);
int8_t is_colon(struct ore_enum_Token t);
void* token_ident_name(struct ore_enum_Token t);
int64_t count_fn_params(void* tokens, int64_t start, int64_t len);
int64_t collect_fn_defs(void* tc, void* tokens);
int64_t collect_type_defs(void* tc, void* tokens);
int64_t collect_enum_variants(void* tc, void* tokens, int64_t start, int64_t len, void* tname);
int8_t is_known_name(void* tc, void* name);
int64_t check_call(void* tc, int64_t func_id, void* call_args, void* exprs, void* stmts);
int64_t check_call_target(void* tc, struct ore_enum_Expr func_e, void* call_args);
int64_t check_call_arity(void* tc, void* fname, int64_t nargs);
int64_t check_ident(void* tc, void* name);
int64_t check_expr(void* tc, int64_t expr_id, void* exprs, void* stmts);
int64_t check_string_interp(void* tc, void* parts, void* exprs, void* stmts);
int64_t check_match_arms(void* tc, void* arms, void* exprs, void* stmts);
int64_t add_pattern_bindings(void* tc, struct ore_enum_Pattern pat);
int64_t check_stmt(void* tc, int64_t stmt_id, void* exprs, void* stmts);
int64_t check_block(void* tc, struct ore_rec_Block block, void* exprs, void* stmts);
int64_t check_all_stmts(void* tc, void* exprs, void* stmts);
void* typecheck_with_scopes(void* tokens, void* exprs, void* stmts);
void* typecheck(void* tokens, void* exprs, void* stmts);
int64_t add_fn_params_from_tokens(void* tc, void* tokens);
int64_t add_match_arm_bindings(void* scopes, void* tokens, int64_t arrow_pos);
int64_t add_params_for_fn(void* scopes, void* tokens, int64_t start, int64_t len);
void* cg_new();
void* cg_list(void* st, int64_t idx);
void* str_at(void* lst, int64_t idx);
void* cg_lines(void* st);
int64_t cg_indent(void* st);
int64_t cg_set_indent(void* st, int64_t val);
int64_t cg_inc_indent(void* st);
int64_t cg_dec_indent(void* st);
void* cg_tmp(void* st);
void* cg_label(void* st, void* prefix);
void* cg_errors(void* st);
int64_t cg_error(void* st, void* msg);
void* indent_str(int64_t n);
int64_t emit(void* st, void* line);
int64_t emit_raw(void* st, void* line);
int64_t cg_set_var(void* st, void* name, void* kind);
void* cg_get_var_kind(void* st, void* name);
int8_t cg_has_var(void* st, void* name);
int64_t cg_set_fn(void* st, void* name, void* ret_kind);
void* cg_get_fn_ret(void* st, void* name);
int8_t cg_has_fn(void* st, void* name);
int64_t cg_add_generic_fn(void* st, void* name, struct ore_rec_FnDef fd);
void* cg_get_generic_fn(void* st, void* name);
int8_t cg_has_generic_fn(void* st, void* name);
int8_t cg_has_mono(void* st, void* mono_name);
int64_t cg_add_mono(void* st, void* mono_name);
int64_t cg_add_record(void* st, void* name, void* fields_str, int64_t count, void* field_kinds_str);
int8_t cg_is_record(void* st, void* name);
void* cg_get_record_field_kind(void* st, void* rec_name, void* field);
int64_t cg_add_enum(void* st, void* name, void* variants_str, int64_t count);
int8_t cg_is_enum(void* st, void* name);
int64_t cg_add_variant_map(void* st, void* variant, void* enum_name);
void* cg_variant_enum(void* st, void* variant);
void* kind_to_c_type(void* kind);
void* value_to_i64_expr(void* st, void* val, void* kind);
void* coerce_from_i64_expr(void* val, void* kind);
void* type_expr_to_kind_str(void* st, struct ore_enum_TypeExpr te);
void* kind_to_suffix(void* kind);
void* kind_str_to_type_name(void* kind);
int8_t is_c_reserved(void* name);
void* mangle_fn(void* name);
void* mangle_var(void* name);
void* c_escape(void* s);
int64_t emit_runtime_decls(void* st);
void* compile_expr(void* st, void* exprs, void* stmts, int64_t expr_id);
void* compile_expr_node(void* st, void* exprs, void* stmts, struct ore_enum_Expr e);
void* compile_string_lit(void* st, void* s);
void* compile_ident(void* st, void* name);
void* compile_variant_zero_arg(void* st, void* variant, void* enum_name);
int64_t find_variant_tag(void* st, void* enum_name, void* variant);
void* compile_binop(void* st, void* exprs, void* stmts, struct ore_enum_BinOp op, int64_t left, int64_t right);
void* compile_binop_values(void* st, struct ore_enum_BinOp op, void* l, void* r);
void* binop_to_c(void* st, struct ore_enum_BinOp op, int8_t is_float, void* lc, void* rc);
void* compile_str_binop(void* st, struct ore_enum_BinOp op, void* lv, void* rv);
void* compile_pipe(void* st, void* exprs, void* stmts, int64_t left, int64_t right);
void* compile_call(void* st, void* exprs, void* stmts, int64_t func, void* args);
void* compile_fn_call_with_args(void* st, void* fname, void* arg_strs, void* arg_kinds);
void* compile_generic_call(void* st, void* fname, void* arg_strs, void* arg_kinds);
void* resolve_type_param(void* name, void* keys, void* vals);
int8_t list_contains_str(void* lst, void* s);
void* compile_variant_call(void* st, void* variant, void* enum_name, void* arg_strs, void* arg_kinds);
void* try_builtin(void* st, void* fname, void* args, void* kinds);
void* compile_print(void* st, void* exprs, void* stmts, int64_t inner);
void* compile_if_else(void* st, void* exprs, void* stmts, int64_t cond, struct ore_rec_Block then_block, struct ore_rec_Block else_block);
void* compile_block(void* st, void* exprs, void* stmts, struct ore_rec_Block block);
void* compile_stmt_by_id(void* st, void* exprs, void* stmts, int64_t stmt_id);
void* compile_stmt_node(void* st, void* exprs, void* stmts, struct ore_enum_Stmt s);
void* compile_let(void* st, void* exprs, void* stmts, void* name, int8_t mutable, int64_t value);
void* compile_assign(void* st, void* exprs, void* stmts, void* name, int64_t value);
void* compile_index_assign(void* st, void* exprs, void* stmts, int64_t object, int64_t index, int64_t value);
void* compile_field_assign(void* st, void* exprs, void* stmts, int64_t object, void* field, int64_t value);
void* compile_for_in(void* st, void* exprs, void* stmts, void* var_name, int64_t start, int64_t end, int64_t step, struct ore_rec_Block body);
void* compile_for_each(void* st, void* exprs, void* stmts, void* var_name, int64_t iterable, struct ore_rec_Block body);
void* compile_while(void* st, void* exprs, void* stmts, int64_t cond, struct ore_rec_Block body);
void* compile_loop(void* st, void* exprs, void* stmts, struct ore_rec_Block body);
void* compile_string_interp(void* st, void* exprs, void* stmts, void* parts);
void* compile_list_lit(void* st, void* exprs, void* stmts, void* elements);
void* compile_map_lit(void* st, void* exprs, void* stmts, void* entries);
void* compile_index(void* st, void* exprs, void* stmts, int64_t object, int64_t index);
void* compile_field_access(void* st, void* exprs, void* stmts, int64_t object, void* field);
void* compile_method_call(void* st, void* exprs, void* stmts, int64_t object, void* method, void* args);
void* compile_str_method(void* st, void* obj, void* method, void* args, void* kinds);
void* compile_list_method(void* st, void* obj, void* method, void* args, void* kinds);
void* compile_map_method(void* st, void* obj, void* method, void* args, void* kinds);
void* compile_option_method(void* st, void* obj, void* kind, void* method, void* args, void* kinds);
void* compile_block_expr(void* st, void* exprs, void* stmts, struct ore_rec_Block block);
void* match_assign_expr(void* t, void* kind, void* val);
void* match_default_init(void* kind);
void* match_temp_c_type(void* kind);
void* compile_match(void* st, void* exprs, void* stmts, int64_t subject, void* arms);
int64_t collect_free_vars_expr(void* exprs, void* stmts, int64_t expr_id, void* bound, void* seen, void* ore_v_free);
int64_t collect_free_vars_stmt(void* exprs, void* stmts, int64_t stmt_id, void* bound, void* seen, void* ore_v_free);
int64_t collect_free_vars_block(void* exprs, void* stmts, struct ore_rec_Block block, void* bound, void* seen, void* ore_v_free);
int8_t list_contains_str(void* lst, void* needle);
void* copy_str_list(void* src);
void* find_free_vars(void* exprs, void* stmts, int64_t expr_id, void* params);
void* compile_lambda(void* st, void* exprs, void* stmts, void* params, int64_t body);
void* parse_closure_expr(void* cexpr);
void* compile_record_construct(void* st, void* exprs, void* stmts, void* type_name, void* fields);
int64_t compile_type_def(void* st, struct ore_rec_TypeDefNode td);
int64_t compile_enum_def(void* st, struct ore_rec_EnumDefNode ed);
int64_t compile_fn_def(void* st, void* exprs, void* stmts, struct ore_rec_FnDef fd);
struct ore_enum_TypeExpr resolve_self_type(struct ore_enum_TypeExpr te, void* type_name);
struct ore_rec_FnDef mangle_impl_method(void* type_name, struct ore_rec_FnDef method);
int64_t compile_impl_methods(void* st, void* exprs, void* stmts, void* type_name, void* methods);
int64_t compile_item(void* st, void* exprs, void* stmts, struct ore_enum_Item item);
void* generate(void* items, void* exprs, void* stmts);
void* dir_of_file(void* path);
void* resolve_use_path(void* base_dir, void* use_path);
void* parse_use_line(void* line);
void* resolve_imports(void* source, void* base_dir, void* loaded);
void* parse_source(void* source);
void* run_typecheck(void* parsed);
int64_t report_errors(void* errors, void* file);
void* find_runtime_lib();
void* c_output_path(void* file);
void* bin_output_path(void* file);
int64_t cmd_check(void* file);
int64_t cmd_run(void* file);
int64_t cmd_build(void* file, void* output);
int64_t print_usage();
int main();


int8_t is_layout(struct ore_enum_Token tok) {
    int8_t __tmp_0 = 0;
    if (tok.tag == 72) {
        __tmp_0 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 73) {
        __tmp_0 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 74) {
        __tmp_0 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_0 = (int8_t)(((int8_t)0));
    }
    return __tmp_0;
}

int8_t is_keyword(struct ore_enum_Token tok) {
    int8_t __tmp_1 = 0;
    if (tok.tag == 7) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 8) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 9) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 10) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 11) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 12) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 13) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 14) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 15) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 16) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 17) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 18) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 19) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 20) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 21) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 22) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 23) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 24) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 25) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 26) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 27) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 28) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 29) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 30) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 31) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 32) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 33) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 34) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else if (tok.tag == 35) {
        __tmp_1 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_1 = (int8_t)(((int8_t)0));
    }
    return __tmp_1;
}

int64_t push_token(void* pool, struct ore_enum_Token t) {
    ore_list_push(pool, ({ struct ore_enum_Token __v2i = t; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Token)), &__v2i, sizeof(struct ore_enum_Token)); }));
}

struct ore_enum_Token get_token(void* pool, int64_t idx) {
    int64_t __tmp_2 = ore_list_get(pool, idx);
    int8_t __tmp_3 = ore_list_get_kind(pool, idx);
    return *(struct ore_enum_Token*)(intptr_t)(__tmp_2);
}

void* token_to_str(struct ore_enum_Token tok) {
    void* __tmp_4 = 0;
    if (tok.tag == 0) {
        int64_t v = tok.data[0];
        __tmp_4 = (void*)(ore_str_concat(ore_str_concat(ore_str_new("Int(", 4), ore_int_to_str(v)), ore_str_new(")", 1)));
    }
    else if (tok.tag == 1) {
        int64_t v = tok.data[0];
        __tmp_4 = (void*)(ore_str_concat(ore_str_concat(ore_str_new("Float(", 6), ore_int_to_str(v)), ore_str_new(")", 1)));
    }
    else if (tok.tag == 2) {
        int64_t n = tok.data[0];
        __tmp_4 = (void*)(ore_str_concat(ore_str_concat(ore_str_new("Ident(", 6), ore_int_to_str(n)), ore_str_new(")", 1)));
    }
    else if (tok.tag == 3) {
        int64_t v = tok.data[0];
        __tmp_4 = (void*)(ore_str_concat(ore_str_concat(ore_str_new("String(", 7), ore_int_to_str(v)), ore_str_new(")", 1)));
    }
    else if (tok.tag == 4) {
        int64_t v = tok.data[0];
        __tmp_4 = (void*)(ore_str_concat(ore_str_concat(ore_str_new("StringStart(", 12), ore_int_to_str(v)), ore_str_new(")", 1)));
    }
    else if (tok.tag == 5) {
        int64_t v = tok.data[0];
        __tmp_4 = (void*)(ore_str_concat(ore_str_concat(ore_str_new("StringMid(", 10), ore_int_to_str(v)), ore_str_new(")", 1)));
    }
    else if (tok.tag == 6) {
        int64_t v = tok.data[0];
        __tmp_4 = (void*)(ore_str_concat(ore_str_concat(ore_str_new("StringEnd(", 10), ore_int_to_str(v)), ore_str_new(")", 1)));
    }
    else if (tok.tag == 7) {
        __tmp_4 = (void*)(ore_str_new("fn", 2));
    }
    else if (tok.tag == 8) {
        __tmp_4 = (void*)(ore_str_new("mut", 3));
    }
    else if (tok.tag == 9) {
        __tmp_4 = (void*)(ore_str_new("if", 2));
    }
    else if (tok.tag == 10) {
        __tmp_4 = (void*)(ore_str_new("then", 4));
    }
    else if (tok.tag == 11) {
        __tmp_4 = (void*)(ore_str_new("else", 4));
    }
    else if (tok.tag == 12) {
        __tmp_4 = (void*)(ore_str_new("true", 4));
    }
    else if (tok.tag == 13) {
        __tmp_4 = (void*)(ore_str_new("false", 5));
    }
    else if (tok.tag == 14) {
        __tmp_4 = (void*)(ore_str_new("return", 6));
    }
    else if (tok.tag == 15) {
        __tmp_4 = (void*)(ore_str_new("for", 3));
    }
    else if (tok.tag == 16) {
        __tmp_4 = (void*)(ore_str_new("while", 5));
    }
    else if (tok.tag == 17) {
        __tmp_4 = (void*)(ore_str_new("loop", 4));
    }
    else if (tok.tag == 18) {
        __tmp_4 = (void*)(ore_str_new("break", 5));
    }
    else if (tok.tag == 19) {
        __tmp_4 = (void*)(ore_str_new("in", 2));
    }
    else if (tok.tag == 20) {
        __tmp_4 = (void*)(ore_str_new("type", 4));
    }
    else if (tok.tag == 21) {
        __tmp_4 = (void*)(ore_str_new("impl", 4));
    }
    else if (tok.tag == 22) {
        __tmp_4 = (void*)(ore_str_new("trait", 5));
    }
    else if (tok.tag == 23) {
        __tmp_4 = (void*)(ore_str_new("Some", 4));
    }
    else if (tok.tag == 24) {
        __tmp_4 = (void*)(ore_str_new("None", 4));
    }
    else if (tok.tag == 25) {
        __tmp_4 = (void*)(ore_str_new("Ok", 2));
    }
    else if (tok.tag == 26) {
        __tmp_4 = (void*)(ore_str_new("Err", 3));
    }
    else if (tok.tag == 27) {
        __tmp_4 = (void*)(ore_str_new("use", 3));
    }
    else if (tok.tag == 28) {
        __tmp_4 = (void*)(ore_str_new("pub", 3));
    }
    else if (tok.tag == 29) {
        __tmp_4 = (void*)(ore_str_new("spawn", 5));
    }
    else if (tok.tag == 30) {
        __tmp_4 = (void*)(ore_str_new("match", 5));
    }
    else if (tok.tag == 31) {
        __tmp_4 = (void*)(ore_str_new("continue", 8));
    }
    else if (tok.tag == 32) {
        __tmp_4 = (void*)(ore_str_new("test", 4));
    }
    else if (tok.tag == 33) {
        __tmp_4 = (void*)(ore_str_new("assert", 6));
    }
    else if (tok.tag == 34) {
        __tmp_4 = (void*)(ore_str_new("assert_eq", 9));
    }
    else if (tok.tag == 35) {
        __tmp_4 = (void*)(ore_str_new("assert_ne", 9));
    }
    else if (tok.tag == 36) {
        __tmp_4 = (void*)(ore_str_new("+", 1));
    }
    else if (tok.tag == 37) {
        __tmp_4 = (void*)(ore_str_new("-", 1));
    }
    else if (tok.tag == 38) {
        __tmp_4 = (void*)(ore_str_new("*", 1));
    }
    else if (tok.tag == 39) {
        __tmp_4 = (void*)(ore_str_new("/", 1));
    }
    else if (tok.tag == 40) {
        __tmp_4 = (void*)(ore_str_new("%", 1));
    }
    else if (tok.tag == 41) {
        __tmp_4 = (void*)(ore_str_new(":=", 2));
    }
    else if (tok.tag == 42) {
        __tmp_4 = (void*)(ore_str_new("=", 1));
    }
    else if (tok.tag == 43) {
        __tmp_4 = (void*)(ore_str_new("==", 2));
    }
    else if (tok.tag == 44) {
        __tmp_4 = (void*)(ore_str_new("!=", 2));
    }
    else if (tok.tag == 45) {
        __tmp_4 = (void*)(ore_str_new("<", 1));
    }
    else if (tok.tag == 46) {
        __tmp_4 = (void*)(ore_str_new(">", 1));
    }
    else if (tok.tag == 47) {
        __tmp_4 = (void*)(ore_str_new("<=", 2));
    }
    else if (tok.tag == 48) {
        __tmp_4 = (void*)(ore_str_new(">=", 2));
    }
    else if (tok.tag == 49) {
        __tmp_4 = (void*)(ore_str_new("->", 2));
    }
    else if (tok.tag == 50) {
        __tmp_4 = (void*)(ore_str_new("=>", 2));
    }
    else if (tok.tag == 51) {
        __tmp_4 = (void*)(ore_str_new("|", 1));
    }
    else if (tok.tag == 52) {
        __tmp_4 = (void*)(ore_str_new(":", 1));
    }
    else if (tok.tag == 53) {
        __tmp_4 = (void*)(ore_str_new(".", 1));
    }
    else if (tok.tag == 54) {
        __tmp_4 = (void*)(ore_str_new("..", 2));
    }
    else if (tok.tag == 55) {
        __tmp_4 = (void*)(ore_str_new("?", 1));
    }
    else if (tok.tag == 56) {
        __tmp_4 = (void*)(ore_str_new("?.", 2));
    }
    else if (tok.tag == 57) {
        __tmp_4 = (void*)(ore_str_new("+=", 2));
    }
    else if (tok.tag == 58) {
        __tmp_4 = (void*)(ore_str_new("-=", 2));
    }
    else if (tok.tag == 59) {
        __tmp_4 = (void*)(ore_str_new("*=", 2));
    }
    else if (tok.tag == 60) {
        __tmp_4 = (void*)(ore_str_new("/=", 2));
    }
    else if (tok.tag == 61) {
        __tmp_4 = (void*)(ore_str_new("%=", 2));
    }
    else if (tok.tag == 62) {
        __tmp_4 = (void*)(ore_str_new("and", 3));
    }
    else if (tok.tag == 63) {
        __tmp_4 = (void*)(ore_str_new("or", 2));
    }
    else if (tok.tag == 64) {
        __tmp_4 = (void*)(ore_str_new("not", 3));
    }
    else if (tok.tag == 65) {
        __tmp_4 = (void*)(ore_str_new("(", 1));
    }
    else if (tok.tag == 66) {
        __tmp_4 = (void*)(ore_str_new(")", 1));
    }
    else if (tok.tag == 67) {
        __tmp_4 = (void*)(ore_str_new("{", 1));
    }
    else if (tok.tag == 68) {
        __tmp_4 = (void*)(ore_str_new("}", 1));
    }
    else if (tok.tag == 69) {
        __tmp_4 = (void*)(ore_str_new("[", 1));
    }
    else if (tok.tag == 70) {
        __tmp_4 = (void*)(ore_str_new("]", 1));
    }
    else if (tok.tag == 71) {
        __tmp_4 = (void*)(ore_str_new(",", 1));
    }
    else if (tok.tag == 72) {
        __tmp_4 = (void*)(ore_str_new("Newline", 7));
    }
    else if (tok.tag == 73) {
        __tmp_4 = (void*)(ore_str_new("Indent", 6));
    }
    else if (tok.tag == 74) {
        __tmp_4 = (void*)(ore_str_new("Dedent", 6));
    }
    else if (tok.tag == 75) {
        __tmp_4 = (void*)(ore_str_new("EOF", 3));
    }
    return __tmp_4;
}

struct ore_rec_LexerState new_lexer(void* source) {
    struct ore_rec_LexerState __tmp_5;
    __tmp_5.source = source;
    __tmp_5.pos = 0LL;
    __tmp_5.line = 1LL;
    __tmp_5.col = 1LL;
    return __tmp_5;
}

int8_t has_more(struct ore_rec_LexerState ls) {
    return (96070731123808 < 96070731126544);
}

int64_t peek_char(struct ore_rec_LexerState ls) {
    int64_t __tmp_6 = 0;
    if ((96070731134640 >= 96070731137376)) {
        __tmp_6 = (int64_t)((-(1LL)));
    } else {
        __tmp_6 = (int64_t)(ore_ord(ore_str_char_at(ls.source, ls.pos)));
    }
    return __tmp_6;
}

int64_t peek_char2(struct ore_rec_LexerState ls) {
    int64_t __tmp_7 = 0;
    if ((96070731156576 >= 96070731159344)) {
        __tmp_7 = (int64_t)((-(1LL)));
    } else {
        __tmp_7 = (int64_t)(ore_ord(ore_str_char_at(ls.source, (96070731166896 + 96070731167088))));
    }
    return __tmp_7;
}

int8_t is_digit(int64_t c) {
    return (96070731178480 && 96070731181008);
}

int8_t is_alpha(int64_t c) {
    return (96070731198704 || 96070731201232);
}

int8_t is_alnum(int64_t c) {
    return (96070731210928 || 96070731215392);
}

int8_t is_space(int64_t c) {
    return (96070731222976 || 96070731225504);
}

OreTaggedUnion lex(void* source) {
    struct ore_rec_LexerState ls = new_lexer(source);
    void* __tmp_8 = ore_list_new();
    void* tokens = __tmp_8;
    void* __tmp_9 = ore_list_new();
    ore_list_push(__tmp_9, (int64_t)(0LL));
    void* indent_stack = __tmp_9;
    int8_t at_line_start = ((int8_t)1);
    void* __tmp_10 = ore_list_new();
    void* errors = __tmp_10;
    while (1) {
        int64_t __tmp_11 = 0;
        if ((!(has_more(ls)))) {
            goto brk_0;
        } else {
        }
        int64_t __tmp_12 = 0;
        if (at_line_start) {
            OreTaggedUnion indent_result = handle_indentation(ls, tokens, indent_stack);
            int64_t __tmp_13 = 0;
            if (indent_result.tag == 0) {
                int64_t e = indent_result.value;
                OreTaggedUnion __tmp_14; __tmp_14.tag = 0; __tmp_14.value = (int64_t)(e);
                return __tmp_14;
            }
            else if (indent_result.tag == 1) {
                int64_t new_pos = indent_result.value;
                struct ore_rec_LexerState __tmp_15;
                __tmp_15.source = ls.source;
                __tmp_15.pos = new_pos;
                __tmp_15.line = ls.line;
                __tmp_15.col = ls.col;
                ls = __tmp_15;
            }
            at_line_start = ((int8_t)0);
            __tmp_12 = (int64_t)(__tmp_13);
        } else {
        }
        int64_t ch = peek_char(ls);
        int64_t __tmp_16 = 0;
        if ((96070652946256 == 96070731301856)) {
            goto brk_0;
        } else {
        }
        int64_t __tmp_17 = 0;
        if ((96070652950576 == 96070731306336)) {
            int64_t __tmp_18 = 0;
            if ((!(last_is_layout(tokens)))) {
                struct ore_rec_SpannedToken __tmp_19;
                struct ore_enum_Token __tmp_20; __tmp_20.tag = 72;
                __tmp_19.token = __tmp_20;
                __tmp_19.line = ls.line;
                __tmp_19.col = ls.col;
                ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_19; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
            } else {
            }
            ls = advance(ls);
            at_line_start = ((int8_t)1);
            __tmp_17 = (int64_t)(__tmp_18);
        } else {
            int64_t __tmp_21 = 0;
            if ((96070652990160 == 96070731343664)) {
                ls = advance(ls);
            } else {
                int64_t __tmp_22 = 0;
                if ((96070731354240 || 96070731356768)) {
                    ls = advance(ls);
                } else {
                    int64_t __tmp_23 = 0;
                    if ((96070731367648 && 96070731373200)) {
                        while ((96070731380224 && 96070731385744)) {
                            ls = advance(ls);
                            cont_3: ;
                        }
                        brk_2: ;
                    } else {
                        ls = lex_token(ls, tokens, errors);
                        int64_t __tmp_24 = 0;
                        if ((96070731408272 > 96070731408512)) {
                            int64_t __tmp_25 = ore_list_get(errors, 0LL);
                            int8_t __tmp_26 = ore_list_get_kind(errors, 0LL);
                            OreTaggedUnion __tmp_27; __tmp_27.tag = 0; __tmp_27.value = (int64_t)(__tmp_25);
                            return __tmp_27;
                        } else {
                        }
                        __tmp_23 = (int64_t)(__tmp_24);
                    }
                    __tmp_22 = (int64_t)(__tmp_23);
                }
                __tmp_21 = (int64_t)(__tmp_22);
            }
            __tmp_17 = (int64_t)(__tmp_21);
        }
        cont_1: ;
    }
    brk_0: ;
    while ((96070731427168 > 96070731427408)) {
        struct ore_rec_SpannedToken __tmp_28;
        struct ore_enum_Token __tmp_29; __tmp_29.tag = 74;
        __tmp_28.token = __tmp_29;
        __tmp_28.line = ls.line;
        __tmp_28.col = ls.col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_28; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        cont_5: ;
    }
    brk_4: ;
    struct ore_rec_SpannedToken __tmp_30;
    struct ore_enum_Token __tmp_31; __tmp_31.tag = 75;
    __tmp_30.token = __tmp_31;
    __tmp_30.line = ls.line;
    __tmp_30.col = ls.col;
    ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_30; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
    tokens = fixup_multiline_pipes(tokens);
    OreTaggedUnion __tmp_32; __tmp_32.tag = 1; __tmp_32.value = (int64_t)(tokens);
    return __tmp_32;
}

struct ore_rec_LexerState advance(struct ore_rec_LexerState ls) {
    int64_t ch = peek_char(ls);
    int64_t __tmp_33 = 0;
    if ((96070653131808 == 96070731497072)) {
        struct ore_rec_LexerState __tmp_34;
        __tmp_34.source = ls.source;
        __tmp_34.pos = (96070731504160 + 96070731504352);
        __tmp_34.line = (96070731507952 + 96070731508144);
        __tmp_34.col = 1LL;
        __tmp_33 = (int64_t)(__tmp_34);
    } else {
        struct ore_rec_LexerState __tmp_35;
        __tmp_35.source = ls.source;
        __tmp_35.pos = (96070731516480 + 96070731516672);
        __tmp_35.line = ls.line;
        __tmp_35.col = (96070731522944 + 96070731523136);
        __tmp_33 = (int64_t)(__tmp_35);
    }
    return __tmp_33;
}

int8_t last_is_layout(void* tokens) {
    int64_t __tmp_36 = 0;
    if ((96070731532000 == 96070731532240)) {
        __tmp_36 = (int64_t)(((int8_t)1));
    } else {
        int64_t tok = 0;
        int8_t __tmp_37 = 0;
        if (tok.token.tag == 72) {
            __tmp_37 = (int8_t)(((int8_t)1));
        }
        else if (tok.token.tag == 73) {
            __tmp_37 = (int8_t)(((int8_t)1));
        }
        else if (tok.token.tag == 74) {
            __tmp_37 = (int8_t)(((int8_t)1));
        }
        else {
            __tmp_37 = (int8_t)(((int8_t)0));
        }
        __tmp_36 = (int64_t)(__tmp_37);
    }
    return __tmp_36;
}

int8_t is_pipe_tok(struct ore_enum_Token t) {
    int8_t __tmp_38 = 0;
    if (t.tag == 51) {
        __tmp_38 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_38 = (int8_t)(((int8_t)0));
    }
    return __tmp_38;
}

int8_t is_dot_tok(struct ore_enum_Token t) {
    int8_t __tmp_39 = 0;
    if (t.tag == 53) {
        __tmp_39 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_39 = (int8_t)(((int8_t)0));
    }
    return __tmp_39;
}

OreTaggedUnion handle_indentation(struct ore_rec_LexerState ls, void* tokens, void* indent_stack) {
    int64_t pos = ls.pos;
    int64_t indent = 0LL;
    void* src = ls.source;
    while ((96070653279360 < 96070731626464)) {
        int64_t ch = ore_ord(ore_str_char_at(src, pos));
        int64_t __tmp_40 = 0;
        if ((96070653293184 == 96070731636656)) {
            indent = (96070653297424 + 96070731639792);
            pos = (96070653301360 + 96070731643936);
        } else {
            int64_t __tmp_41 = 0;
            if ((96070653304608 == 96070731648400)) {
                indent = (96070653308816 + 96070731651632);
                pos = (96070653312752 + 96070731655856);
            } else {
                goto brk_6;
            }
            __tmp_40 = (int64_t)(__tmp_41);
        }
        cont_7: ;
    }
    brk_6: ;
    int64_t __tmp_42 = 0;
    if ((96070653320704 >= 96070731664656)) {
        OreTaggedUnion __tmp_43; __tmp_43.tag = 1; __tmp_43.value = (int64_t)(pos);
        return __tmp_43;
    } else {
    }
    int64_t ch = ore_ord(ore_str_char_at(src, pos));
    int64_t __tmp_44 = 0;
    if ((96070731680064 || 96070731682592)) {
        OreTaggedUnion __tmp_45; __tmp_45.tag = 1; __tmp_45.value = (int64_t)(pos);
        return __tmp_45;
    } else {
    }
    int64_t __tmp_46 = 0;
    if ((96070731696576 && 96070731704336)) {
        OreTaggedUnion __tmp_47; __tmp_47.tag = 1; __tmp_47.value = (int64_t)(pos);
        return __tmp_47;
    } else {
    }
    int64_t current = 0;
    int64_t __tmp_48 = 0;
    if ((96070653373936 > 96070653375696)) {
        ore_list_push(indent_stack, (int64_t)(indent));
        struct ore_rec_SpannedToken __tmp_49;
        struct ore_enum_Token __tmp_50; __tmp_50.tag = 73;
        __tmp_49.token = __tmp_50;
        __tmp_49.line = ls.line;
        __tmp_49.col = 1LL;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_49; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
    } else {
        int64_t __tmp_51 = 0;
        if ((96070653399200 < 96070653400960)) {
            while ((96070653403584 < 96070731749520)) {
                struct ore_rec_SpannedToken __tmp_52;
                struct ore_enum_Token __tmp_53; __tmp_53.tag = 74;
                __tmp_52.token = __tmp_53;
                __tmp_52.line = ls.line;
                __tmp_52.col = 1LL;
                ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_52; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
                cont_9: ;
            }
            brk_8: ;
            int64_t __tmp_54 = 0;
            if ((96070653429072 != 96070731774960)) {
                OreTaggedUnion __tmp_55; __tmp_55.tag = 0; __tmp_55.value = (int64_t)(ore_str_concat(ore_str_new("inconsistent indentation at line ", 33), ore_int_to_str(ls.line)));
                return __tmp_55;
            } else {
            }
            __tmp_51 = (int64_t)(__tmp_54);
        } else {
        }
        __tmp_48 = (int64_t)(__tmp_51);
    }
    OreTaggedUnion __tmp_56; __tmp_56.tag = 1; __tmp_56.value = (int64_t)(pos);
    return __tmp_56;
}

struct ore_rec_LexerState lex_token(struct ore_rec_LexerState ls, void* tokens, void* errors) {
    int64_t ch = peek_char(ls);
    int64_t start_line = ls.line;
    int64_t start_col = ls.col;
    int64_t __tmp_57 = 0;
    if (is_digit(ch)) {
        return lex_number(ls, tokens);
    } else {
    }
    int64_t __tmp_58 = 0;
    if ((96070653495376 == 96070731833584)) {
        return lex_string(ls, tokens, errors);
    } else {
    }
    int64_t __tmp_59 = 0;
    if (is_alpha(ch)) {
        return lex_ident(ls, tokens);
    } else {
    }
    int64_t ch2 = peek_char2(ls);
    int64_t __tmp_60 = 0;
    if ((96070731865152 && 96070731867680)) {
        struct ore_rec_SpannedToken __tmp_61;
        struct ore_enum_Token __tmp_62; __tmp_62.tag = 41;
        __tmp_61.token = __tmp_62;
        __tmp_61.line = start_line;
        __tmp_61.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_61; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_63 = 0;
    if ((96070731896928 && 96070731899456)) {
        struct ore_rec_SpannedToken __tmp_64;
        struct ore_enum_Token __tmp_65; __tmp_65.tag = 43;
        __tmp_64.token = __tmp_65;
        __tmp_64.line = start_line;
        __tmp_64.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_64; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_66 = 0;
    if ((96070731928896 && 96070731931424)) {
        struct ore_rec_SpannedToken __tmp_67;
        struct ore_enum_Token __tmp_68; __tmp_68.tag = 50;
        __tmp_67.token = __tmp_68;
        __tmp_67.line = start_line;
        __tmp_67.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_67; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_69 = 0;
    if ((96070731961088 && 96070731963616)) {
        struct ore_rec_SpannedToken __tmp_70;
        struct ore_enum_Token __tmp_71; __tmp_71.tag = 44;
        __tmp_70.token = __tmp_71;
        __tmp_70.line = start_line;
        __tmp_70.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_70; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_72 = 0;
    if ((96070731993088 && 96070731995616)) {
        struct ore_rec_SpannedToken __tmp_73;
        struct ore_enum_Token __tmp_74; __tmp_74.tag = 47;
        __tmp_73.token = __tmp_74;
        __tmp_73.line = start_line;
        __tmp_73.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_73; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_75 = 0;
    if ((96070732025184 && 96070732027712)) {
        struct ore_rec_SpannedToken __tmp_76;
        struct ore_enum_Token __tmp_77; __tmp_77.tag = 48;
        __tmp_76.token = __tmp_77;
        __tmp_76.line = start_line;
        __tmp_76.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_76; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_78 = 0;
    if ((96070732057312 && 96070732059840)) {
        struct ore_rec_SpannedToken __tmp_79;
        struct ore_enum_Token __tmp_80; __tmp_80.tag = 49;
        __tmp_79.token = __tmp_80;
        __tmp_79.line = start_line;
        __tmp_79.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_79; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_81 = 0;
    if ((96070732089472 && 96070732092000)) {
        struct ore_rec_SpannedToken __tmp_82;
        struct ore_enum_Token __tmp_83; __tmp_83.tag = 58;
        __tmp_82.token = __tmp_83;
        __tmp_82.line = start_line;
        __tmp_82.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_82; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_84 = 0;
    if ((96070732121920 && 96070732124448)) {
        struct ore_rec_SpannedToken __tmp_85;
        struct ore_enum_Token __tmp_86; __tmp_86.tag = 57;
        __tmp_85.token = __tmp_86;
        __tmp_85.line = start_line;
        __tmp_85.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_85; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_87 = 0;
    if ((96070732154336 && 96070732156864)) {
        struct ore_rec_SpannedToken __tmp_88;
        struct ore_enum_Token __tmp_89; __tmp_89.tag = 59;
        __tmp_88.token = __tmp_89;
        __tmp_88.line = start_line;
        __tmp_88.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_88; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_90 = 0;
    if ((96070732186816 && 96070732189344)) {
        struct ore_rec_SpannedToken __tmp_91;
        struct ore_enum_Token __tmp_92; __tmp_92.tag = 60;
        __tmp_91.token = __tmp_92;
        __tmp_91.line = start_line;
        __tmp_91.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_91; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_93 = 0;
    if ((96070732219328 && 96070732221856)) {
        struct ore_rec_SpannedToken __tmp_94;
        struct ore_enum_Token __tmp_95; __tmp_95.tag = 61;
        __tmp_94.token = __tmp_95;
        __tmp_94.line = start_line;
        __tmp_94.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_94; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_96 = 0;
    if ((96070732251872 && 96070732254400)) {
        struct ore_rec_SpannedToken __tmp_97;
        struct ore_enum_Token __tmp_98; __tmp_98.tag = 54;
        __tmp_97.token = __tmp_98;
        __tmp_97.line = start_line;
        __tmp_97.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_97; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_99 = 0;
    if ((96070732284192 && 96070732286720)) {
        struct ore_rec_SpannedToken __tmp_100;
        struct ore_enum_Token __tmp_101; __tmp_101.tag = 56;
        __tmp_100.token = __tmp_101;
        __tmp_100.line = start_line;
        __tmp_100.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_100; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(advance(ls));
    } else {
    }
    int64_t __tmp_102 = 0;
    if ((96070653932768 == 96070732315776)) {
        struct ore_rec_SpannedToken __tmp_103;
        struct ore_enum_Token __tmp_104; __tmp_104.tag = 36;
        __tmp_103.token = __tmp_104;
        __tmp_103.line = start_line;
        __tmp_103.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_103; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_105 = 0;
    if ((96070653955024 == 96070732341168)) {
        struct ore_rec_SpannedToken __tmp_106;
        struct ore_enum_Token __tmp_107; __tmp_107.tag = 37;
        __tmp_106.token = __tmp_107;
        __tmp_106.line = start_line;
        __tmp_106.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_106; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_108 = 0;
    if ((96070653977312 == 96070732366592)) {
        struct ore_rec_SpannedToken __tmp_109;
        struct ore_enum_Token __tmp_110; __tmp_110.tag = 38;
        __tmp_109.token = __tmp_110;
        __tmp_109.line = start_line;
        __tmp_109.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_109; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_111 = 0;
    if ((96070653999568 == 96070732392048)) {
        struct ore_rec_SpannedToken __tmp_112;
        struct ore_enum_Token __tmp_113; __tmp_113.tag = 39;
        __tmp_112.token = __tmp_113;
        __tmp_112.line = start_line;
        __tmp_112.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_112; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_114 = 0;
    if ((96070654021856 == 96070732417536)) {
        struct ore_rec_SpannedToken __tmp_115;
        struct ore_enum_Token __tmp_116; __tmp_116.tag = 40;
        __tmp_115.token = __tmp_116;
        __tmp_115.line = start_line;
        __tmp_115.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_115; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_117 = 0;
    if ((96070654044208 == 96070732443056)) {
        struct ore_rec_SpannedToken __tmp_118;
        struct ore_enum_Token __tmp_119; __tmp_119.tag = 42;
        __tmp_118.token = __tmp_119;
        __tmp_118.line = start_line;
        __tmp_118.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_118; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_120 = 0;
    if ((96070654066400 == 96070732468640)) {
        struct ore_rec_SpannedToken __tmp_121;
        struct ore_enum_Token __tmp_122; __tmp_122.tag = 52;
        __tmp_121.token = __tmp_122;
        __tmp_121.line = start_line;
        __tmp_121.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_121; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_123 = 0;
    if ((96070654088688 == 96070732494544)) {
        struct ore_rec_SpannedToken __tmp_124;
        struct ore_enum_Token __tmp_125; __tmp_125.tag = 53;
        __tmp_124.token = __tmp_125;
        __tmp_124.line = start_line;
        __tmp_124.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_124; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_126 = 0;
    if ((96070654110912 == 96070732520480)) {
        struct ore_rec_SpannedToken __tmp_127;
        struct ore_enum_Token __tmp_128; __tmp_128.tag = 51;
        __tmp_127.token = __tmp_128;
        __tmp_127.line = start_line;
        __tmp_127.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_127; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_129 = 0;
    if ((96070654133200 == 96070732546352)) {
        struct ore_rec_SpannedToken __tmp_130;
        struct ore_enum_Token __tmp_131; __tmp_131.tag = 55;
        __tmp_130.token = __tmp_131;
        __tmp_130.line = start_line;
        __tmp_130.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_130; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_132 = 0;
    if ((96070654155712 == 96070732572352)) {
        struct ore_rec_SpannedToken __tmp_133;
        struct ore_enum_Token __tmp_134; __tmp_134.tag = 45;
        __tmp_133.token = __tmp_134;
        __tmp_133.line = start_line;
        __tmp_133.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_133; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_135 = 0;
    if ((96070654177904 == 96070732598032)) {
        struct ore_rec_SpannedToken __tmp_136;
        struct ore_enum_Token __tmp_137; __tmp_137.tag = 46;
        __tmp_136.token = __tmp_137;
        __tmp_136.line = start_line;
        __tmp_136.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_136; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_138 = 0;
    if ((96070654200096 == 96070732623744)) {
        struct ore_rec_SpannedToken __tmp_139;
        struct ore_enum_Token __tmp_140; __tmp_140.tag = 65;
        __tmp_139.token = __tmp_140;
        __tmp_139.line = start_line;
        __tmp_139.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_139; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_141 = 0;
    if ((96070654222416 == 96070732650064)) {
        struct ore_rec_SpannedToken __tmp_142;
        struct ore_enum_Token __tmp_143; __tmp_143.tag = 66;
        __tmp_142.token = __tmp_143;
        __tmp_142.line = start_line;
        __tmp_142.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_142; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_144 = 0;
    if ((96070654244736 == 96070732676416)) {
        struct ore_rec_SpannedToken __tmp_145;
        struct ore_enum_Token __tmp_146; __tmp_146.tag = 67;
        __tmp_145.token = __tmp_146;
        __tmp_145.line = start_line;
        __tmp_145.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_145; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_147 = 0;
    if ((96070654267088 == 96070732702800)) {
        struct ore_rec_SpannedToken __tmp_148;
        struct ore_enum_Token __tmp_149; __tmp_149.tag = 68;
        __tmp_148.token = __tmp_149;
        __tmp_148.line = start_line;
        __tmp_148.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_148; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_150 = 0;
    if ((96070654289440 == 96070731447232)) {
        struct ore_rec_SpannedToken __tmp_151;
        struct ore_enum_Token __tmp_152; __tmp_152.tag = 69;
        __tmp_151.token = __tmp_152;
        __tmp_151.line = start_line;
        __tmp_151.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_151; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_153 = 0;
    if ((96070654311824 == 96070732764992)) {
        struct ore_rec_SpannedToken __tmp_154;
        struct ore_enum_Token __tmp_155; __tmp_155.tag = 70;
        __tmp_154.token = __tmp_155;
        __tmp_154.line = start_line;
        __tmp_154.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_154; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_156 = 0;
    if ((96070654334208 == 96070732791472)) {
        struct ore_rec_SpannedToken __tmp_157;
        struct ore_enum_Token __tmp_158; __tmp_158.tag = 71;
        __tmp_157.token = __tmp_158;
        __tmp_157.line = start_line;
        __tmp_157.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_157; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
        return advance(ls);
    } else {
    }
    int64_t __tmp_159 = 0;
    if ((96070654358416 == 96070732817984)) {
        ore_list_push(errors, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("unexpected '!' at line ", 23), ore_int_to_str(start_line))));
        return advance(ls);
    } else {
    }
    ore_list_push(errors, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("unexpected character '", 22), ore_chr(ch)), ore_str_new("' at line ", 10)), ore_int_to_str(start_line))));
    return advance(ls);
}

struct ore_rec_LexerState lex_number(struct ore_rec_LexerState ls, void* tokens) {
    int64_t pos = ls.pos;
    int8_t is_float = ((int8_t)0);
    void* src = ls.source;
    int64_t start_line = ls.line;
    int64_t start_col = ls.col;
    while ((96070654424736 < 96070732889344)) {
        int64_t ch = ore_ord(ore_str_char_at(src, pos));
        int64_t __tmp_160 = 0;
        if ((96070732902480 || 96070732904976)) {
            pos = (96070654446464 + 96070732908176);
        } else {
            int64_t __tmp_161 = 0;
            if ((96070732922160 && 96070732932160)) {
                is_float = ((int8_t)1);
                pos = (96070654475008 + 96070732937216);
            } else {
                goto brk_10;
            }
            __tmp_160 = (int64_t)(__tmp_161);
        }
        cont_11: ;
    }
    brk_10: ;
    void* text = ore_str_replace(ore_str_substr(src, ls.pos, (96070654490560 - 96070732950400)), ore_str_new("_", 1), ore_str_new("", 0));
    int64_t __tmp_162 = 0;
    if (is_float) {
        struct ore_rec_SpannedToken __tmp_163;
        struct ore_rec_FloatTok __tmp_164;
        __tmp_164.value = ore_str_to_float(text);
        __tmp_163.token = __tmp_164;
        __tmp_163.line = start_line;
        __tmp_163.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_163; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
    } else {
        struct ore_rec_SpannedToken __tmp_165;
        struct ore_rec_IntTok __tmp_166;
        __tmp_166.value = ore_str_to_int(text);
        __tmp_165.token = __tmp_166;
        __tmp_165.line = start_line;
        __tmp_165.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_165; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
    }
    struct ore_rec_LexerState __tmp_167;
    __tmp_167.source = src;
    __tmp_167.pos = pos;
    __tmp_167.line = ls.line;
    __tmp_167.col = (96070732994992 + 96070732999488);
    return __tmp_167;
}

struct ore_rec_LexerState lex_string(struct ore_rec_LexerState ls, void* tokens, void* errors) {
    int64_t pos = (96070733012880 + 96070733013072);
    void* src = ls.source;
    int64_t start_line = ls.line;
    int64_t start_col = ls.col;
    int64_t current_line = ls.line;
    int64_t current_col = (96070733034928 + 96070733035120);
    void* s = ore_str_new("", 0);
    int8_t has_interp = ((int8_t)0);
    int64_t __tmp_168 = 0;
    if ((96070733053968 && 96070733060544)) {
        struct ore_rec_LexerState __tmp_169;
        __tmp_169.source = src;
        __tmp_169.pos = (96070654652336 + 96070733066240);
        __tmp_169.line = current_line;
        __tmp_169.col = (96070654659792 + 96070733071024);
        return lex_triple_string(__tmp_169, tokens, errors, start_line, start_col);
    } else {
    }
    while (1) {
        int64_t __tmp_170 = 0;
        if ((96070654670944 >= 96070733087040)) {
            ore_list_push(errors, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("unterminated string at line ", 28), ore_int_to_str(start_line))));
            struct ore_rec_LexerState __tmp_171;
            __tmp_171.source = src;
            __tmp_171.pos = pos;
            __tmp_171.line = current_line;
            __tmp_171.col = current_col;
            return __tmp_171;
        } else {
        }
        int64_t ch = ore_ord(ore_str_char_at(src, pos));
        int64_t __tmp_172 = 0;
        if ((96070654709600 == 96070733119344)) {
            pos = (96070654715120 + 96070733122576);
            current_col = (96070654719568 + 96070733126800);
            goto brk_12;
        } else {
        }
        int64_t __tmp_173 = 0;
        if ((96070654723600 == 96070733132352)) {
            int64_t __tmp_174 = 0;
            if ((!(has_interp))) {
                struct ore_rec_SpannedToken __tmp_175;
                struct ore_rec_StringStartTok __tmp_176;
                __tmp_176.value = s;
                __tmp_175.token = __tmp_176;
                __tmp_175.line = start_line;
                __tmp_175.col = start_col;
                ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_175; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
            } else {
                struct ore_rec_SpannedToken __tmp_177;
                struct ore_rec_StringMidTok __tmp_178;
                __tmp_178.value = s;
                __tmp_177.token = __tmp_178;
                __tmp_177.line = current_line;
                __tmp_177.col = current_col;
                ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_177; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
            }
            s = ore_str_new("", 0);
            has_interp = ((int8_t)1);
            pos = (96070654776960 + 96070733166912);
            current_col = (96070654781408 + 96070733171136);
            int64_t depth = 1LL;
            struct ore_rec_LexerState __tmp_179;
            __tmp_179.source = src;
            __tmp_179.pos = pos;
            __tmp_179.line = current_line;
            __tmp_179.col = current_col;
            struct ore_rec_LexerState inner_ls = __tmp_179;
            while ((96070654808096 > 96070733188144)) {
                int64_t __tmp_180 = 0;
                if ((!(has_more(inner_ls)))) {
                    ore_list_push(errors, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("unterminated interpolation at line ", 35), ore_int_to_str(start_line))));
                    struct ore_rec_LexerState __tmp_181;
                    __tmp_181.source = src;
                    __tmp_181.pos = inner_ls.pos;
                    __tmp_181.line = inner_ls.line;
                    __tmp_181.col = inner_ls.col;
                    return __tmp_181;
                } else {
                }
                int64_t ich = peek_char(inner_ls);
                int64_t __tmp_182 = 0;
                if ((96070654851472 == 96070733232208)) {
                    depth = (96070654855808 - 96070733235456);
                    int64_t __tmp_183 = 0;
                    if ((96070654858768 > 96070733240016)) {
                        inner_ls = lex_token(inner_ls, tokens, errors);
                        int64_t __tmp_184 = 0;
                        if ((96070733253792 > 96070733254032)) {
                            struct ore_rec_LexerState __tmp_185;
                            __tmp_185.source = src;
                            __tmp_185.pos = inner_ls.pos;
                            __tmp_185.line = inner_ls.line;
                            __tmp_185.col = inner_ls.col;
                            return __tmp_185;
                        } else {
                        }
                        __tmp_183 = (int64_t)(__tmp_184);
                    } else {
                        inner_ls = advance(inner_ls);
                    }
                    __tmp_182 = (int64_t)(__tmp_183);
                } else {
                    int64_t __tmp_186 = 0;
                    if ((96070654903728 == 96070733283968)) {
                        depth = (96070654908064 + 96070733287888);
                        inner_ls = lex_token(inner_ls, tokens, errors);
                        int64_t __tmp_187 = 0;
                        if ((96070733302416 > 96070733302656)) {
                            struct ore_rec_LexerState __tmp_188;
                            __tmp_188.source = src;
                            __tmp_188.pos = inner_ls.pos;
                            __tmp_188.line = inner_ls.line;
                            __tmp_188.col = inner_ls.col;
                            return __tmp_188;
                        } else {
                        }
                        __tmp_186 = (int64_t)(__tmp_187);
                    } else {
                        int64_t __tmp_189 = 0;
                        if ((96070733324912 || 96070733327440)) {
                            inner_ls = advance(inner_ls);
                        } else {
                            inner_ls = lex_token(inner_ls, tokens, errors);
                            int64_t __tmp_190 = 0;
                            if ((96070733348896 > 96070733349136)) {
                                struct ore_rec_LexerState __tmp_191;
                                __tmp_191.source = src;
                                __tmp_191.pos = inner_ls.pos;
                                __tmp_191.line = inner_ls.line;
                                __tmp_191.col = inner_ls.col;
                                return __tmp_191;
                            } else {
                            }
                            __tmp_189 = (int64_t)(__tmp_190);
                        }
                        __tmp_186 = (int64_t)(__tmp_189);
                    }
                    __tmp_182 = (int64_t)(__tmp_186);
                }
                cont_15: ;
            }
            brk_14: ;
            pos = inner_ls.pos;
            current_line = inner_ls.line;
            current_col = inner_ls.col;
            __tmp_173 = (int64_t)(__tmp_174);
        } else {
            int64_t __tmp_192 = 0;
            if ((96070655009088 == 96070733388096)) {
                pos = (96070655014736 + 96070733391344);
                int64_t __tmp_193 = 0;
                if ((96070655017472 >= 96070733397232)) {
                    ore_list_push(errors, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("unterminated escape at line ", 28), ore_int_to_str(current_line))));
                    struct ore_rec_LexerState __tmp_194;
                    __tmp_194.source = src;
                    __tmp_194.pos = pos;
                    __tmp_194.line = current_line;
                    __tmp_194.col = current_col;
                    return __tmp_194;
                } else {
                }
                int64_t ech = ore_ord(ore_str_char_at(src, pos));
                int64_t __tmp_195 = 0;
                if ((96070655093136 == 96070733431872)) {
                    s = ore_str_concat(s, ore_str_new("\n", 1));
                } else {
                    int64_t __tmp_196 = 0;
                    if ((96070655100560 == 96070733440288)) {
                        s = ore_str_concat(s, ore_str_new("\t", 1));
                    } else {
                        int64_t __tmp_197 = 0;
                        if ((96070655107984 == 96070733449840)) {
                            s = ore_str_concat(s, ore_str_new("", 1));
                        } else {
                            int64_t __tmp_198 = 0;
                            if ((96070655115408 == 96070733459696)) {
                                s = ore_str_concat(s, ore_str_new(" ", 1));
                            } else {
                                int64_t __tmp_199 = 0;
                                if ((96070655122800 == 96070733469904)) {
                                    s = ore_str_concat(s, ore_str_new("\\", 1));
                                } else {
                                    int64_t __tmp_200 = 0;
                                    if ((96070655130192 == 96070733480384)) {
                                        s = ore_str_concat(s, ore_str_new("\"", 1));
                                    } else {
                                        int64_t __tmp_201 = 0;
                                        if ((96070655137584 == 96070733491296)) {
                                            s = ore_str_concat(s, ore_str_new("{", 1));
                                        } else {
                                            ore_list_push(errors, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("unknown escape '\\", 17), ore_chr(ech)), ore_str_new("' at line ", 10)), ore_int_to_str(current_line))));
                                            struct ore_rec_LexerState __tmp_202;
                                            __tmp_202.source = src;
                                            __tmp_202.pos = pos;
                                            __tmp_202.line = current_line;
                                            __tmp_202.col = current_col;
                                            return __tmp_202;
                                        }
                                        __tmp_200 = (int64_t)(__tmp_201);
                                    }
                                    __tmp_199 = (int64_t)(__tmp_200);
                                }
                                __tmp_198 = (int64_t)(__tmp_199);
                            }
                            __tmp_197 = (int64_t)(__tmp_198);
                        }
                        __tmp_196 = (int64_t)(__tmp_197);
                    }
                    __tmp_195 = (int64_t)(__tmp_196);
                }
                pos = (96070655174240 + 96070733546912);
                current_col = (96070655178688 + 96070733551136);
                __tmp_192 = (int64_t)(__tmp_195);
            } else {
                s = ore_str_concat(s, ore_str_char_at(src, pos));
                int64_t __tmp_203 = 0;
                if ((96070655192160 == 96070733563840)) {
                    current_line = (96070655196848 + 96070733567088);
                    current_col = 1LL;
                } else {
                    current_col = (96070655204944 + 96070733574304);
                }
                pos = (96070655208928 + 96070733579472);
                __tmp_192 = (int64_t)(__tmp_203);
            }
            __tmp_173 = (int64_t)(__tmp_192);
        }
        cont_13: ;
    }
    brk_12: ;
    int64_t __tmp_204 = 0;
    if (has_interp) {
        struct ore_rec_SpannedToken __tmp_205;
        struct ore_rec_StringEndTok __tmp_206;
        __tmp_206.value = s;
        __tmp_205.token = __tmp_206;
        __tmp_205.line = start_line;
        __tmp_205.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_205; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
    } else {
        struct ore_rec_SpannedToken __tmp_207;
        struct ore_rec_StringLitTok __tmp_208;
        __tmp_208.value = s;
        __tmp_207.token = __tmp_208;
        __tmp_207.line = start_line;
        __tmp_207.col = start_col;
        ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_207; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
    }
    struct ore_rec_LexerState __tmp_209;
    __tmp_209.source = src;
    __tmp_209.pos = pos;
    __tmp_209.line = current_line;
    __tmp_209.col = current_col;
    return __tmp_209;
}

struct ore_rec_LexerState lex_triple_string(struct ore_rec_LexerState ls, void* tokens, void* errors, int64_t start_line, int64_t start_col) {
    int64_t pos = ls.pos;
    void* src = ls.source;
    int64_t current_line = ls.line;
    int64_t current_col = ls.col;
    void* s = ore_str_new("", 0);
    int64_t __tmp_210 = 0;
    if ((96070733656256 && 96070733662880)) {
        pos = (96070655330192 + 96070733666064);
        current_line = (96070655334640 + 96070733670272);
        current_col = 1LL;
    } else {
        int64_t __tmp_211 = 0;
        if ((96070733679248 && 96070733685872)) {
            pos = (96070655354864 + 96070733689072);
            int64_t __tmp_212 = 0;
            if ((96070733695776 && 96070733702400)) {
                pos = (96070655372112 + 96070733705616);
            } else {
            }
            current_line = (96070655376608 + 96070733710752);
            current_col = 1LL;
            __tmp_211 = (int64_t)(__tmp_212);
        } else {
        }
        __tmp_210 = (int64_t)(__tmp_211);
    }
    while (1) {
        int64_t __tmp_213 = 0;
        if ((96070655383024 >= 96070733721152)) {
            ore_list_push(errors, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("unterminated triple-quoted string at line ", 42), ore_int_to_str(start_line))));
            struct ore_rec_LexerState __tmp_214;
            __tmp_214.source = src;
            __tmp_214.pos = pos;
            __tmp_214.line = current_line;
            __tmp_214.col = current_col;
            return __tmp_214;
        } else {
        }
        int64_t ch = ore_ord(ore_str_char_at(src, pos));
        int64_t __tmp_215 = 0;
        if ((96070733771552 && 96070733778128)) {
            pos = (96070655446688 + 96070733781408);
            current_col = (96070655451136 + 96070733785632);
            goto brk_16;
        } else {
        }
        s = ore_str_concat(s, ore_str_char_at(src, pos));
        int64_t __tmp_216 = 0;
        if ((96070655463120 == 96070733798976)) {
            current_line = (96070655467744 + 96070733802208);
            current_col = 1LL;
        } else {
            current_col = (96070655475648 + 96070733808704);
        }
        pos = (96070655479568 + 96070733813504);
        cont_17: ;
    }
    brk_16: ;
    int64_t __tmp_217 = 0;
    if (ore_str_ends_with(s, ore_str_new("\n", 1))) {
        s = ore_str_substr(s, 0LL, (96070733824464 - 96070733824688));
        int64_t __tmp_218 = 0;
        if (ore_str_ends_with(s, ore_str_new("", 1))) {
            s = ore_str_substr(s, 0LL, (96070733835952 - 96070733836176));
        } else {
        }
        __tmp_217 = (int64_t)(__tmp_218);
    } else {
    }
    void* lines = ore_str_split(s, ore_str_new("\n", 1));
    int64_t min_indent = 999999LL;
    void* __tmp_219 = lines;
    int64_t __tmp_220 = ore_list_len(__tmp_219);
    for (int64_t __tmp_221 = 0; __tmp_221 < __tmp_220; __tmp_221++) {
        int64_t line = (int64_t)ore_list_get(__tmp_219, __tmp_221);
        int64_t trimmed = line.trim();
        int64_t __tmp_222 = 0;
        if ((96070733861280 > 96070733861504)) {
            int64_t indent = (96070733865152 - 96070733867264);
            int64_t __tmp_223 = 0;
            if ((96070655548048 < 96070655549904)) {
                min_indent = indent;
            } else {
            }
            __tmp_222 = (int64_t)(__tmp_223);
        } else {
        }
        cont_19: ;
    }
    brk_18: ;
    int64_t __tmp_224 = 0;
    if ((96070733882912 && 96070733885424)) {
        void* __tmp_225 = ore_list_new();
        void* dedented = __tmp_225;
        void* __tmp_226 = lines;
        int64_t __tmp_227 = ore_list_len(__tmp_226);
        for (int64_t __tmp_228 = 0; __tmp_228 < __tmp_227; __tmp_228++) {
            int64_t line = (int64_t)ore_list_get(__tmp_226, __tmp_228);
            int64_t __tmp_229 = 0;
            if ((96070733897472 >= 96070655573440)) {
                ore_list_push(dedented, (int64_t)(line.substr(min_indent, (96070733907248 - 96070655587152))));
            } else {
                ore_list_push(dedented, (int64_t)(line));
            }
            cont_21: ;
        }
        brk_20: ;
        s = ore_list_join(dedented, ore_str_new("\n", 1));
    } else {
    }
    struct ore_rec_SpannedToken __tmp_230;
    struct ore_rec_StringLitTok __tmp_231;
    __tmp_231.value = s;
    __tmp_230.token = __tmp_231;
    __tmp_230.line = start_line;
    __tmp_230.col = start_col;
    ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_230; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
    struct ore_rec_LexerState __tmp_232;
    __tmp_232.source = src;
    __tmp_232.pos = pos;
    __tmp_232.line = current_line;
    __tmp_232.col = current_col;
    return __tmp_232;
}

struct ore_rec_LexerState lex_ident(struct ore_rec_LexerState ls, void* tokens) {
    int64_t pos = ls.pos;
    void* src = ls.source;
    int64_t start_line = ls.line;
    int64_t start_col = ls.col;
    while ((96070733973232 && 96070733982096)) {
        pos = (96070655683952 + 96070733984848);
        cont_23: ;
    }
    brk_22: ;
    void* text = ore_str_substr(src, ls.pos, (96070655694144 - 96070733995888));
    struct ore_enum_Token tok = keyword_lookup(text);
    struct ore_rec_SpannedToken __tmp_233;
    __tmp_233.token = tok;
    __tmp_233.line = start_line;
    __tmp_233.col = start_col;
    ore_list_push(tokens, ({ struct ore_rec_SpannedToken __v2i = __tmp_233; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedToken)), &__v2i, sizeof(struct ore_rec_SpannedToken)); }));
    struct ore_rec_LexerState __tmp_234;
    __tmp_234.source = src;
    __tmp_234.pos = pos;
    __tmp_234.line = ls.line;
    __tmp_234.col = (96070734026624 + 96070734031120);
    return __tmp_234;
}

struct ore_enum_Token keyword_lookup(void* text) {
    int64_t __tmp_235 = 0;
    if (ore_str_eq(text, ore_str_new("fn", 2))) {
        struct ore_enum_Token __tmp_236; __tmp_236.tag = 7;
        __tmp_235 = (int64_t)(__tmp_236);
    } else {
        int64_t __tmp_237 = 0;
        if (ore_str_eq(text, ore_str_new("mut", 3))) {
            struct ore_enum_Token __tmp_238; __tmp_238.tag = 8;
            __tmp_237 = (int64_t)(__tmp_238);
        } else {
            int64_t __tmp_239 = 0;
            if (ore_str_eq(text, ore_str_new("if", 2))) {
                struct ore_enum_Token __tmp_240; __tmp_240.tag = 9;
                __tmp_239 = (int64_t)(__tmp_240);
            } else {
                int64_t __tmp_241 = 0;
                if (ore_str_eq(text, ore_str_new("then", 4))) {
                    struct ore_enum_Token __tmp_242; __tmp_242.tag = 10;
                    __tmp_241 = (int64_t)(__tmp_242);
                } else {
                    int64_t __tmp_243 = 0;
                    if (ore_str_eq(text, ore_str_new("else", 4))) {
                        struct ore_enum_Token __tmp_244; __tmp_244.tag = 11;
                        __tmp_243 = (int64_t)(__tmp_244);
                    } else {
                        int64_t __tmp_245 = 0;
                        if (ore_str_eq(text, ore_str_new("true", 4))) {
                            struct ore_enum_Token __tmp_246; __tmp_246.tag = 12;
                            __tmp_245 = (int64_t)(__tmp_246);
                        } else {
                            int64_t __tmp_247 = 0;
                            if (ore_str_eq(text, ore_str_new("false", 5))) {
                                struct ore_enum_Token __tmp_248; __tmp_248.tag = 13;
                                __tmp_247 = (int64_t)(__tmp_248);
                            } else {
                                int64_t __tmp_249 = 0;
                                if (ore_str_eq(text, ore_str_new("return", 6))) {
                                    struct ore_enum_Token __tmp_250; __tmp_250.tag = 14;
                                    __tmp_249 = (int64_t)(__tmp_250);
                                } else {
                                    int64_t __tmp_251 = 0;
                                    if (ore_str_eq(text, ore_str_new("and", 3))) {
                                        struct ore_enum_Token __tmp_252; __tmp_252.tag = 62;
                                        __tmp_251 = (int64_t)(__tmp_252);
                                    } else {
                                        int64_t __tmp_253 = 0;
                                        if (ore_str_eq(text, ore_str_new("or", 2))) {
                                            struct ore_enum_Token __tmp_254; __tmp_254.tag = 63;
                                            __tmp_253 = (int64_t)(__tmp_254);
                                        } else {
                                            int64_t __tmp_255 = 0;
                                            if (ore_str_eq(text, ore_str_new("not", 3))) {
                                                struct ore_enum_Token __tmp_256; __tmp_256.tag = 64;
                                                __tmp_255 = (int64_t)(__tmp_256);
                                            } else {
                                                int64_t __tmp_257 = 0;
                                                if (ore_str_eq(text, ore_str_new("for", 3))) {
                                                    struct ore_enum_Token __tmp_258; __tmp_258.tag = 15;
                                                    __tmp_257 = (int64_t)(__tmp_258);
                                                } else {
                                                    int64_t __tmp_259 = 0;
                                                    if (ore_str_eq(text, ore_str_new("while", 5))) {
                                                        struct ore_enum_Token __tmp_260; __tmp_260.tag = 16;
                                                        __tmp_259 = (int64_t)(__tmp_260);
                                                    } else {
                                                        int64_t __tmp_261 = 0;
                                                        if (ore_str_eq(text, ore_str_new("loop", 4))) {
                                                            struct ore_enum_Token __tmp_262; __tmp_262.tag = 17;
                                                            __tmp_261 = (int64_t)(__tmp_262);
                                                        } else {
                                                            int64_t __tmp_263 = 0;
                                                            if (ore_str_eq(text, ore_str_new("break", 5))) {
                                                                struct ore_enum_Token __tmp_264; __tmp_264.tag = 18;
                                                                __tmp_263 = (int64_t)(__tmp_264);
                                                            } else {
                                                                int64_t __tmp_265 = 0;
                                                                if (ore_str_eq(text, ore_str_new("in", 2))) {
                                                                    struct ore_enum_Token __tmp_266; __tmp_266.tag = 19;
                                                                    __tmp_265 = (int64_t)(__tmp_266);
                                                                } else {
                                                                    int64_t __tmp_267 = 0;
                                                                    if (ore_str_eq(text, ore_str_new("type", 4))) {
                                                                        struct ore_enum_Token __tmp_268; __tmp_268.tag = 20;
                                                                        __tmp_267 = (int64_t)(__tmp_268);
                                                                    } else {
                                                                        int64_t __tmp_269 = 0;
                                                                        if (ore_str_eq(text, ore_str_new("impl", 4))) {
                                                                            struct ore_enum_Token __tmp_270; __tmp_270.tag = 21;
                                                                            __tmp_269 = (int64_t)(__tmp_270);
                                                                        } else {
                                                                            int64_t __tmp_271 = 0;
                                                                            if (ore_str_eq(text, ore_str_new("trait", 5))) {
                                                                                struct ore_enum_Token __tmp_272; __tmp_272.tag = 22;
                                                                                __tmp_271 = (int64_t)(__tmp_272);
                                                                            } else {
                                                                                int64_t __tmp_273 = 0;
                                                                                if (ore_str_eq(text, ore_str_new("Some", 4))) {
                                                                                    struct ore_enum_Token __tmp_274; __tmp_274.tag = 23;
                                                                                    __tmp_273 = (int64_t)(__tmp_274);
                                                                                } else {
                                                                                    int64_t __tmp_275 = 0;
                                                                                    if (ore_str_eq(text, ore_str_new("None", 4))) {
                                                                                        struct ore_enum_Token __tmp_276; __tmp_276.tag = 24;
                                                                                        __tmp_275 = (int64_t)(__tmp_276);
                                                                                    } else {
                                                                                        int64_t __tmp_277 = 0;
                                                                                        if (ore_str_eq(text, ore_str_new("Ok", 2))) {
                                                                                            struct ore_enum_Token __tmp_278; __tmp_278.tag = 25;
                                                                                            __tmp_277 = (int64_t)(__tmp_278);
                                                                                        } else {
                                                                                            int64_t __tmp_279 = 0;
                                                                                            if (ore_str_eq(text, ore_str_new("Err", 3))) {
                                                                                                struct ore_enum_Token __tmp_280; __tmp_280.tag = 26;
                                                                                                __tmp_279 = (int64_t)(__tmp_280);
                                                                                            } else {
                                                                                                int64_t __tmp_281 = 0;
                                                                                                if (ore_str_eq(text, ore_str_new("use", 3))) {
                                                                                                    struct ore_enum_Token __tmp_282; __tmp_282.tag = 27;
                                                                                                    __tmp_281 = (int64_t)(__tmp_282);
                                                                                                } else {
                                                                                                    int64_t __tmp_283 = 0;
                                                                                                    if (ore_str_eq(text, ore_str_new("pub", 3))) {
                                                                                                        struct ore_enum_Token __tmp_284; __tmp_284.tag = 28;
                                                                                                        __tmp_283 = (int64_t)(__tmp_284);
                                                                                                    } else {
                                                                                                        int64_t __tmp_285 = 0;
                                                                                                        if (ore_str_eq(text, ore_str_new("spawn", 5))) {
                                                                                                            struct ore_enum_Token __tmp_286; __tmp_286.tag = 29;
                                                                                                            __tmp_285 = (int64_t)(__tmp_286);
                                                                                                        } else {
                                                                                                            int64_t __tmp_287 = 0;
                                                                                                            if (ore_str_eq(text, ore_str_new("match", 5))) {
                                                                                                                struct ore_enum_Token __tmp_288; __tmp_288.tag = 30;
                                                                                                                __tmp_287 = (int64_t)(__tmp_288);
                                                                                                            } else {
                                                                                                                int64_t __tmp_289 = 0;
                                                                                                                if (ore_str_eq(text, ore_str_new("continue", 8))) {
                                                                                                                    struct ore_enum_Token __tmp_290; __tmp_290.tag = 31;
                                                                                                                    __tmp_289 = (int64_t)(__tmp_290);
                                                                                                                } else {
                                                                                                                    int64_t __tmp_291 = 0;
                                                                                                                    if (ore_str_eq(text, ore_str_new("test", 4))) {
                                                                                                                        struct ore_enum_Token __tmp_292; __tmp_292.tag = 32;
                                                                                                                        __tmp_291 = (int64_t)(__tmp_292);
                                                                                                                    } else {
                                                                                                                        int64_t __tmp_293 = 0;
                                                                                                                        if (ore_str_eq(text, ore_str_new("assert", 6))) {
                                                                                                                            struct ore_enum_Token __tmp_294; __tmp_294.tag = 33;
                                                                                                                            __tmp_293 = (int64_t)(__tmp_294);
                                                                                                                        } else {
                                                                                                                            int64_t __tmp_295 = 0;
                                                                                                                            if (ore_str_eq(text, ore_str_new("assert_eq", 9))) {
                                                                                                                                struct ore_enum_Token __tmp_296; __tmp_296.tag = 34;
                                                                                                                                __tmp_295 = (int64_t)(__tmp_296);
                                                                                                                            } else {
                                                                                                                                int64_t __tmp_297 = 0;
                                                                                                                                if (ore_str_eq(text, ore_str_new("assert_ne", 9))) {
                                                                                                                                    struct ore_enum_Token __tmp_298; __tmp_298.tag = 35;
                                                                                                                                    __tmp_297 = (int64_t)(__tmp_298);
                                                                                                                                } else {
                                                                                                                                    struct ore_rec_IdentTok __tmp_299;
                                                                                                                                    __tmp_299.name = text;
                                                                                                                                    __tmp_297 = (int64_t)(__tmp_299);
                                                                                                                                }
                                                                                                                                __tmp_295 = (int64_t)(__tmp_297);
                                                                                                                            }
                                                                                                                            __tmp_293 = (int64_t)(__tmp_295);
                                                                                                                        }
                                                                                                                        __tmp_291 = (int64_t)(__tmp_293);
                                                                                                                    }
                                                                                                                    __tmp_289 = (int64_t)(__tmp_291);
                                                                                                                }
                                                                                                                __tmp_287 = (int64_t)(__tmp_289);
                                                                                                            }
                                                                                                            __tmp_285 = (int64_t)(__tmp_287);
                                                                                                        }
                                                                                                        __tmp_283 = (int64_t)(__tmp_285);
                                                                                                    }
                                                                                                    __tmp_281 = (int64_t)(__tmp_283);
                                                                                                }
                                                                                                __tmp_279 = (int64_t)(__tmp_281);
                                                                                            }
                                                                                            __tmp_277 = (int64_t)(__tmp_279);
                                                                                        }
                                                                                        __tmp_275 = (int64_t)(__tmp_277);
                                                                                    }
                                                                                    __tmp_273 = (int64_t)(__tmp_275);
                                                                                }
                                                                                __tmp_271 = (int64_t)(__tmp_273);
                                                                            }
                                                                            __tmp_269 = (int64_t)(__tmp_271);
                                                                        }
                                                                        __tmp_267 = (int64_t)(__tmp_269);
                                                                    }
                                                                    __tmp_265 = (int64_t)(__tmp_267);
                                                                }
                                                                __tmp_263 = (int64_t)(__tmp_265);
                                                            }
                                                            __tmp_261 = (int64_t)(__tmp_263);
                                                        }
                                                        __tmp_259 = (int64_t)(__tmp_261);
                                                    }
                                                    __tmp_257 = (int64_t)(__tmp_259);
                                                }
                                                __tmp_255 = (int64_t)(__tmp_257);
                                            }
                                            __tmp_253 = (int64_t)(__tmp_255);
                                        }
                                        __tmp_251 = (int64_t)(__tmp_253);
                                    }
                                    __tmp_249 = (int64_t)(__tmp_251);
                                }
                                __tmp_247 = (int64_t)(__tmp_249);
                            }
                            __tmp_245 = (int64_t)(__tmp_247);
                        }
                        __tmp_243 = (int64_t)(__tmp_245);
                    }
                    __tmp_241 = (int64_t)(__tmp_243);
                }
                __tmp_239 = (int64_t)(__tmp_241);
            }
            __tmp_237 = (int64_t)(__tmp_239);
        }
        __tmp_235 = (int64_t)(__tmp_237);
    }
    return __tmp_235;
}

void* fixup_multiline_pipes(void* tokens) {
    void* __tmp_300 = ore_list_new();
    void* result = __tmp_300;
    int64_t i = 0LL;
    int64_t pending_dedent_removes = 0LL;
    while ((96070655962960 < 96070734841072)) {
        int64_t __tmp_301 = ore_list_get(tokens, i);
        int8_t __tmp_302 = ore_list_get_kind(tokens, i);
        int64_t tok = __tmp_301;
        int64_t __tmp_303 = 0;
        if (tok.token.tag == 72) {
            int64_t j = (96070655989440 + 96070734860576);
            int64_t indent_count = 0LL;
            while ((96070655995712 < 96070734869056)) {
                int64_t __tmp_304 = ore_list_get(tokens, j);
                int8_t __tmp_305 = ore_list_get_kind(tokens, j);
                int64_t __tmp_306 = 0;
                if (__tmp_304.token.tag == 73) {
                    indent_count = (96070656012384 + 96070734885040);
                    j = (96070656016448 + 96070734889552);
                }
                else {
                    __tmp_306 = (int64_t)(0);
                }
                cont_27: ;
            }
            brk_26: ;
            int64_t __tmp_307 = 0;
            if ((96070656022368 < 96070734898416)) {
                int64_t __tmp_308 = ore_list_get(tokens, j);
                int8_t __tmp_309 = ore_list_get_kind(tokens, j);
                int64_t j_tok = __tmp_308.token;
                int8_t is_pipe_or_dot = (96070734911360 || 96070734915840);
                int64_t __tmp_310 = 0;
                if ((96070656045296 && 96070734922656)) {
                    pending_dedent_removes = (96070656056304 + 96070656058240);
                    i = j;
                    goto cont_25;
                } else {
                }
                __tmp_307 = (int64_t)(__tmp_310);
            } else {
            }
            ore_list_push(result, (int64_t)(tok));
            i = (96070656074064 + 96070734942576);
            __tmp_303 = (int64_t)(__tmp_307);
        }
        else if (tok.token.tag == 74) {
            int64_t __tmp_311 = 0;
            if ((96070656079760 > 96070734956016)) {
                pending_dedent_removes = (96070656085120 - 96070734959184);
                i = (96070656089104 + 96070734963472);
            } else {
                ore_list_push(result, (int64_t)(tok));
                i = (96070656099200 + 96070734972528);
            }
            __tmp_303 = (int64_t)(__tmp_311);
        }
        else {
            ore_list_push(result, (int64_t)(tok));
            i = (96070656110224 + 96070734982800);
        }
        cont_25: ;
    }
    brk_24: ;
    return result;
}

void* lex_split(void* source) {
    void* __tmp_312 = ore_list_new();
    int64_t __tmp_313 = (lex(source).tag != 0) ? lex(source).value : (int64_t)(__tmp_312);
    int64_t tokens = __tmp_313;
    void* __tmp_314 = ore_list_new();
    void* toks = __tmp_314;
    void* __tmp_315 = ore_list_new();
    void* lines = __tmp_315;
    void* __tmp_316 = ore_list_new();
    void* cols = __tmp_316;
    for (int64_t i = 0LL; i < tokens.len(); i++) {
        int64_t st = ((tokens)[i]);
        ore_list_push(lines, (int64_t)(st.line));
        ore_list_push(cols, (int64_t)(st.col));
        cont_29: ;
    }
    brk_28: ;
    void* __tmp_317 = ore_list_new();
    void* r = __tmp_317;
    ore_list_push(r, (int64_t)(intptr_t)(toks));
    ore_list_push(r, (int64_t)(intptr_t)(lines));
    ore_list_push(r, (int64_t)(intptr_t)(cols));
    return r;
}

int64_t alloc_expr(void* pool, struct ore_enum_Expr e) {
    ore_list_push(pool, ({ struct ore_enum_Expr __v2i = e; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Expr)), &__v2i, sizeof(struct ore_enum_Expr)); }));
    return (96070735067120 - 96070735067360);
}

struct ore_enum_Expr get_expr(void* pool, int64_t id) {
    int64_t __tmp_318 = ore_list_get(pool, id);
    int8_t __tmp_319 = ore_list_get_kind(pool, id);
    return *(struct ore_enum_Expr*)(intptr_t)(__tmp_318);
}

int64_t alloc_stmt(void* pool, struct ore_enum_Stmt s) {
    ore_list_push(pool, ({ struct ore_enum_Stmt __v2i = s; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Stmt)), &__v2i, sizeof(struct ore_enum_Stmt)); }));
    return (96070735095200 - 96070735095440);
}

struct ore_enum_Stmt get_stmt(void* pool, int64_t id) {
    int64_t __tmp_320 = ore_list_get(pool, id);
    int8_t __tmp_321 = ore_list_get_kind(pool, id);
    return *(struct ore_enum_Stmt*)(intptr_t)(__tmp_320);
}

int64_t alloc_pattern(void* pool, struct ore_enum_Pattern p) {
    ore_list_push(pool, ({ struct ore_enum_Pattern __v2i = p; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Pattern)), &__v2i, sizeof(struct ore_enum_Pattern)); }));
    return (96070735123344 - 96070735123584);
}

struct ore_enum_Pattern get_pattern(void* pool, int64_t id) {
    int64_t __tmp_322 = ore_list_get(pool, id);
    int8_t __tmp_323 = ore_list_get_kind(pool, id);
    return *(struct ore_enum_Pattern*)(intptr_t)(__tmp_322);
}

int64_t alloc_item(void* pool, struct ore_enum_Item i) {
    ore_list_push(pool, ({ struct ore_enum_Item __v2i = i; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Item)), &__v2i, sizeof(struct ore_enum_Item)); }));
    return (96070735151920 - 96070735152160);
}

struct ore_enum_Item get_item(void* pool, int64_t id) {
    int64_t __tmp_324 = ore_list_get(pool, id);
    int8_t __tmp_325 = ore_list_get_kind(pool, id);
    return *(struct ore_enum_Item*)(intptr_t)(__tmp_324);
}

int64_t alloc_type(void* pool, struct ore_enum_TypeExpr t) {
    ore_list_push(pool, ({ struct ore_enum_TypeExpr __v2i = t; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_TypeExpr)), &__v2i, sizeof(struct ore_enum_TypeExpr)); }));
    return (96070735180240 - 96070735180480);
}

struct ore_enum_TypeExpr get_type(void* pool, int64_t id) {
    int64_t __tmp_326 = ore_list_get(pool, id);
    int8_t __tmp_327 = ore_list_get_kind(pool, id);
    return *(struct ore_enum_TypeExpr*)(intptr_t)(__tmp_326);
}

struct ore_rec_SpannedStmt get_sstmt(void* pool, int64_t id) {
    int64_t __tmp_328 = ore_list_get(pool, id);
    int8_t __tmp_329 = ore_list_get_kind(pool, id);
    return *(struct ore_rec_SpannedStmt*)(intptr_t)(__tmp_328);
}

int64_t no_node() {
    return (-(1LL));
}

int8_t has_node(int64_t id) {
    return (96070657216400 >= 96070735215088);
}

struct ore_enum_StringPart get_string_part(void* pool, int64_t id) {
    int64_t __tmp_330 = ore_list_get(pool, id);
    int8_t __tmp_331 = ore_list_get_kind(pool, id);
    return *(struct ore_enum_StringPart*)(intptr_t)(__tmp_330);
}

struct ore_rec_MatchArm get_match_arm(void* pool, int64_t id) {
    int64_t __tmp_332 = ore_list_get(pool, id);
    int8_t __tmp_333 = ore_list_get_kind(pool, id);
    return *(struct ore_rec_MatchArm*)(intptr_t)(__tmp_332);
}

struct ore_rec_FnDef get_fn_def(void* pool, int64_t id) {
    int64_t __tmp_334 = ore_list_get(pool, id);
    int8_t __tmp_335 = ore_list_get_kind(pool, id);
    return *(struct ore_rec_FnDef*)(intptr_t)(__tmp_334);
}

struct ore_rec_FieldDef get_field_def(void* pool, int64_t id) {
    int64_t __tmp_336 = ore_list_get(pool, id);
    int8_t __tmp_337 = ore_list_get_kind(pool, id);
    return *(struct ore_rec_FieldDef*)(intptr_t)(__tmp_336);
}

struct ore_rec_VariantDef get_variant_def(void* pool, int64_t id) {
    int64_t __tmp_338 = ore_list_get(pool, id);
    int8_t __tmp_339 = ore_list_get_kind(pool, id);
    return *(struct ore_rec_VariantDef*)(intptr_t)(__tmp_338);
}

struct ore_rec_ParamDef get_param_def(void* pool, int64_t id) {
    int64_t __tmp_340 = ore_list_get(pool, id);
    int8_t __tmp_341 = ore_list_get_kind(pool, id);
    return *(struct ore_rec_ParamDef*)(intptr_t)(__tmp_340);
}

struct ore_rec_TypeParamDef get_type_param_def(void* pool, int64_t id) {
    int64_t __tmp_342 = ore_list_get(pool, id);
    int8_t __tmp_343 = ore_list_get_kind(pool, id);
    return *(struct ore_rec_TypeParamDef*)(intptr_t)(__tmp_342);
}

void* s_get_list(void* s, int64_t idx) {
    int64_t __tmp_344 = ore_list_get(s, idx);
    int8_t __tmp_345 = ore_list_get_kind(s, idx);
    return __tmp_344;
}

int64_t s_pos(void* s) {
    void* p = s_get_list(s, 1LL);
    int64_t __tmp_346 = ore_list_get(p, 0LL);
    int8_t __tmp_347 = ore_list_get_kind(p, 0LL);
    return __tmp_346;
}

int64_t s_set_pos(void* s, int64_t val) {
    void* p = s_get_list(s, 1LL);
    ore_list_set(p, 0LL, (int64_t)(val));
}

void* s_tokens(void* s) {
    return s_get_list(s, 0LL);
}

void* s_exprs(void* s) {
    return s_get_list(s, 2LL);
}

void* s_stmts(void* s) {
    return s_get_list(s, 3LL);
}

void* s_errors(void* s) {
    return s_get_list(s, 4LL);
}

void* s_items(void* s) {
    return s_get_list(s, 5LL);
}

void* s_types(void* s) {
    return s_get_list(s, 6LL);
}

void* s_patterns(void* s) {
    return s_get_list(s, 7LL);
}

int8_t s_has_error(void* s) {
    return (96070735420240 > 96070735420480);
}

int64_t s_error(void* s, void* msg) {
    void* errs = s_errors(s);
    int64_t __tmp_348 = 0;
    if ((96070735436176 == 96070735436416)) {
        int64_t line = p_peek_line(s);
        int64_t col = p_peek_col(s);
        ore_list_push(errs, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("parse error at ", 15), ore_int_to_str(line)), ore_str_new(":", 1)), ore_int_to_str(col)), ore_str_new(": ", 2)), msg)));
    } else {
    }
    return __tmp_348;
}

int64_t s_alloc_expr(void* s, struct ore_enum_Expr e) {
    void* pool = s_exprs(s);
    ore_list_push(pool, ({ struct ore_enum_Expr __v2i = e; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Expr)), &__v2i, sizeof(struct ore_enum_Expr)); }));
    return (96070735489872 - 96070735490112);
}

int64_t s_alloc_stmt(void* s, struct ore_enum_Stmt st) {
    void* pool = s_stmts(s);
    ore_list_push(pool, ({ struct ore_enum_Stmt __v2i = st; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Stmt)), &__v2i, sizeof(struct ore_enum_Stmt)); }));
    return (96070735512224 - 96070735512464);
}

int64_t s_alloc_item(void* s, struct ore_enum_Item it) {
    void* pool = s_items(s);
    ore_list_push(pool, ({ struct ore_enum_Item __v2i = it; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Item)), &__v2i, sizeof(struct ore_enum_Item)); }));
    return (96070735534576 - 96070735534816);
}

int64_t s_alloc_type(void* s, struct ore_enum_TypeExpr te) {
    void* pool = s_types(s);
    ore_list_push(pool, ({ struct ore_enum_TypeExpr __v2i = te; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_TypeExpr)), &__v2i, sizeof(struct ore_enum_TypeExpr)); }));
    return (96070735557136 - 96070735557376);
}

int64_t s_alloc_pat(void* s, struct ore_enum_Pattern p) {
    void* pool = s_patterns(s);
    ore_list_push(pool, ({ struct ore_enum_Pattern __v2i = p; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Pattern)), &__v2i, sizeof(struct ore_enum_Pattern)); }));
    return (96070735598016 - 96070735598256);
}

struct ore_enum_Token p_peek(void* s) {
    int64_t pos = s_pos(s);
    void* toks = s_tokens(s);
    int64_t __tmp_349 = 0;
    if ((96070657648464 < 96070735621152)) {
        __tmp_349 = (int64_t)(get_token(toks, pos));
    } else {
        struct ore_enum_Token __tmp_350; __tmp_350.tag = 75;
        __tmp_349 = (int64_t)(__tmp_350);
    }
    return __tmp_349;
}

int64_t p_peek_line(void* s) {
    int64_t pos = s_pos(s);
    void* lines = s_lines(s);
    int64_t __tmp_351 = 0;
    if ((96070657678544 < 96070735659088)) {
        int64_t __tmp_352 = ore_list_get(lines, pos);
        int8_t __tmp_353 = ore_list_get_kind(lines, pos);
        __tmp_351 = (int64_t)(__tmp_352);
    } else {
        __tmp_351 = (int64_t)(0LL);
    }
    return __tmp_351;
}

int64_t p_peek_col(void* s) {
    int64_t pos = s_pos(s);
    void* cols = s_cols(s);
    int64_t __tmp_354 = 0;
    if ((96070657705776 < 96070735687584)) {
        int64_t __tmp_355 = ore_list_get(cols, pos);
        int8_t __tmp_356 = ore_list_get_kind(cols, pos);
        __tmp_354 = (int64_t)(__tmp_355);
    } else {
        __tmp_354 = (int64_t)(0LL);
    }
    return __tmp_354;
}

struct ore_enum_Token p_peek_at(void* s, int64_t offset) {
    int64_t idx = (96070735706848 + 96070657730720);
    void* toks = s_tokens(s);
    int64_t __tmp_357 = 0;
    if ((96070657737760 < 96070735720752)) {
        __tmp_357 = (int64_t)(get_token(toks, idx));
    } else {
        struct ore_enum_Token __tmp_358; __tmp_358.tag = 75;
        __tmp_357 = (int64_t)(__tmp_358);
    }
    return __tmp_357;
}

int64_t p_skip(void* s) {
    int64_t pos = s_pos(s);
    int64_t __tmp_359 = 0;
    if ((96070657760944 < 96070735755584)) {
        __tmp_359 = (int64_t)(s_set_pos(s, (96070657771088 + 96070735760464)));
    } else {
    }
    return __tmp_359;
}

int8_t p_is_eof(struct ore_enum_Token tok) {
    int8_t __tmp_360 = 0;
    if (tok.tag == 75) {
        __tmp_360 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_360 = (int8_t)(((int8_t)0));
    }
    return __tmp_360;
}

int8_t p_is_dedent(struct ore_enum_Token tok) {
    int8_t __tmp_361 = 0;
    if (tok.tag == 74) {
        __tmp_361 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_361 = (int8_t)(((int8_t)0));
    }
    return __tmp_361;
}

int8_t p_is_newline(struct ore_enum_Token tok) {
    int8_t __tmp_362 = 0;
    if (tok.tag == 72) {
        __tmp_362 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_362 = (int8_t)(((int8_t)0));
    }
    return __tmp_362;
}

int8_t p_is_indent(struct ore_enum_Token tok) {
    int8_t __tmp_363 = 0;
    if (tok.tag == 73) {
        __tmp_363 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_363 = (int8_t)(((int8_t)0));
    }
    return __tmp_363;
}

int8_t p_at_block_end(void* s) {
    struct ore_enum_Token t = p_peek(s);
    return (96070735850816 || 96070735855280);
}

int64_t p_check_delim(struct ore_enum_Token tok, void* tag) {
    int64_t __tmp_364 = 0;
    if (ore_str_eq(tag, ore_str_new("(", 1))) {
        int64_t __tmp_365 = 0;
        if (tok.tag == 65) {
            __tmp_365 = (int64_t)(1LL);
        }
        else {
            __tmp_365 = (int64_t)(0LL);
        }
        __tmp_364 = (int64_t)(__tmp_365);
    }
    else if (ore_str_eq(tag, ore_str_new(")", 1))) {
        int64_t __tmp_366 = 0;
        if (tok.tag == 66) {
            __tmp_366 = (int64_t)(1LL);
        }
        else {
            __tmp_366 = (int64_t)(0LL);
        }
        __tmp_364 = (int64_t)(__tmp_366);
    }
    else if (ore_str_eq(tag, ore_str_new("[", 1))) {
        int64_t __tmp_367 = 0;
        if (tok.tag == 69) {
            __tmp_367 = (int64_t)(1LL);
        }
        else {
            __tmp_367 = (int64_t)(0LL);
        }
        __tmp_364 = (int64_t)(__tmp_367);
    }
    else if (ore_str_eq(tag, ore_str_new("]", 1))) {
        int64_t __tmp_368 = 0;
        if (tok.tag == 70) {
            __tmp_368 = (int64_t)(1LL);
        }
        else {
            __tmp_368 = (int64_t)(0LL);
        }
        __tmp_364 = (int64_t)(__tmp_368);
    }
    else if (ore_str_eq(tag, ore_str_new("lbrace", 6))) {
        int64_t __tmp_369 = 0;
        if (tok.tag == 67) {
            __tmp_369 = (int64_t)(1LL);
        }
        else {
            __tmp_369 = (int64_t)(0LL);
        }
        __tmp_364 = (int64_t)(__tmp_369);
    }
    else if (ore_str_eq(tag, ore_str_new("rbrace", 6))) {
        int64_t __tmp_370 = 0;
        if (tok.tag == 68) {
            __tmp_370 = (int64_t)(1LL);
        }
        else {
            __tmp_370 = (int64_t)(0LL);
        }
        __tmp_364 = (int64_t)(__tmp_370);
    }
    else if (ore_str_eq(tag, ore_str_new(",", 1))) {
        int64_t __tmp_371 = 0;
        if (tok.tag == 71) {
            __tmp_371 = (int64_t)(1LL);
        }
        else {
            __tmp_371 = (int64_t)(0LL);
        }
        __tmp_364 = (int64_t)(__tmp_371);
    }
    else {
        __tmp_364 = (int64_t)((-(1LL)));
    }
    return __tmp_364;
}

int64_t p_check_op(struct ore_enum_Token tok, void* tag) {
    int64_t __tmp_372 = 0;
    if (ore_str_eq(tag, ore_str_new(":", 1))) {
        int64_t __tmp_373 = 0;
        if (tok.tag == 52) {
            __tmp_373 = (int64_t)(1LL);
        }
        else {
            __tmp_373 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_373);
    }
    else if (ore_str_eq(tag, ore_str_new(":=", 2))) {
        int64_t __tmp_374 = 0;
        if (tok.tag == 41) {
            __tmp_374 = (int64_t)(1LL);
        }
        else {
            __tmp_374 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_374);
    }
    else if (ore_str_eq(tag, ore_str_new("=", 1))) {
        int64_t __tmp_375 = 0;
        if (tok.tag == 42) {
            __tmp_375 = (int64_t)(1LL);
        }
        else {
            __tmp_375 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_375);
    }
    else if (ore_str_eq(tag, ore_str_new("->", 2))) {
        int64_t __tmp_376 = 0;
        if (tok.tag == 49) {
            __tmp_376 = (int64_t)(1LL);
        }
        else {
            __tmp_376 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_376);
    }
    else if (ore_str_eq(tag, ore_str_new("=>", 2))) {
        int64_t __tmp_377 = 0;
        if (tok.tag == 50) {
            __tmp_377 = (int64_t)(1LL);
        }
        else {
            __tmp_377 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_377);
    }
    else if (ore_str_eq(tag, ore_str_new("..", 2))) {
        int64_t __tmp_378 = 0;
        if (tok.tag == 54) {
            __tmp_378 = (int64_t)(1LL);
        }
        else {
            __tmp_378 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_378);
    }
    else if (ore_str_eq(tag, ore_str_new(".", 1))) {
        int64_t __tmp_379 = 0;
        if (tok.tag == 53) {
            __tmp_379 = (int64_t)(1LL);
        }
        else {
            __tmp_379 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_379);
    }
    else if (ore_str_eq(tag, ore_str_new("?.", 2))) {
        int64_t __tmp_380 = 0;
        if (tok.tag == 56) {
            __tmp_380 = (int64_t)(1LL);
        }
        else {
            __tmp_380 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_380);
    }
    else if (ore_str_eq(tag, ore_str_new("?", 1))) {
        int64_t __tmp_381 = 0;
        if (tok.tag == 55) {
            __tmp_381 = (int64_t)(1LL);
        }
        else {
            __tmp_381 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_381);
    }
    else if (ore_str_eq(tag, ore_str_new("|", 1))) {
        int64_t __tmp_382 = 0;
        if (tok.tag == 51) {
            __tmp_382 = (int64_t)(1LL);
        }
        else {
            __tmp_382 = (int64_t)(0LL);
        }
        __tmp_372 = (int64_t)(__tmp_382);
    }
    else {
        __tmp_372 = (int64_t)((-(1LL)));
    }
    return __tmp_372;
}

int64_t p_check_kw1(struct ore_enum_Token tok, void* tag) {
    int64_t __tmp_383 = 0;
    if (ore_str_eq(tag, ore_str_new("fn", 2))) {
        int64_t __tmp_384 = 0;
        if (tok.tag == 7) {
            __tmp_384 = (int64_t)(1LL);
        }
        else {
            __tmp_384 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_384);
    }
    else if (ore_str_eq(tag, ore_str_new("if", 2))) {
        int64_t __tmp_385 = 0;
        if (tok.tag == 9) {
            __tmp_385 = (int64_t)(1LL);
        }
        else {
            __tmp_385 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_385);
    }
    else if (ore_str_eq(tag, ore_str_new("then", 4))) {
        int64_t __tmp_386 = 0;
        if (tok.tag == 10) {
            __tmp_386 = (int64_t)(1LL);
        }
        else {
            __tmp_386 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_386);
    }
    else if (ore_str_eq(tag, ore_str_new("else", 4))) {
        int64_t __tmp_387 = 0;
        if (tok.tag == 11) {
            __tmp_387 = (int64_t)(1LL);
        }
        else {
            __tmp_387 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_387);
    }
    else if (ore_str_eq(tag, ore_str_new("in", 2))) {
        int64_t __tmp_388 = 0;
        if (tok.tag == 19) {
            __tmp_388 = (int64_t)(1LL);
        }
        else {
            __tmp_388 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_388);
    }
    else if (ore_str_eq(tag, ore_str_new("for", 3))) {
        int64_t __tmp_389 = 0;
        if (tok.tag == 15) {
            __tmp_389 = (int64_t)(1LL);
        }
        else {
            __tmp_389 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_389);
    }
    else if (ore_str_eq(tag, ore_str_new("while", 5))) {
        int64_t __tmp_390 = 0;
        if (tok.tag == 16) {
            __tmp_390 = (int64_t)(1LL);
        }
        else {
            __tmp_390 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_390);
    }
    else if (ore_str_eq(tag, ore_str_new("loop", 4))) {
        int64_t __tmp_391 = 0;
        if (tok.tag == 17) {
            __tmp_391 = (int64_t)(1LL);
        }
        else {
            __tmp_391 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_391);
    }
    else if (ore_str_eq(tag, ore_str_new("return", 6))) {
        int64_t __tmp_392 = 0;
        if (tok.tag == 14) {
            __tmp_392 = (int64_t)(1LL);
        }
        else {
            __tmp_392 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_392);
    }
    else if (ore_str_eq(tag, ore_str_new("break", 5))) {
        int64_t __tmp_393 = 0;
        if (tok.tag == 18) {
            __tmp_393 = (int64_t)(1LL);
        }
        else {
            __tmp_393 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_393);
    }
    else if (ore_str_eq(tag, ore_str_new("continue", 8))) {
        int64_t __tmp_394 = 0;
        if (tok.tag == 31) {
            __tmp_394 = (int64_t)(1LL);
        }
        else {
            __tmp_394 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_394);
    }
    else if (ore_str_eq(tag, ore_str_new("match", 5))) {
        int64_t __tmp_395 = 0;
        if (tok.tag == 30) {
            __tmp_395 = (int64_t)(1LL);
        }
        else {
            __tmp_395 = (int64_t)(0LL);
        }
        __tmp_383 = (int64_t)(__tmp_395);
    }
    else {
        __tmp_383 = (int64_t)((-(1LL)));
    }
    return __tmp_383;
}

int64_t p_check_kw2(struct ore_enum_Token tok, void* tag) {
    int64_t __tmp_396 = 0;
    if (ore_str_eq(tag, ore_str_new("mut", 3))) {
        int64_t __tmp_397 = 0;
        if (tok.tag == 8) {
            __tmp_397 = (int64_t)(1LL);
        }
        else {
            __tmp_397 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_397);
    }
    else if (ore_str_eq(tag, ore_str_new("type", 4))) {
        int64_t __tmp_398 = 0;
        if (tok.tag == 20) {
            __tmp_398 = (int64_t)(1LL);
        }
        else {
            __tmp_398 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_398);
    }
    else if (ore_str_eq(tag, ore_str_new("use", 3))) {
        int64_t __tmp_399 = 0;
        if (tok.tag == 27) {
            __tmp_399 = (int64_t)(1LL);
        }
        else {
            __tmp_399 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_399);
    }
    else if (ore_str_eq(tag, ore_str_new("test", 4))) {
        int64_t __tmp_400 = 0;
        if (tok.tag == 32) {
            __tmp_400 = (int64_t)(1LL);
        }
        else {
            __tmp_400 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_400);
    }
    else if (ore_str_eq(tag, ore_str_new("spawn", 5))) {
        int64_t __tmp_401 = 0;
        if (tok.tag == 29) {
            __tmp_401 = (int64_t)(1LL);
        }
        else {
            __tmp_401 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_401);
    }
    else if (ore_str_eq(tag, ore_str_new("assert", 6))) {
        int64_t __tmp_402 = 0;
        if (tok.tag == 33) {
            __tmp_402 = (int64_t)(1LL);
        }
        else {
            __tmp_402 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_402);
    }
    else if (ore_str_eq(tag, ore_str_new("assert_eq", 9))) {
        int64_t __tmp_403 = 0;
        if (tok.tag == 34) {
            __tmp_403 = (int64_t)(1LL);
        }
        else {
            __tmp_403 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_403);
    }
    else if (ore_str_eq(tag, ore_str_new("assert_ne", 9))) {
        int64_t __tmp_404 = 0;
        if (tok.tag == 35) {
            __tmp_404 = (int64_t)(1LL);
        }
        else {
            __tmp_404 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_404);
    }
    else if (ore_str_eq(tag, ore_str_new("indent", 6))) {
        int64_t __tmp_405 = 0;
        if (tok.tag == 73) {
            __tmp_405 = (int64_t)(1LL);
        }
        else {
            __tmp_405 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_405);
    }
    else if (ore_str_eq(tag, ore_str_new("dedent", 6))) {
        int64_t __tmp_406 = 0;
        if (tok.tag == 74) {
            __tmp_406 = (int64_t)(1LL);
        }
        else {
            __tmp_406 = (int64_t)(0LL);
        }
        __tmp_396 = (int64_t)(__tmp_406);
    }
    else {
        __tmp_396 = (int64_t)((-(1LL)));
    }
    return __tmp_396;
}

int8_t p_check_tok(struct ore_enum_Token tok, void* tag) {
    int64_t r = p_check_delim(tok, tag);
    int64_t __tmp_407 = 0;
    if ((96070658242688 >= 96070736446272)) {
        return (96070658245664 == 96070736449488);
    } else {
    }
    int64_t r2 = p_check_op(tok, tag);
    int64_t __tmp_408 = 0;
    if ((96070658254992 >= 96070736461264)) {
        return (96070658258000 == 96070736464480);
    } else {
    }
    int64_t r3 = p_check_kw1(tok, tag);
    int64_t __tmp_409 = 0;
    if ((96070658267360 >= 96070736476256)) {
        return (96070658270368 == 96070736479472);
    } else {
    }
    int64_t r4 = p_check_kw2(tok, tag);
    int64_t __tmp_410 = 0;
    if ((96070658279728 >= 96070736491248)) {
        return (96070658282736 == 96070736494464);
    } else {
    }
    return ((int8_t)0);
}

int8_t p_at(void* s, void* tag) {
    return p_check_tok(p_peek(s), tag);
}

int64_t p_expect(void* s, void* tag) {
    int64_t __tmp_411 = 0;
    if (p_at(s, tag)) {
        __tmp_411 = (int64_t)(p_skip(s));
    } else {
        __tmp_411 = (int64_t)(s_error(s, ore_str_concat(ore_str_new("expected ", 9), tag)));
    }
    return __tmp_411;
}

void* p_expect_ident(void* s, void* ctx) {
    int64_t __tmp_412 = 0;
    if (p_peek(s).tag == 2) {
        int64_t n = p_peek(s).data[0];
        __tmp_412 = (int64_t)(n);
    }
    else {
        __tmp_412 = (int64_t)(ore_str_new("", 0));
    }
    return __tmp_412;
}

int64_t p_skip_nl(void* s) {
    while (1) {
        int64_t __tmp_413 = 0;
        if (p_is_newline(p_peek(s))) {
            __tmp_413 = (int64_t)(p_skip(s));
        } else {
            goto brk_30;
        }
        cont_31: ;
    }
    brk_30: ;
}

int64_t p_skip_ws(void* s) {
    while (1) {
        struct ore_enum_Token t = p_peek(s);
        int64_t __tmp_414 = 0;
        if ((96070736621392 || 96070736625888)) {
            __tmp_414 = (int64_t)(p_skip(s));
        } else {
            goto brk_32;
        }
        cont_33: ;
    }
    brk_32: ;
}

int8_t is_compound(struct ore_enum_Token t) {
    int8_t __tmp_415 = 0;
    if (t.tag == 57) {
        __tmp_415 = (int8_t)(((int8_t)1));
    }
    else if (t.tag == 58) {
        __tmp_415 = (int8_t)(((int8_t)1));
    }
    else if (t.tag == 59) {
        __tmp_415 = (int8_t)(((int8_t)1));
    }
    else if (t.tag == 60) {
        __tmp_415 = (int8_t)(((int8_t)1));
    }
    else if (t.tag == 61) {
        __tmp_415 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_415 = (int8_t)(((int8_t)0));
    }
    return __tmp_415;
}

struct ore_enum_BinOp compound_op(struct ore_enum_Token t) {
    struct ore_enum_BinOp __tmp_416 = {0};
    if (t.tag == 57) {
        struct ore_enum_BinOp __tmp_417; __tmp_417.tag = 0;
        __tmp_416 = __tmp_417;
    }
    else if (t.tag == 58) {
        struct ore_enum_BinOp __tmp_418; __tmp_418.tag = 1;
        __tmp_416 = __tmp_418;
    }
    else if (t.tag == 59) {
        struct ore_enum_BinOp __tmp_419; __tmp_419.tag = 2;
        __tmp_416 = __tmp_419;
    }
    else if (t.tag == 60) {
        struct ore_enum_BinOp __tmp_420; __tmp_420.tag = 3;
        __tmp_416 = __tmp_420;
    }
    else {
        struct ore_enum_BinOp __tmp_421; __tmp_421.tag = 4;
        __tmp_416 = __tmp_421;
    }
    return __tmp_416;
}

int64_t tok_to_op(struct ore_enum_Token tok) {
    int64_t __tmp_422 = 0;
    if (tok.tag == 36) {
        __tmp_422 = (int64_t)(1LL);
    }
    else if (tok.tag == 37) {
        __tmp_422 = (int64_t)(2LL);
    }
    else if (tok.tag == 38) {
        __tmp_422 = (int64_t)(3LL);
    }
    else if (tok.tag == 39) {
        __tmp_422 = (int64_t)(4LL);
    }
    else if (tok.tag == 40) {
        __tmp_422 = (int64_t)(5LL);
    }
    else if (tok.tag == 43) {
        __tmp_422 = (int64_t)(6LL);
    }
    else if (tok.tag == 44) {
        __tmp_422 = (int64_t)(7LL);
    }
    else if (tok.tag == 45) {
        __tmp_422 = (int64_t)(8LL);
    }
    else if (tok.tag == 46) {
        __tmp_422 = (int64_t)(9LL);
    }
    else if (tok.tag == 47) {
        __tmp_422 = (int64_t)(10LL);
    }
    else if (tok.tag == 48) {
        __tmp_422 = (int64_t)(11LL);
    }
    else if (tok.tag == 62) {
        __tmp_422 = (int64_t)(12LL);
    }
    else if (tok.tag == 63) {
        __tmp_422 = (int64_t)(13LL);
    }
    else if (tok.tag == 51) {
        __tmp_422 = (int64_t)(14LL);
    }
    else {
        __tmp_422 = (int64_t)(0LL);
    }
    return __tmp_422;
}

struct ore_enum_BinOp int_to_op(int64_t n) {
    int64_t __tmp_423 = 0;
    if ((96070658512160 == 96070736853984)) {
        struct ore_enum_BinOp __tmp_424; __tmp_424.tag = 0;
        __tmp_423 = (int64_t)(__tmp_424);
    } else {
        int64_t __tmp_425 = 0;
        if ((96070658517280 == 96070736859744)) {
            struct ore_enum_BinOp __tmp_426; __tmp_426.tag = 1;
            __tmp_425 = (int64_t)(__tmp_426);
        } else {
            int64_t __tmp_427 = 0;
            if ((96070658522400 == 96070736865472)) {
                struct ore_enum_BinOp __tmp_428; __tmp_428.tag = 2;
                __tmp_427 = (int64_t)(__tmp_428);
            } else {
                int64_t __tmp_429 = 0;
                if ((96070658527520 == 96070736871616)) {
                    struct ore_enum_BinOp __tmp_430; __tmp_430.tag = 3;
                    __tmp_429 = (int64_t)(__tmp_430);
                } else {
                    int64_t __tmp_431 = 0;
                    if ((96070658532640 == 96070736878464)) {
                        struct ore_enum_BinOp __tmp_432; __tmp_432.tag = 4;
                        __tmp_431 = (int64_t)(__tmp_432);
                    } else {
                        int64_t __tmp_433 = 0;
                        if ((96070658537760 == 96070736886512)) {
                            struct ore_enum_BinOp __tmp_434; __tmp_434.tag = 5;
                            __tmp_433 = (int64_t)(__tmp_434);
                        } else {
                            int64_t __tmp_435 = 0;
                            if ((96070658542848 == 96070736895008)) {
                                struct ore_enum_BinOp __tmp_436; __tmp_436.tag = 6;
                                __tmp_435 = (int64_t)(__tmp_436);
                            } else {
                                int64_t __tmp_437 = 0;
                                if ((96070658548032 == 96070736904000)) {
                                    struct ore_enum_BinOp __tmp_438; __tmp_438.tag = 7;
                                    __tmp_437 = (int64_t)(__tmp_438);
                                } else {
                                    int64_t __tmp_439 = 0;
                                    if ((96070658553120 == 96070736913456)) {
                                        struct ore_enum_BinOp __tmp_440; __tmp_440.tag = 8;
                                        __tmp_439 = (int64_t)(__tmp_440);
                                    } else {
                                        int64_t __tmp_441 = 0;
                                        if ((96070658558208 == 96070736923424)) {
                                            struct ore_enum_BinOp __tmp_442; __tmp_442.tag = 9;
                                            __tmp_441 = (int64_t)(__tmp_442);
                                        } else {
                                            int64_t __tmp_443 = 0;
                                            if ((96070658563392 == 96070736933920)) {
                                                struct ore_enum_BinOp __tmp_444; __tmp_444.tag = 10;
                                                __tmp_443 = (int64_t)(__tmp_444);
                                            } else {
                                                int64_t __tmp_445 = 0;
                                                if ((96070658568576 == 96070736944992)) {
                                                    struct ore_enum_BinOp __tmp_446; __tmp_446.tag = 11;
                                                    __tmp_445 = (int64_t)(__tmp_446);
                                                } else {
                                                    int64_t __tmp_447 = 0;
                                                    if ((96070658573728 == 96070736956608)) {
                                                        struct ore_enum_BinOp __tmp_448; __tmp_448.tag = 12;
                                                        __tmp_447 = (int64_t)(__tmp_448);
                                                    } else {
                                                        struct ore_enum_BinOp __tmp_449; __tmp_449.tag = 13;
                                                        __tmp_447 = (int64_t)(__tmp_449);
                                                    }
                                                    __tmp_445 = (int64_t)(__tmp_447);
                                                }
                                                __tmp_443 = (int64_t)(__tmp_445);
                                            }
                                            __tmp_441 = (int64_t)(__tmp_443);
                                        }
                                        __tmp_439 = (int64_t)(__tmp_441);
                                    }
                                    __tmp_437 = (int64_t)(__tmp_439);
                                }
                                __tmp_435 = (int64_t)(__tmp_437);
                            }
                            __tmp_433 = (int64_t)(__tmp_435);
                        }
                        __tmp_431 = (int64_t)(__tmp_433);
                    }
                    __tmp_429 = (int64_t)(__tmp_431);
                }
                __tmp_427 = (int64_t)(__tmp_429);
            }
            __tmp_425 = (int64_t)(__tmp_427);
        }
        __tmp_423 = (int64_t)(__tmp_425);
    }
    return __tmp_423;
}

int64_t bp_l(int64_t n) {
    int64_t __tmp_450 = 0;
    if ((96070658587168 <= 96070737002384)) {
        __tmp_450 = (int64_t)(0LL);
    } else {
        int64_t __tmp_451 = 0;
        if ((96070658591200 == 96070737006544)) {
            __tmp_451 = (int64_t)(3LL);
        } else {
            int64_t __tmp_452 = 0;
            if ((96070658595264 == 96070737010720)) {
                __tmp_452 = (int64_t)(5LL);
            } else {
                int64_t __tmp_453 = 0;
                if ((96070737015792 && 96070737018320)) {
                    __tmp_453 = (int64_t)(7LL);
                } else {
                    int64_t __tmp_454 = 0;
                    if ((96070658606016 == 96070737022832)) {
                        __tmp_454 = (int64_t)(9LL);
                    } else {
                        int64_t __tmp_455 = 0;
                        if ((96070658610080 <= 96070737028480)) {
                            __tmp_455 = (int64_t)(11LL);
                        } else {
                            __tmp_455 = (int64_t)(13LL);
                        }
                        __tmp_454 = (int64_t)(__tmp_455);
                    }
                    __tmp_453 = (int64_t)(__tmp_454);
                }
                __tmp_452 = (int64_t)(__tmp_453);
            }
            __tmp_451 = (int64_t)(__tmp_452);
        }
        __tmp_450 = (int64_t)(__tmp_451);
    }
    return __tmp_450;
}

int64_t bp_r(int64_t n) {
    int64_t __tmp_456 = 0;
    if ((96070658621376 <= 96070737046256)) {
        __tmp_456 = (int64_t)(0LL);
    } else {
        int64_t __tmp_457 = 0;
        if ((96070658625408 == 96070737050416)) {
            __tmp_457 = (int64_t)(4LL);
        } else {
            int64_t __tmp_458 = 0;
            if ((96070658629472 == 96070737054592)) {
                __tmp_458 = (int64_t)(6LL);
            } else {
                int64_t __tmp_459 = 0;
                if ((96070737059664 && 96070737062192)) {
                    __tmp_459 = (int64_t)(8LL);
                } else {
                    int64_t __tmp_460 = 0;
                    if ((96070658640224 == 96070737066704)) {
                        __tmp_460 = (int64_t)(20LL);
                    } else {
                        int64_t __tmp_461 = 0;
                        if ((96070658644320 <= 96070737072352)) {
                            __tmp_461 = (int64_t)(12LL);
                        } else {
                            __tmp_461 = (int64_t)(14LL);
                        }
                        __tmp_460 = (int64_t)(__tmp_461);
                    }
                    __tmp_459 = (int64_t)(__tmp_460);
                }
                __tmp_458 = (int64_t)(__tmp_459);
            }
            __tmp_457 = (int64_t)(__tmp_458);
        }
        __tmp_456 = (int64_t)(__tmp_457);
    }
    return __tmp_456;
}

int64_t parse_type_expr(void* s) {
    int64_t __tmp_462 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_463 = 0;
    if (p_peek(s).tag == 2) {
        int64_t n = p_peek(s).data[0];
        struct ore_rec_NamedType __tmp_464;
        __tmp_464.name = n;
        __tmp_463 = (int64_t)(s_alloc_type(s, __tmp_464));
    }
    else {
        __tmp_463 = (int64_t)((-(1LL)));
    }
    return __tmp_463;
}

int64_t parse_param(void* s, void* out) {
    void* name = p_expect_ident(s, ore_str_new("parameter", 9));
    int64_t __tmp_465 = 0;
    if (s_has_error(s)) {
        return;
    } else {
    }
    int64_t __tmp_466 = 0;
    if (s_has_error(s)) {
        return;
    } else {
    }
    int64_t ty_id = parse_type_expr(s);
    int64_t __tmp_467 = 0;
    if (s_has_error(s)) {
        return;
    } else {
    }
    struct ore_enum_TypeExpr ty = get_type(s_types(s), ty_id);
    struct ore_rec_ParamDef __tmp_468;
    __tmp_468.name = name;
    __tmp_468.ty = ty;
    __tmp_468.default_expr = no_node();
    ore_list_push(out, ({ struct ore_rec_ParamDef __v2i = __tmp_468; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_ParamDef)), &__v2i, sizeof(struct ore_rec_ParamDef)); }));
    return __tmp_467;
}

void* parse_call_args(void* s) {
    void* __tmp_469 = ore_list_new();
    void* args = __tmp_469;
    int64_t __tmp_470 = 0;
    if ((!(p_at(s, ore_str_new(")", 1))))) {
        ore_list_push(args, (int64_t)(parse_expr(s, 0LL)));
        int64_t __tmp_471 = 0;
        if (s_has_error(s)) {
            void* __tmp_472 = ore_list_new();
            return __tmp_472;
        } else {
        }
        while (1) {
            int64_t __tmp_473 = 0;
            if (p_at(s, ore_str_new(",", 1))) {
                int64_t __tmp_474 = 0;
                if (p_at(s, ore_str_new(")", 1))) {
                    goto brk_34;
                } else {
                }
                ore_list_push(args, (int64_t)(parse_expr(s, 0LL)));
                int64_t __tmp_475 = 0;
                if (s_has_error(s)) {
                    void* __tmp_476 = ore_list_new();
                    return __tmp_476;
                } else {
                }
                __tmp_473 = (int64_t)(p_skip_ws(s));
            } else {
                goto brk_34;
            }
            cont_35: ;
        }
        brk_34: ;
        __tmp_470 = (int64_t)(p_skip_ws(s));
    } else {
    }
    return args;
}

int64_t parse_block(void* s, void* out) {
    int64_t __tmp_477 = 0;
    if (s_has_error(s)) {
        return;
    } else {
    }
    while (1) {
        int64_t __tmp_478 = 0;
        if (p_at_block_end(s)) {
            goto brk_36;
        } else {
        }
        int64_t line = p_peek_line(s);
        int64_t sid = parse_stmt(s);
        int64_t __tmp_479 = 0;
        if (s_has_error(s)) {
            return;
        } else {
        }
        struct ore_rec_SpannedStmt __tmp_480;
        __tmp_480.stmt_id = sid;
        __tmp_480.line = line;
        ore_list_push(out, ({ struct ore_rec_SpannedStmt __v2i = __tmp_480; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedStmt)), &__v2i, sizeof(struct ore_rec_SpannedStmt)); }));
        cont_37: ;
    }
    brk_36: ;
    int64_t __tmp_481 = 0;
    if (p_at(s, ore_str_new("dedent", 6))) {
        __tmp_481 = (int64_t)(p_skip(s));
    } else {
    }
    return __tmp_481;
}

void* parse_optional_msg(void* s) {
    int64_t __tmp_482 = 0;
    if (p_at(s, ore_str_new(",", 1))) {
        int64_t __tmp_483 = 0;
        if (p_peek(s).tag == 3) {
            int64_t sv = p_peek(s).data[0];
            __tmp_483 = (int64_t)(sv);
        }
        else {
            __tmp_483 = (int64_t)(ore_str_new("", 0));
        }
        __tmp_482 = (int64_t)(__tmp_483);
    } else {
        __tmp_482 = (int64_t)(ore_str_new("", 0));
    }
    return __tmp_482;
}

int64_t parse_stmt(void* s) {
    int64_t __tmp_484 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_485 = 0;
    if (p_peek(s).tag == 14) {
        struct ore_enum_Token t = p_peek(s);
        int64_t __tmp_486 = 0;
        if ((96070737451792 || 96070737456288)) {
            struct ore_rec_ReturnStmt __tmp_487;
            __tmp_487.value = no_node();
            __tmp_486 = (int64_t)(s_alloc_stmt(s, __tmp_487));
        } else {
            int64_t eid = parse_expr(s, 0LL);
            struct ore_rec_ReturnStmt __tmp_488;
            __tmp_488.value = eid;
            __tmp_486 = (int64_t)(s_alloc_stmt(s, __tmp_488));
        }
        __tmp_485 = (int64_t)(__tmp_486);
    }
    else if (p_peek(s).tag == 15) {
        __tmp_485 = (int64_t)(parse_for_stmt(s));
    }
    else if (p_peek(s).tag == 16) {
        int64_t cid = parse_expr(s, 0LL);
        int64_t __tmp_489 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        void* __tmp_490 = ore_list_new();
        void* bstmts = __tmp_490;
        struct ore_rec_WhileStmt __tmp_491;
        __tmp_491.cond = cid;
        struct ore_rec_Block __tmp_492;
        __tmp_492.stmts = bstmts;
        __tmp_491.body = __tmp_492;
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_491));
    }
    else if (p_peek(s).tag == 17) {
        void* __tmp_493 = ore_list_new();
        void* bstmts = __tmp_493;
        struct ore_rec_LoopStmt __tmp_494;
        struct ore_rec_Block __tmp_495;
        __tmp_495.stmts = bstmts;
        __tmp_494.body = __tmp_495;
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_494));
    }
    else if (p_peek(s).tag == 18) {
        struct ore_enum_Stmt __tmp_496; __tmp_496.tag = 12;
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_496));
    }
    else if (p_peek(s).tag == 31) {
        struct ore_enum_Stmt __tmp_497; __tmp_497.tag = 13;
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_497));
    }
    else if (p_peek(s).tag == 8) {
        void* name = p_expect_ident(s, ore_str_new("variable", 8));
        int64_t __tmp_498 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t __tmp_499 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t eid = parse_expr(s, 0LL);
        struct ore_rec_LetStmt __tmp_500;
        __tmp_500.name = name;
        __tmp_500.mutable = ((int8_t)1);
        __tmp_500.value = eid;
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_500));
    }
    else if (p_peek(s).tag == 29) {
        int64_t eid = parse_expr(s, 0LL);
        struct ore_rec_SpawnStmt __tmp_501;
        __tmp_501.expr = eid;
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_501));
    }
    else if (p_peek(s).tag == 33) {
        int8_t has_p = p_at(s, ore_str_new("(", 1));
        int64_t __tmp_502 = 0;
        if (has_p) {
            __tmp_502 = (int64_t)(p_skip(s));
        } else {
        }
        int64_t cid = parse_expr(s, 0LL);
        void* msg = parse_optional_msg(s);
        int64_t __tmp_503 = 0;
        if (has_p) {
            __tmp_503 = (int64_t)(p_expect(s, ore_str_new(")", 1)));
        } else {
        }
        struct ore_rec_ExprStmt __tmp_504;
        struct ore_rec_AssertExpr __tmp_505;
        __tmp_505.cond = cid;
        __tmp_505.message = msg;
        __tmp_504.expr = s_alloc_expr(s, __tmp_505);
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_504));
    }
    else if (p_peek(s).tag == 34) {
        int8_t has_p = p_at(s, ore_str_new("(", 1));
        int64_t __tmp_506 = 0;
        if (has_p) {
            __tmp_506 = (int64_t)(p_skip(s));
        } else {
        }
        int64_t lid = parse_expr(s, 0LL);
        int64_t __tmp_507 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t rid = parse_expr(s, 0LL);
        void* msg = parse_optional_msg(s);
        int64_t __tmp_508 = 0;
        if (has_p) {
            __tmp_508 = (int64_t)(p_expect(s, ore_str_new(")", 1)));
        } else {
        }
        struct ore_rec_ExprStmt __tmp_509;
        struct ore_rec_AssertEqExpr __tmp_510;
        __tmp_510.left = lid;
        __tmp_510.right = rid;
        __tmp_510.message = msg;
        __tmp_509.expr = s_alloc_expr(s, __tmp_510);
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_509));
    }
    else if (p_peek(s).tag == 35) {
        int8_t has_p = p_at(s, ore_str_new("(", 1));
        int64_t __tmp_511 = 0;
        if (has_p) {
            __tmp_511 = (int64_t)(p_skip(s));
        } else {
        }
        int64_t lid = parse_expr(s, 0LL);
        int64_t __tmp_512 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t rid = parse_expr(s, 0LL);
        void* msg = parse_optional_msg(s);
        int64_t __tmp_513 = 0;
        if (has_p) {
            __tmp_513 = (int64_t)(p_expect(s, ore_str_new(")", 1)));
        } else {
        }
        struct ore_rec_ExprStmt __tmp_514;
        struct ore_rec_AssertNeExpr __tmp_515;
        __tmp_515.left = lid;
        __tmp_515.right = rid;
        __tmp_515.message = msg;
        __tmp_514.expr = s_alloc_expr(s, __tmp_515);
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_514));
    }
    else if (p_peek(s).tag == 2) {
        int64_t _ = p_peek(s).data[0];
        __tmp_485 = (int64_t)(parse_ident_stmt(s));
    }
    else {
        int64_t eid = parse_expr(s, 0LL);
        struct ore_rec_ExprStmt __tmp_516;
        __tmp_516.expr = eid;
        __tmp_485 = (int64_t)(s_alloc_stmt(s, __tmp_516));
    }
    return __tmp_485;
}

int64_t parse_for_stmt(void* s) {
    void* vname = p_expect_ident(s, ore_str_new("variable", 8));
    int64_t __tmp_517 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_518 = 0;
    if (p_at(s, ore_str_new(",", 1))) {
        void* vv = p_expect_ident(s, ore_str_new("value variable", 14));
        int64_t __tmp_519 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t __tmp_520 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t iter_id = parse_expr(s, 0LL);
        int64_t __tmp_521 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        void* __tmp_522 = ore_list_new();
        void* bstmts = __tmp_522;
        struct ore_rec_ForEachKVStmt __tmp_523;
        __tmp_523.key_var = vname;
        __tmp_523.val_var = vv;
        __tmp_523.iterable = iter_id;
        struct ore_rec_Block __tmp_524;
        __tmp_524.stmts = bstmts;
        __tmp_523.body = __tmp_524;
        return s_alloc_stmt(s, __tmp_523);
        __tmp_518 = (int64_t)(parse_block(s, bstmts));
    } else {
    }
    int64_t __tmp_525 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t start_id = parse_expr(s, 3LL);
    int64_t __tmp_526 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_527 = 0;
    if (p_at(s, ore_str_new("..", 2))) {
        int64_t end_id = parse_expr(s, 0LL);
        int64_t __tmp_528 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t __tmp_529 = 0;
        if (p_peek(s).tag == 2) {
            int64_t n = p_peek(s).data[0];
            int64_t __tmp_530 = 0;
            if (ore_str_eq((void*)(intptr_t)(96070659542560), ore_str_new("step", 4))) {
                __tmp_530 = (int64_t)(parse_expr(s, 0LL));
            } else {
                __tmp_530 = (int64_t)(no_node());
            }
            __tmp_529 = (int64_t)(__tmp_530);
        }
        else {
            __tmp_529 = (int64_t)(no_node());
        }
        int64_t step_id = __tmp_529;
        void* __tmp_531 = ore_list_new();
        void* bstmts = __tmp_531;
        struct ore_rec_ForInStmt __tmp_532;
        __tmp_532.var_name = vname;
        __tmp_532.start = start_id;
        __tmp_532.end = end_id;
        __tmp_532.step = step_id;
        struct ore_rec_Block __tmp_533;
        __tmp_533.stmts = bstmts;
        __tmp_532.body = __tmp_533;
        __tmp_527 = (int64_t)(s_alloc_stmt(s, __tmp_532));
    } else {
        void* __tmp_534 = ore_list_new();
        void* bstmts = __tmp_534;
        struct ore_rec_ForEachStmt __tmp_535;
        __tmp_535.var_name = vname;
        __tmp_535.iterable = start_id;
        struct ore_rec_Block __tmp_536;
        __tmp_536.stmts = bstmts;
        __tmp_535.body = __tmp_536;
        __tmp_527 = (int64_t)(s_alloc_stmt(s, __tmp_535));
    }
    return __tmp_527;
}

int64_t parse_ident_stmt(void* s) {
    int64_t saved = s_pos(s);
    int64_t __tmp_537 = 0;
    if (p_peek(s).tag == 2) {
        int64_t name = p_peek(s).data[0];
        int64_t __tmp_538 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070659649008), ore_str_new("print", 5))) {
            int64_t eid = parse_expr(s, 0LL);
            struct ore_rec_ExprStmt __tmp_539;
            struct ore_rec_PrintExpr __tmp_540;
            __tmp_540.inner = eid;
            __tmp_539.expr = s_alloc_expr(s, __tmp_540);
            return s_alloc_stmt(s, __tmp_539);
            __tmp_538 = (int64_t)(p_skip(s));
        } else {
        }
        int64_t __tmp_541 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070659677968), ore_str_new("sleep", 5))) {
            int64_t eid = parse_expr(s, 0LL);
            struct ore_rec_ExprStmt __tmp_542;
            struct ore_rec_SleepExpr __tmp_543;
            __tmp_543.inner = eid;
            __tmp_542.expr = s_alloc_expr(s, __tmp_543);
            return s_alloc_stmt(s, __tmp_542);
            __tmp_541 = (int64_t)(p_skip(s));
        } else {
        }
        int64_t __tmp_544 = 0;
        if (p_at(s, ore_str_new(":=", 2))) {
            int64_t eid = parse_expr(s, 0LL);
            struct ore_rec_LetStmt __tmp_545;
            __tmp_545.name = name;
            __tmp_545.mutable = ((int8_t)0);
            __tmp_545.value = eid;
            return s_alloc_stmt(s, __tmp_545);
            __tmp_544 = (int64_t)(p_skip(s));
        } else {
        }
        int64_t __tmp_546 = 0;
        if (p_at(s, ore_str_new("=", 1))) {
            int64_t eid = parse_expr(s, 0LL);
            struct ore_rec_AssignStmt __tmp_547;
            __tmp_547.name = name;
            __tmp_547.value = eid;
            return s_alloc_stmt(s, __tmp_547);
            __tmp_546 = (int64_t)(p_skip(s));
        } else {
        }
        int64_t __tmp_548 = 0;
        if (is_compound(p_peek(s))) {
            struct ore_enum_BinOp op = compound_op(p_peek(s));
            int64_t rhs = parse_expr(s, 0LL);
            int64_t __tmp_549 = 0;
            if (s_has_error(s)) {
                return (-(1LL));
            } else {
            }
            struct ore_rec_IdentExpr __tmp_550;
            __tmp_550.name = name;
            int64_t iid = s_alloc_expr(s, __tmp_550);
            struct ore_rec_BinOpExpr __tmp_551;
            __tmp_551.op = op;
            __tmp_551.left = iid;
            __tmp_551.right = rhs;
            int64_t vid = s_alloc_expr(s, __tmp_551);
            struct ore_rec_AssignStmt __tmp_552;
            __tmp_552.name = name;
            __tmp_552.value = vid;
            return s_alloc_stmt(s, __tmp_552);
            __tmp_548 = (int64_t)(__tmp_549);
        } else {
        }
        int64_t eid = parse_expr(s, 0LL);
        int64_t __tmp_553 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t __tmp_554 = 0;
        if (p_at(s, ore_str_new("=", 1))) {
            int64_t vid = parse_expr(s, 0LL);
            int64_t __tmp_555 = 0;
            if (s_has_error(s)) {
                return (-(1LL));
            } else {
            }
            struct ore_enum_Expr e = get_expr(s_exprs(s), eid);
            int64_t __tmp_556 = 0;
            if (e.tag == 22) {
                int64_t obj = e.data[0];
                int64_t idx = e.data[1];
                struct ore_rec_IndexAssignStmt __tmp_557;
                __tmp_557.object = obj;
                __tmp_557.index = idx;
                __tmp_557.value = vid;
                return s_alloc_stmt(s, __tmp_557);
            }
            else if (e.tag == 17) {
                int64_t obj = e.data[0];
                int64_t field = e.data[1];
                struct ore_rec_FieldAssignStmt __tmp_558;
                __tmp_558.object = obj;
                __tmp_558.field = field;
                __tmp_558.value = vid;
                return s_alloc_stmt(s, __tmp_558);
            }
            else {
                return (-(1LL));
                __tmp_556 = (int64_t)(s_error(s, ore_str_new("invalid assignment target", 25)));
            }
            __tmp_554 = (int64_t)(__tmp_556);
        } else {
        }
        struct ore_rec_ExprStmt __tmp_559;
        __tmp_559.expr = eid;
        __tmp_537 = (int64_t)(s_alloc_stmt(s, __tmp_559));
    }
    else {
        __tmp_537 = (int64_t)((-(1LL)));
    }
    return __tmp_537;
}

int64_t parse_expr(void* s, int64_t min_bp) {
    int64_t __tmp_560 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t lhs = parse_prefix(s);
    int64_t __tmp_561 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    while (1) {
        int64_t __tmp_562 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t __tmp_563 = 0;
        if (p_at(s, ore_str_new("?", 1))) {
            int64_t __tmp_564 = 0;
            if ((96070738580896 >= 96070659992880)) {
                struct ore_rec_TryExpr __tmp_565;
                __tmp_565.inner = lhs;
                lhs = s_alloc_expr(s, __tmp_565);
                goto cont_39;
                __tmp_564 = (int64_t)(p_skip(s));
            } else {
            }
            __tmp_563 = (int64_t)(__tmp_564);
        } else {
        }
        int64_t __tmp_566 = 0;
        if (p_at(s, ore_str_new("[", 1))) {
            int64_t __tmp_567 = 0;
            if ((96070738606304 >= 96070660015824)) {
                int64_t idx = parse_expr(s, 0LL);
                int64_t __tmp_568 = 0;
                if (s_has_error(s)) {
                    return (-(1LL));
                } else {
                }
                struct ore_rec_IndexExpr __tmp_569;
                __tmp_569.object = lhs;
                __tmp_569.index = idx;
                lhs = s_alloc_expr(s, __tmp_569);
                goto cont_39;
                __tmp_567 = (int64_t)(p_expect(s, ore_str_new("]", 1)));
            } else {
            }
            __tmp_566 = (int64_t)(__tmp_567);
        } else {
        }
        int8_t is_dot = p_at(s, ore_str_new(".", 1));
        int8_t is_qd = p_at(s, ore_str_new("?.", 2));
        int64_t __tmp_570 = 0;
        if ((96070660064096 || 96070660066208)) {
            int64_t __tmp_571 = 0;
            if (p_peek_at(s, 1LL).tag == 2) {
                int64_t _ = p_peek_at(s, 1LL).data[0];
                int64_t __tmp_572 = 0;
                if ((96070738678432 >= 96070660078272)) {
                    int64_t __tmp_573 = 0;
                    if (p_peek(s).tag == 2) {
                        int64_t f = p_peek(s).data[0];
                        __tmp_573 = (int64_t)(f);
                    }
                    else {
                        __tmp_573 = (int64_t)(ore_str_new("", 0));
                    }
                    int64_t field = __tmp_573;
                    int64_t __tmp_574 = 0;
                    if (p_at(s, ore_str_new("(", 1))) {
                        void* args = parse_call_args(s);
                        int64_t __tmp_575 = 0;
                        if (s_has_error(s)) {
                            return (-(1LL));
                        } else {
                        }
                        int64_t __tmp_576 = 0;
                        if (is_qd) {
                            struct ore_rec_OptionalMethodCallExpr __tmp_577;
                            __tmp_577.object = lhs;
                            __tmp_577.method = field;
                            __tmp_577.args = args;
                            lhs = s_alloc_expr(s, __tmp_577);
                        } else {
                            struct ore_rec_MethodCallExpr __tmp_578;
                            __tmp_578.object = lhs;
                            __tmp_578.method = field;
                            __tmp_578.args = args;
                            lhs = s_alloc_expr(s, __tmp_578);
                        }
                        __tmp_574 = (int64_t)(__tmp_576);
                    } else {
                        int64_t __tmp_579 = 0;
                        if (is_qd) {
                            struct ore_rec_OptionalChainExpr __tmp_580;
                            __tmp_580.object = lhs;
                            __tmp_580.field = field;
                            lhs = s_alloc_expr(s, __tmp_580);
                        } else {
                            struct ore_rec_FieldAccessExpr __tmp_581;
                            __tmp_581.object = lhs;
                            __tmp_581.field = field;
                            lhs = s_alloc_expr(s, __tmp_581);
                        }
                        __tmp_574 = (int64_t)(__tmp_579);
                    }
                    goto cont_39;
                    __tmp_572 = (int64_t)(__tmp_574);
                } else {
                }
                __tmp_571 = (int64_t)(__tmp_572);
            }
            else {
                __tmp_571 = (int64_t)(0LL);
            }
            __tmp_570 = (int64_t)(__tmp_571);
        } else {
        }
        int64_t otag = tok_to_op(p_peek(s));
        int64_t __tmp_582 = 0;
        if ((96070660205600 > 96070738824864)) {
            int64_t __tmp_583 = 0;
            if ((96070660208416 == 96070738828064)) {
                int64_t __tmp_584 = 0;
                if (p_peek_at(s, 1LL).tag == 11) {
                    int64_t __tmp_585 = 0;
                    if ((96070738840656 < 96070660219920)) {
                        goto brk_38;
                    } else {
                    }
                    int64_t did = parse_expr(s, 10LL);
                    int64_t __tmp_586 = 0;
                    if (s_has_error(s)) {
                        return (-(1LL));
                    } else {
                    }
                    struct ore_rec_MethodCallExpr __tmp_587;
                    __tmp_587.object = lhs;
                    __tmp_587.method = ore_str_new("unwrap_or", 9);
                    void* __tmp_588 = ore_list_new();
                    ore_list_push(__tmp_588, (int64_t)(did));
                    __tmp_587.args = __tmp_588;
                    lhs = s_alloc_expr(s, __tmp_587);
                    goto cont_39;
                    __tmp_584 = (int64_t)(__tmp_586);
                }
                else {
                    __tmp_584 = (int64_t)(0LL);
                }
                __tmp_583 = (int64_t)(__tmp_584);
            } else {
            }
            int64_t lbp = bp_l(otag);
            int64_t rbp = bp_r(otag);
            int64_t __tmp_589 = 0;
            if ((96070660274848 < 96070660276576)) {
                goto brk_38;
            } else {
            }
            struct ore_enum_BinOp op = int_to_op(otag);
            int64_t rhs = parse_expr(s, rbp);
            int64_t __tmp_590 = 0;
            if (s_has_error(s)) {
                return (-(1LL));
            } else {
            }
            struct ore_rec_BinOpExpr __tmp_591;
            __tmp_591.op = op;
            __tmp_591.left = lhs;
            __tmp_591.right = rhs;
            lhs = s_alloc_expr(s, __tmp_591);
            __tmp_582 = (int64_t)(__tmp_590);
        } else {
            goto brk_38;
        }
        cont_39: ;
    }
    brk_38: ;
    return lhs;
}

int64_t parse_prefix(void* s) {
    int64_t __tmp_592 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_593 = 0;
    if (p_peek(s).tag == 0) {
        int64_t n = p_peek(s).data[0];
        struct ore_rec_IntLitExpr __tmp_594;
        __tmp_594.value = n;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_594));
    }
    else if (p_peek(s).tag == 1) {
        int64_t f = p_peek(s).data[0];
        struct ore_rec_FloatLitExpr __tmp_595;
        __tmp_595.value = f;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_595));
    }
    else if (p_peek(s).tag == 12) {
        struct ore_rec_BoolLitExpr __tmp_596;
        __tmp_596.value = ((int8_t)1);
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_596));
    }
    else if (p_peek(s).tag == 13) {
        struct ore_rec_BoolLitExpr __tmp_597;
        __tmp_597.value = ((int8_t)0);
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_597));
    }
    else if (p_peek(s).tag == 3) {
        int64_t sv = p_peek(s).data[0];
        struct ore_rec_StringLitExpr __tmp_598;
        __tmp_598.value = sv;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_598));
    }
    else if (p_peek(s).tag == 4) {
        int64_t sv = p_peek(s).data[0];
        __tmp_593 = (int64_t)(parse_string_interp(s, sv));
    }
    else if (p_peek(s).tag == 37) {
        int64_t inner = parse_expr(s, 17LL);
        struct ore_rec_UnaryMinusExpr __tmp_599;
        __tmp_599.inner = inner;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_599));
    }
    else if (p_peek(s).tag == 64) {
        int64_t inner = parse_expr(s, 17LL);
        struct ore_rec_UnaryNotExpr __tmp_600;
        __tmp_600.inner = inner;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_600));
    }
    else if (p_peek(s).tag == 18) {
        struct ore_enum_Expr __tmp_601; __tmp_601.tag = 23;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_601));
    }
    else if (p_peek(s).tag == 24) {
        struct ore_enum_Expr __tmp_602; __tmp_602.tag = 24;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_602));
    }
    else if (p_peek(s).tag == 23) {
        int64_t inner = parse_expr(s, 0LL);
        struct ore_rec_OptionSomeExpr __tmp_603;
        __tmp_603.inner = inner;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_603));
    }
    else if (p_peek(s).tag == 25) {
        int64_t inner = parse_expr(s, 0LL);
        struct ore_rec_ResultOkExpr __tmp_604;
        __tmp_604.inner = inner;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_604));
    }
    else if (p_peek(s).tag == 26) {
        int64_t inner = parse_expr(s, 0LL);
        struct ore_rec_ResultErrExpr __tmp_605;
        __tmp_605.inner = inner;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_605));
    }
    else if (p_peek(s).tag == 30) {
        int64_t subj = parse_expr(s, 0LL);
        int64_t __tmp_606 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        void* arms = parse_match_arms(s);
        struct ore_rec_MatchExpr __tmp_607;
        __tmp_607.subject = subj;
        __tmp_607.arms = arms;
        __tmp_593 = (int64_t)(s_alloc_expr(s, __tmp_607));
    }
    else if (p_peek(s).tag == 9) {
        __tmp_593 = (int64_t)(parse_if_expr(s));
    }
    else if (p_peek(s).tag == 65) {
        __tmp_593 = (int64_t)(parse_paren_expr(s));
    }
    else if (p_peek(s).tag == 69) {
        __tmp_593 = (int64_t)(parse_list_lit(s));
    }
    else if (p_peek(s).tag == 67) {
        __tmp_593 = (int64_t)(parse_map_lit(s));
    }
    else if (p_peek(s).tag == 2) {
        int64_t _ = p_peek(s).data[0];
        __tmp_593 = (int64_t)(parse_ident_expr(s));
    }
    else {
        __tmp_593 = (int64_t)((-(1LL)));
    }
    return __tmp_593;
}

int64_t parse_string_interp(void* s, void* start) {
    void* __tmp_608 = ore_list_new();
    void* parts = __tmp_608;
    int64_t __tmp_609 = 0;
    if ((96070739411568 > 96070739411808)) {
        struct ore_rec_LitPart __tmp_610;
        __tmp_610.value = start;
        ore_list_push(parts, ({ struct ore_rec_LitPart __v2i = __tmp_610; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_LitPart)), &__v2i, sizeof(struct ore_rec_LitPart)); }));
    } else {
    }
    int64_t eid = parse_expr(s, 0LL);
    int64_t __tmp_611 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    struct ore_rec_ExprPart __tmp_612;
    __tmp_612.expr_id = eid;
    ore_list_push(parts, ({ struct ore_rec_ExprPart __v2i = __tmp_612; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_ExprPart)), &__v2i, sizeof(struct ore_rec_ExprPart)); }));
    while (1) {
        int64_t __tmp_613 = 0;
        if (p_peek(s).tag == 5) {
            int64_t sv = p_peek(s).data[0];
            int64_t __tmp_614 = 0;
            if ((96070739461344 > 96070739461568)) {
                struct ore_rec_LitPart __tmp_615;
                __tmp_615.value = sv;
                ore_list_push(parts, ({ struct ore_rec_LitPart __v2i = __tmp_615; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_LitPart)), &__v2i, sizeof(struct ore_rec_LitPart)); }));
            } else {
            }
            int64_t eid2 = parse_expr(s, 0LL);
            int64_t __tmp_616 = 0;
            if (s_has_error(s)) {
                return (-(1LL));
            } else {
            }
            struct ore_rec_ExprPart __tmp_617;
            __tmp_617.expr_id = eid2;
            ore_list_push(parts, ({ struct ore_rec_ExprPart __v2i = __tmp_617; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_ExprPart)), &__v2i, sizeof(struct ore_rec_ExprPart)); }));
            __tmp_613 = (int64_t)(__tmp_616);
        }
        else if (p_peek(s).tag == 6) {
            int64_t sv = p_peek(s).data[0];
            int64_t __tmp_618 = 0;
            if ((96070739507280 > 96070739507504)) {
                struct ore_rec_LitPart __tmp_619;
                __tmp_619.value = sv;
                ore_list_push(parts, ({ struct ore_rec_LitPart __v2i = __tmp_619; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_LitPart)), &__v2i, sizeof(struct ore_rec_LitPart)); }));
            } else {
            }
            goto brk_40;
            __tmp_613 = (int64_t)(__tmp_618);
        }
        else {
            return (-(1LL));
            __tmp_613 = (int64_t)(s_error(s, ore_str_new("expected string continuation", 28)));
        }
        cont_41: ;
    }
    brk_40: ;
    struct ore_rec_StringInterpExpr __tmp_620;
    __tmp_620.parts = parts;
    return s_alloc_expr(s, __tmp_620);
}

int64_t parse_if_expr(void* s) {
    int64_t cid = parse_expr(s, 0LL);
    int64_t __tmp_621 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_622 = 0;
    if (p_at(s, ore_str_new("then", 4))) {
        int64_t line = p_peek_line(s);
        int64_t then_id = parse_expr(s, 0LL);
        int64_t __tmp_623 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        struct ore_rec_ExprStmt __tmp_624;
        __tmp_624.expr = then_id;
        int64_t tsid = s_alloc_stmt(s, __tmp_624);
        struct ore_rec_Block __tmp_625;
        void* __tmp_626 = ore_list_new();
        struct ore_rec_SpannedStmt __tmp_627;
        __tmp_627.stmt_id = tsid;
        __tmp_627.line = line;
        ore_list_push(__tmp_626, ({ struct ore_rec_SpannedStmt __v2i = __tmp_627; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedStmt)), &__v2i, sizeof(struct ore_rec_SpannedStmt)); }));
        __tmp_625.stmts = __tmp_626;
        struct ore_rec_Block tblock = __tmp_625;
        int64_t saved_pos = s_pos(s);
        int64_t depth = 0LL;
        int8_t found_else = ((int8_t)0);
        while (1) {
            struct ore_enum_Token t = p_peek(s);
            int64_t __tmp_628 = 0;
            if (p_is_indent(t)) {
                depth = (96070660886944 + 96070739642048);
                __tmp_628 = (int64_t)(p_skip(s));
            } else {
                int64_t __tmp_629 = 0;
                if (p_is_dedent(t)) {
                    depth = (96070660899200 - 96070739656608);
                    __tmp_629 = (int64_t)(p_skip(s));
                } else {
                    int64_t __tmp_630 = 0;
                    if (p_is_newline(t)) {
                        __tmp_630 = (int64_t)(p_skip(s));
                    } else {
                        goto brk_42;
                    }
                    __tmp_629 = (int64_t)(__tmp_630);
                }
                __tmp_628 = (int64_t)(__tmp_629);
            }
            cont_43: ;
        }
        brk_42: ;
        int64_t __tmp_631 = 0;
        if (p_at(s, ore_str_new("else", 4))) {
            found_else = ((int8_t)1);
        } else {
        }
        int64_t __tmp_632 = 0;
        if ((!(found_else))) {
            struct ore_rec_IfElseExpr __tmp_633;
            __tmp_633.cond = cid;
            __tmp_633.then_block = tblock;
            struct ore_rec_Block __tmp_634;
            void* __tmp_635 = ore_list_new();
            __tmp_634.stmts = __tmp_635;
            __tmp_633.else_block = __tmp_634;
            return s_alloc_expr(s, __tmp_633);
            __tmp_632 = (int64_t)(s_set_pos(s, saved_pos));
        } else {
        }
        int64_t eline = p_peek_line(s);
        int64_t else_id = parse_expr(s, 0LL);
        int64_t __tmp_636 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        struct ore_rec_ExprStmt __tmp_637;
        __tmp_637.expr = else_id;
        int64_t esid = s_alloc_stmt(s, __tmp_637);
        struct ore_rec_Block __tmp_638;
        void* __tmp_639 = ore_list_new();
        struct ore_rec_SpannedStmt __tmp_640;
        __tmp_640.stmt_id = esid;
        __tmp_640.line = eline;
        ore_list_push(__tmp_639, ({ struct ore_rec_SpannedStmt __v2i = __tmp_640; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedStmt)), &__v2i, sizeof(struct ore_rec_SpannedStmt)); }));
        __tmp_638.stmts = __tmp_639;
        struct ore_rec_Block eblock = __tmp_638;
        int64_t i = 0LL;
        while ((96070661003600 < 96070661005296)) {
            int64_t __tmp_641 = 0;
            if (p_is_dedent(p_peek(s))) {
                __tmp_641 = (int64_t)(p_skip_nl(s));
            } else {
            }
            i = (96070661021456 + 96070739787152);
            cont_45: ;
        }
        brk_44: ;
        struct ore_rec_IfElseExpr __tmp_642;
        __tmp_642.cond = cid;
        __tmp_642.then_block = tblock;
        __tmp_642.else_block = eblock;
        return s_alloc_expr(s, __tmp_642);
        __tmp_622 = (int64_t)(p_skip_nl(s));
    } else {
    }
    void* __tmp_643 = ore_list_new();
    void* tstmts = __tmp_643;
    int64_t __tmp_644 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_645 = 0;
    if (p_at(s, ore_str_new("else", 4))) {
        int64_t __tmp_646 = 0;
        if (p_at(s, ore_str_new("if", 2))) {
            int64_t line = p_peek_line(s);
            int64_t nid = parse_expr(s, 0LL);
            int64_t __tmp_647 = 0;
            if (s_has_error(s)) {
                return (-(1LL));
            } else {
            }
            struct ore_rec_ExprStmt __tmp_648;
            __tmp_648.expr = nid;
            int64_t nsid = s_alloc_stmt(s, __tmp_648);
            struct ore_rec_IfElseExpr __tmp_649;
            __tmp_649.cond = cid;
            struct ore_rec_Block __tmp_650;
            __tmp_650.stmts = tstmts;
            __tmp_649.then_block = __tmp_650;
            struct ore_rec_Block __tmp_651;
            void* __tmp_652 = ore_list_new();
            struct ore_rec_SpannedStmt __tmp_653;
            __tmp_653.stmt_id = nsid;
            __tmp_653.line = line;
            ore_list_push(__tmp_652, ({ struct ore_rec_SpannedStmt __v2i = __tmp_653; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_SpannedStmt)), &__v2i, sizeof(struct ore_rec_SpannedStmt)); }));
            __tmp_651.stmts = __tmp_652;
            __tmp_649.else_block = __tmp_651;
            return s_alloc_expr(s, __tmp_649);
            __tmp_646 = (int64_t)(__tmp_647);
        } else {
        }
        void* __tmp_654 = ore_list_new();
        void* estmts = __tmp_654;
        struct ore_rec_IfElseExpr __tmp_655;
        __tmp_655.cond = cid;
        struct ore_rec_Block __tmp_656;
        __tmp_656.stmts = tstmts;
        __tmp_655.then_block = __tmp_656;
        struct ore_rec_Block __tmp_657;
        __tmp_657.stmts = estmts;
        __tmp_655.else_block = __tmp_657;
        return s_alloc_expr(s, __tmp_655);
        __tmp_645 = (int64_t)(parse_block(s, estmts));
    } else {
    }
    struct ore_rec_IfElseExpr __tmp_658;
    __tmp_658.cond = cid;
    struct ore_rec_Block __tmp_659;
    __tmp_659.stmts = tstmts;
    __tmp_658.then_block = __tmp_659;
    struct ore_rec_Block __tmp_660;
    void* __tmp_661 = ore_list_new();
    __tmp_660.stmts = __tmp_661;
    __tmp_658.else_block = __tmp_660;
    return s_alloc_expr(s, __tmp_658);
}

int64_t parse_paren_expr(void* s) {
    int64_t saved = s_pos(s);
    int64_t lam = try_lambda(s);
    int64_t __tmp_662 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_663 = 0;
    if (has_node(lam)) {
        return lam;
    } else {
    }
    int64_t eid = parse_expr(s, 0LL);
    int64_t __tmp_664 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    return eid;
}

int64_t try_lambda(void* s) {
    int64_t saved = s_pos(s);
    void* __tmp_665 = ore_list_new();
    void* params = __tmp_665;
    int64_t __tmp_666 = 0;
    if (p_peek(s).tag == 2) {
        int64_t name = p_peek(s).data[0];
        ore_list_push(params, (int64_t)(name));
        __tmp_666 = (int64_t)(p_skip(s));
    }
    else {
        return no_node();
        __tmp_666 = (int64_t)(s_set_pos(s, saved));
    }
    while (1) {
        int64_t __tmp_667 = 0;
        if (p_at(s, ore_str_new(",", 1))) {
            int64_t __tmp_668 = 0;
            if (p_peek(s).tag == 2) {
                int64_t name = p_peek(s).data[0];
                ore_list_push(params, (int64_t)(name));
                __tmp_668 = (int64_t)(p_skip(s));
            }
            else {
                return no_node();
                __tmp_668 = (int64_t)(s_set_pos(s, saved));
            }
            __tmp_667 = (int64_t)(__tmp_668);
        } else {
            int64_t __tmp_669 = 0;
            if (p_at(s, ore_str_new("=>", 2))) {
                goto brk_46;
            } else {
                int64_t __tmp_670 = 0;
                if (p_at(s, ore_str_new(")", 1))) {
                    int64_t __tmp_671 = 0;
                    if (p_at(s, ore_str_new("=>", 2))) {
                        int64_t bid = parse_lambda_body(s);
                        struct ore_rec_LambdaExpr __tmp_672;
                        __tmp_672.params = params;
                        __tmp_672.body = bid;
                        return s_alloc_expr(s, __tmp_672);
                        __tmp_671 = (int64_t)(p_skip(s));
                    } else {
                    }
                    return no_node();
                    __tmp_670 = (int64_t)(s_set_pos(s, saved));
                } else {
                    return no_node();
                    __tmp_670 = (int64_t)(s_set_pos(s, saved));
                }
                __tmp_669 = (int64_t)(__tmp_670);
            }
            __tmp_667 = (int64_t)(__tmp_669);
        }
        cont_47: ;
    }
    brk_46: ;
    int64_t bid = parse_lambda_body(s);
    int64_t __tmp_673 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    struct ore_rec_LambdaExpr __tmp_674;
    __tmp_674.params = params;
    __tmp_674.body = bid;
    return s_alloc_expr(s, __tmp_674);
}

int64_t parse_lambda_body(void* s) {
    int64_t __tmp_675 = 0;
    if (p_is_newline(p_peek(s))) {
        int64_t __tmp_676 = 0;
        if (p_is_indent(p_peek_at(s, 1LL))) {
            void* __tmp_677 = ore_list_new();
            void* bstmts = __tmp_677;
            struct ore_rec_BlockExprExpr __tmp_678;
            struct ore_rec_Block __tmp_679;
            __tmp_679.stmts = bstmts;
            __tmp_678.block = __tmp_679;
            return s_alloc_expr(s, __tmp_678);
            __tmp_676 = (int64_t)(parse_block(s, bstmts));
        } else {
        }
        __tmp_675 = (int64_t)(__tmp_676);
    } else {
    }
    return parse_expr(s, 0LL);
}

int64_t parse_list_lit(void* s) {
    void* __tmp_680 = ore_list_new();
    void* elems = __tmp_680;
    int64_t __tmp_681 = 0;
    if ((!(p_at(s, ore_str_new("]", 1))))) {
        ore_list_push(elems, (int64_t)(parse_expr(s, 0LL)));
        int64_t __tmp_682 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        while (1) {
            int64_t __tmp_683 = 0;
            if (p_at(s, ore_str_new(",", 1))) {
                int64_t __tmp_684 = 0;
                if (p_at(s, ore_str_new("]", 1))) {
                    goto brk_48;
                } else {
                }
                ore_list_push(elems, (int64_t)(parse_expr(s, 0LL)));
                int64_t __tmp_685 = 0;
                if (s_has_error(s)) {
                    return (-(1LL));
                } else {
                }
                __tmp_683 = (int64_t)(p_skip_ws(s));
            } else {
                goto brk_48;
            }
            cont_49: ;
        }
        brk_48: ;
        __tmp_681 = (int64_t)(p_skip_ws(s));
    } else {
    }
    struct ore_rec_ListLitExpr __tmp_686;
    __tmp_686.elements = elems;
    return s_alloc_expr(s, __tmp_686);
}

int64_t parse_map_lit(void* s) {
    void* __tmp_687 = ore_list_new();
    void* entries = __tmp_687;
    int64_t __tmp_688 = 0;
    if ((!(p_at(s, ore_str_new("rbrace", 6))))) {
        int64_t kid = parse_expr(s, 3LL);
        int64_t __tmp_689 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        int64_t vid = parse_expr(s, 0LL);
        int64_t __tmp_690 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        void* __tmp_691 = ore_list_new();
        ore_list_push(__tmp_691, (int64_t)(kid));
        ore_list_push(__tmp_691, (int64_t)(vid));
        ore_list_push(entries, (int64_t)(intptr_t)(__tmp_691));
        while (1) {
            int64_t __tmp_692 = 0;
            if (p_at(s, ore_str_new(",", 1))) {
                int64_t __tmp_693 = 0;
                if (p_at(s, ore_str_new("rbrace", 6))) {
                    goto brk_50;
                } else {
                }
                int64_t k2 = parse_expr(s, 3LL);
                int64_t __tmp_694 = 0;
                if (s_has_error(s)) {
                    return (-(1LL));
                } else {
                }
                int64_t v2 = parse_expr(s, 0LL);
                int64_t __tmp_695 = 0;
                if (s_has_error(s)) {
                    return (-(1LL));
                } else {
                }
                void* __tmp_696 = ore_list_new();
                ore_list_push(__tmp_696, (int64_t)(k2));
                ore_list_push(__tmp_696, (int64_t)(v2));
                ore_list_push(entries, (int64_t)(intptr_t)(__tmp_696));
                __tmp_692 = (int64_t)(p_skip_ws(s));
            } else {
                goto brk_50;
            }
            cont_51: ;
        }
        brk_50: ;
        __tmp_688 = (int64_t)(p_skip_ws(s));
    } else {
    }
    struct ore_rec_MapLitExpr __tmp_697;
    __tmp_697.entries = entries;
    return s_alloc_expr(s, __tmp_697);
}

int64_t parse_ident_expr(void* s) {
    int64_t __tmp_698 = 0;
    if (p_peek(s).tag == 2) {
        int64_t name = p_peek(s).data[0];
        int64_t __tmp_699 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070661705424), ore_str_new("print", 5))) {
            int64_t inner = parse_expr(s, 0LL);
            struct ore_rec_PrintExpr __tmp_700;
            __tmp_700.inner = inner;
            return s_alloc_expr(s, __tmp_700);
        } else {
        }
        int64_t __tmp_701 = 0;
        if (p_at(s, ore_str_new("=>", 2))) {
            int64_t bid = parse_lambda_body(s);
            struct ore_rec_LambdaExpr __tmp_702;
            void* __tmp_703 = ore_list_new();
            ore_list_push(__tmp_703, (int64_t)(name));
            __tmp_702.params = __tmp_703;
            __tmp_702.body = bid;
            return s_alloc_expr(s, __tmp_702);
            __tmp_701 = (int64_t)(p_skip(s));
        } else {
        }
        int64_t __tmp_704 = 0;
        if (p_at(s, ore_str_new("(", 1))) {
            int64_t saved = s_pos(s);
            int8_t is_upper = (96070740646320 && 96070740650608);
            int64_t __tmp_705 = 0;
            if (is_upper) {
                int64_t __tmp_706 = 0;
                if (p_peek(s).tag == 2) {
                    int64_t _ = p_peek(s).data[0];
                    int64_t __tmp_707 = 0;
                    if (p_peek_at(s, 1LL).tag == 52) {
                        void* __tmp_708 = ore_list_new();
                        void* fields = __tmp_708;
                        while (1) {
                            int64_t __tmp_709 = 0;
                            if (p_at(s, ore_str_new(")", 1))) {
                                goto brk_52;
                            } else {
                            }
                            void* fn_name = p_expect_ident(s, ore_str_new("field", 5));
                            int64_t __tmp_710 = 0;
                            if (s_has_error(s)) {
                                return (-(1LL));
                            } else {
                            }
                            int64_t __tmp_711 = 0;
                            if (s_has_error(s)) {
                                return (-(1LL));
                            } else {
                            }
                            int64_t fval = parse_expr(s, 0LL);
                            int64_t __tmp_712 = 0;
                            if (s_has_error(s)) {
                                return (-(1LL));
                            } else {
                            }
                            void* __tmp_713 = ore_list_new();
                            ore_list_push(__tmp_713, (int64_t)(intptr_t)(fn_name));
                            ore_list_push(__tmp_713, (int64_t)(fval));
                            ore_list_push(fields, (int64_t)(intptr_t)(__tmp_713));
                            int64_t __tmp_714 = 0;
                            if (p_at(s, ore_str_new(",", 1))) {
                                __tmp_714 = (int64_t)(p_skip_ws(s));
                            } else {
                            }
                            cont_53: ;
                        }
                        brk_52: ;
                        struct ore_rec_RecordConstructExpr __tmp_715;
                        __tmp_715.type_name = name;
                        __tmp_715.fields = fields;
                        return s_alloc_expr(s, __tmp_715);
                        __tmp_707 = (int64_t)(p_expect(s, ore_str_new(")", 1)));
                    }
                    else {
                        __tmp_707 = (int64_t)(0LL);
                    }
                    __tmp_706 = (int64_t)(__tmp_707);
                }
                else {
                    __tmp_706 = (int64_t)(0LL);
                }
                __tmp_705 = (int64_t)(__tmp_706);
            } else {
            }
            void* args = parse_call_args(s);
            int64_t __tmp_716 = 0;
            if (s_has_error(s)) {
                return (-(1LL));
            } else {
            }
            struct ore_rec_IdentExpr __tmp_717;
            __tmp_717.name = name;
            int64_t fid = s_alloc_expr(s, __tmp_717);
            struct ore_rec_CallExpr __tmp_718;
            __tmp_718.func = fid;
            __tmp_718.args = args;
            return s_alloc_expr(s, __tmp_718);
            __tmp_704 = (int64_t)(__tmp_716);
        } else {
        }
        struct ore_rec_IdentExpr __tmp_719;
        __tmp_719.name = name;
        __tmp_698 = (int64_t)(s_alloc_expr(s, __tmp_719));
    }
    else {
        __tmp_698 = (int64_t)((-(1LL)));
    }
    return __tmp_698;
}

void* parse_match_arms(void* s) {
    int64_t __tmp_720 = 0;
    if (s_has_error(s)) {
        void* __tmp_721 = ore_list_new();
        return __tmp_721;
    } else {
    }
    void* __tmp_722 = ore_list_new();
    void* arms = __tmp_722;
    while (1) {
        int64_t __tmp_723 = 0;
        if (p_at_block_end(s)) {
            goto brk_54;
        } else {
        }
        struct ore_rec_MatchArm arm = parse_match_arm(s);
        int64_t __tmp_724 = 0;
        if (s_has_error(s)) {
            void* __tmp_725 = ore_list_new();
            return __tmp_725;
        } else {
        }
        ore_list_push(arms, ({ struct ore_rec_MatchArm __v2i = arm; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_MatchArm)), &__v2i, sizeof(struct ore_rec_MatchArm)); }));
        cont_55: ;
    }
    brk_54: ;
    int64_t __tmp_726 = 0;
    if (p_at(s, ore_str_new("dedent", 6))) {
        __tmp_726 = (int64_t)(p_skip(s));
    } else {
    }
    return arms;
}

struct ore_rec_MatchArm parse_match_arm(void* s) {
    int64_t pid = parse_pattern(s);
    int64_t __tmp_727 = 0;
    if (s_has_error(s)) {
        struct ore_rec_MatchArm __tmp_728;
        struct ore_enum_Pattern __tmp_729; __tmp_729.tag = 1;
        __tmp_728.pattern = __tmp_729;
        __tmp_728.guard = (-(1LL));
        __tmp_728.body = (-(1LL));
        return __tmp_728;
    } else {
    }
    struct ore_enum_Pattern pat = get_pattern(s_patterns(s), pid);
    int64_t __tmp_730 = 0;
    if (p_at(s, ore_str_new("|", 1))) {
        void* __tmp_731 = ore_list_new();
        ore_list_push(__tmp_731, ({ struct ore_enum_Pattern __v2i = pat; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Pattern)), &__v2i, sizeof(struct ore_enum_Pattern)); }));
        void* alts = __tmp_731;
        while (1) {
            int64_t __tmp_732 = 0;
            if (p_at(s, ore_str_new("|", 1))) {
                int64_t aid = parse_pattern(s);
                int64_t __tmp_733 = 0;
                if (s_has_error(s)) {
                    struct ore_rec_MatchArm __tmp_734;
                    struct ore_enum_Pattern __tmp_735; __tmp_735.tag = 1;
                    __tmp_734.pattern = __tmp_735;
                    __tmp_734.guard = (-(1LL));
                    __tmp_734.body = (-(1LL));
                    return __tmp_734;
                } else {
                }
                ore_list_push(alts, ({ struct ore_enum_Pattern __v2i = get_pattern(s_patterns(s), aid); (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Pattern)), &__v2i, sizeof(struct ore_enum_Pattern)); }));
                __tmp_732 = (int64_t)(__tmp_733);
            } else {
                goto brk_56;
            }
            cont_57: ;
        }
        brk_56: ;
        struct ore_rec_OrPat __tmp_736;
        __tmp_736.alternatives = alts;
        pat = __tmp_736;
    } else {
    }
    int64_t __tmp_737 = 0;
    if (p_at(s, ore_str_new("if", 2))) {
        __tmp_737 = (int64_t)(parse_expr(s, 0LL));
    } else {
        __tmp_737 = (int64_t)(no_node());
    }
    int64_t gid = __tmp_737;
    int64_t __tmp_738 = 0;
    if (s_has_error(s)) {
        struct ore_rec_MatchArm __tmp_739;
        struct ore_enum_Pattern __tmp_740; __tmp_740.tag = 1;
        __tmp_739.pattern = __tmp_740;
        __tmp_739.guard = (-(1LL));
        __tmp_739.body = (-(1LL));
        return __tmp_739;
    } else {
    }
    int64_t __tmp_741 = 0;
    if (s_has_error(s)) {
        struct ore_rec_MatchArm __tmp_742;
        struct ore_enum_Pattern __tmp_743; __tmp_743.tag = 1;
        __tmp_742.pattern = __tmp_743;
        __tmp_742.guard = (-(1LL));
        __tmp_742.body = (-(1LL));
        return __tmp_742;
    } else {
    }
    struct ore_enum_Token t = p_peek(s);
    int64_t __tmp_744 = 0;
    if (p_is_newline(t)) {
        int64_t __tmp_745 = 0;
        if (p_is_indent(p_peek(s))) {
            void* __tmp_746 = ore_list_new();
            void* bstmts = __tmp_746;
            struct ore_rec_BlockExprExpr __tmp_747;
            struct ore_rec_Block __tmp_748;
            __tmp_748.stmts = bstmts;
            __tmp_747.block = __tmp_748;
            int64_t bid = s_alloc_expr(s, __tmp_747);
            struct ore_rec_MatchArm __tmp_749;
            __tmp_749.pattern = pat;
            __tmp_749.guard = gid;
            __tmp_749.body = bid;
            return __tmp_749;
            __tmp_745 = (int64_t)(parse_block(s, bstmts));
        } else {
        }
        __tmp_744 = (int64_t)(__tmp_745);
    } else {
    }
    int64_t bid = parse_expr(s, 0LL);
    struct ore_rec_MatchArm __tmp_750;
    __tmp_750.pattern = pat;
    __tmp_750.guard = gid;
    __tmp_750.body = bid;
    return __tmp_750;
}

int64_t parse_pattern(void* s) {
    int64_t __tmp_751 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_752 = 0;
    if (p_peek(s).tag == 2) {
        int64_t name = p_peek(s).data[0];
        int64_t __tmp_753 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070662258368), ore_str_new("_", 1))) {
            struct ore_enum_Pattern __tmp_754; __tmp_754.tag = 1;
            __tmp_753 = (int64_t)(s_alloc_pat(s, __tmp_754));
        } else {
            __tmp_753 = (int64_t)(parse_variant_pat(s, name));
        }
        __tmp_752 = (int64_t)(__tmp_753);
    }
    else if (p_peek(s).tag == 24) {
        struct ore_rec_VariantPat __tmp_755;
        __tmp_755.name = ore_str_new("None", 4);
        void* __tmp_756 = ore_list_new();
        __tmp_755.bindings = __tmp_756;
        __tmp_752 = (int64_t)(s_alloc_pat(s, __tmp_755));
    }
    else if (p_peek(s).tag == 23) {
        __tmp_752 = (int64_t)(parse_variant_pat(s, ore_str_new("Some", 4)));
    }
    else if (p_peek(s).tag == 25) {
        __tmp_752 = (int64_t)(parse_variant_pat(s, ore_str_new("Ok", 2)));
    }
    else if (p_peek(s).tag == 26) {
        __tmp_752 = (int64_t)(parse_variant_pat(s, ore_str_new("Err", 3)));
    }
    else if (p_peek(s).tag == 0) {
        int64_t n = p_peek(s).data[0];
        int64_t __tmp_757 = 0;
        if (p_at(s, ore_str_new("..", 2))) {
            int64_t __tmp_758 = 0;
            if (p_peek(s).tag == 0) {
                int64_t ev = p_peek(s).data[0];
                struct ore_rec_RangePat __tmp_759;
                __tmp_759.start = n;
                __tmp_759.end = ev;
                __tmp_758 = (int64_t)(s_alloc_pat(s, __tmp_759));
            }
            else {
                __tmp_758 = (int64_t)((-(1LL)));
            }
            __tmp_757 = (int64_t)(__tmp_758);
        } else {
            struct ore_rec_IntLitPat __tmp_760;
            __tmp_760.value = n;
            __tmp_757 = (int64_t)(s_alloc_pat(s, __tmp_760));
        }
        __tmp_752 = (int64_t)(__tmp_757);
    }
    else if (p_peek(s).tag == 1) {
        int64_t f = p_peek(s).data[0];
        struct ore_rec_FloatLitPat __tmp_761;
        __tmp_761.value = f;
        __tmp_752 = (int64_t)(s_alloc_pat(s, __tmp_761));
    }
    else if (p_peek(s).tag == 12) {
        struct ore_rec_BoolLitPat __tmp_762;
        __tmp_762.value = ((int8_t)1);
        __tmp_752 = (int64_t)(s_alloc_pat(s, __tmp_762));
    }
    else if (p_peek(s).tag == 13) {
        struct ore_rec_BoolLitPat __tmp_763;
        __tmp_763.value = ((int8_t)0);
        __tmp_752 = (int64_t)(s_alloc_pat(s, __tmp_763));
    }
    else if (p_peek(s).tag == 3) {
        int64_t sv = p_peek(s).data[0];
        struct ore_rec_StringLitPat __tmp_764;
        __tmp_764.value = sv;
        __tmp_752 = (int64_t)(s_alloc_pat(s, __tmp_764));
    }
    else if (p_peek(s).tag == 37) {
        int64_t __tmp_765 = 0;
        if (p_peek(s).tag == 0) {
            int64_t n = p_peek(s).data[0];
            struct ore_rec_IntLitPat __tmp_766;
            __tmp_766.value = (96070741463696 - 96070662467360);
            __tmp_765 = (int64_t)(s_alloc_pat(s, __tmp_766));
        }
        else if (p_peek(s).tag == 1) {
            int64_t f = p_peek(s).data[0];
            struct ore_rec_FloatLitPat __tmp_767;
            __tmp_767.value = (96070741484736 - 96070741486624);
            __tmp_765 = (int64_t)(s_alloc_pat(s, __tmp_767));
        }
        else {
            __tmp_765 = (int64_t)((-(1LL)));
        }
        __tmp_752 = (int64_t)(__tmp_765);
    }
    else {
        __tmp_752 = (int64_t)((-(1LL)));
    }
    return __tmp_752;
}

int64_t parse_variant_pat(void* s, void* name) {
    void* __tmp_768 = ore_list_new();
    void* bindings = __tmp_768;
    while (1) {
        int64_t __tmp_769 = 0;
        if (p_peek(s).tag == 2) {
            int64_t b = p_peek(s).data[0];
            int64_t __tmp_770 = 0;
            if (p_at(s, ore_str_new("->", 2))) {
                goto brk_58;
            } else {
            }
            ore_list_push(bindings, (int64_t)(b));
            __tmp_769 = (int64_t)(p_skip(s));
        }
        else {
            goto brk_58;
        }
        cont_59: ;
    }
    brk_58: ;
    struct ore_rec_VariantPat __tmp_771;
    __tmp_771.name = name;
    __tmp_771.bindings = bindings;
    return s_alloc_pat(s, __tmp_771);
}

int64_t parse_fn_def(void* s, void* out) {
    int64_t __tmp_772 = 0;
    if (s_has_error(s)) {
        return;
    } else {
    }
    void* name = p_expect_ident(s, ore_str_new("function", 8));
    int64_t __tmp_773 = 0;
    if (s_has_error(s)) {
        return;
    } else {
    }
    void* __tmp_774 = ore_list_new();
    void* params = __tmp_774;
    struct ore_rec_NamedType __tmp_775;
    __tmp_775.name = ore_str_new("", 0);
    struct ore_rec_NamedType ret_type = __tmp_775;
    int8_t has_ret = ((int8_t)0);
    while (1) {
        int64_t __tmp_776 = 0;
        if (p_at(s, ore_str_new("->", 2))) {
            int64_t rt_id = parse_type_expr(s);
            int64_t __tmp_777 = 0;
            if (s_has_error(s)) {
                return;
            } else {
            }
            ret_type = get_type(s_types(s), rt_id);
            has_ret = ((int8_t)1);
            goto brk_60;
            __tmp_776 = (int64_t)(__tmp_777);
        } else {
        }
        int64_t __tmp_778 = 0;
        if (p_peek(s).tag == 2) {
            int64_t _ = p_peek(s).data[0];
            void* __tmp_779 = ore_list_new();
            void* pout = __tmp_779;
            int64_t __tmp_780 = 0;
            if (s_has_error(s)) {
                return;
            } else {
            }
            int64_t __tmp_781 = 0;
            if ((96070741673584 > 96070741673824)) {
                int64_t __tmp_782 = ore_list_get(pout, 0LL);
                int8_t __tmp_783 = ore_list_get_kind(pout, 0LL);
                ore_list_push(params, (int64_t)(__tmp_782));
            } else {
            }
            __tmp_778 = (int64_t)(__tmp_781);
        }
        else {
            goto brk_60;
        }
        cont_61: ;
    }
    brk_60: ;
    void* __tmp_784 = ore_list_new();
    void* bstmts = __tmp_784;
    int64_t __tmp_785 = 0;
    if (s_has_error(s)) {
        return;
    } else {
    }
    struct ore_rec_FnDef __tmp_786;
    __tmp_786.name = name;
    void* __tmp_787 = ore_list_new();
    __tmp_786.type_params = __tmp_787;
    __tmp_786.params = params;
    __tmp_786.ret_type = ret_type;
    struct ore_rec_Block __tmp_788;
    __tmp_788.stmts = bstmts;
    __tmp_786.body = __tmp_788;
    ore_list_push(out, ({ struct ore_rec_FnDef __v2i = __tmp_786; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_FnDef)), &__v2i, sizeof(struct ore_rec_FnDef)); }));
    return __tmp_785;
}

int64_t parse_type_or_enum(void* s) {
    void* name = p_expect_ident(s, ore_str_new("type", 4));
    int64_t __tmp_789 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_790 = 0;
    if (p_at(s, ore_str_new("lbrace", 6))) {
        void* __tmp_791 = ore_list_new();
        void* fields = __tmp_791;
        while (1) {
            int64_t __tmp_792 = 0;
            if (p_at(s, ore_str_new("rbrace", 6))) {
                goto brk_62;
            } else {
            }
            void* fn_name = p_expect_ident(s, ore_str_new("field", 5));
            int64_t __tmp_793 = 0;
            if (s_has_error(s)) {
                return (-(1LL));
            } else {
            }
            int64_t __tmp_794 = 0;
            if (s_has_error(s)) {
                return (-(1LL));
            } else {
            }
            int64_t tid = parse_type_expr(s);
            int64_t __tmp_795 = 0;
            if (s_has_error(s)) {
                return (-(1LL));
            } else {
            }
            struct ore_enum_TypeExpr ty = get_type(s_types(s), tid);
            struct ore_rec_FieldDef __tmp_796;
            __tmp_796.name = fn_name;
            __tmp_796.ty = ty;
            ore_list_push(fields, ({ struct ore_rec_FieldDef __v2i = __tmp_796; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_FieldDef)), &__v2i, sizeof(struct ore_rec_FieldDef)); }));
            int64_t __tmp_797 = 0;
            if (p_at(s, ore_str_new(",", 1))) {
                __tmp_797 = (int64_t)(p_skip_ws(s));
            } else {
            }
            cont_63: ;
        }
        brk_62: ;
        struct ore_rec_TypeDefItem __tmp_798;
        struct ore_rec_TypeDefNode __tmp_799;
        __tmp_799.name = name;
        void* __tmp_800 = ore_list_new();
        __tmp_799.type_params = __tmp_800;
        __tmp_799.fields = fields;
        __tmp_798.type_def = __tmp_799;
        return s_alloc_item(s, __tmp_798);
        __tmp_790 = (int64_t)(p_expect(s, ore_str_new("rbrace", 6)));
    } else {
    }
    int64_t __tmp_801 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    void* __tmp_802 = ore_list_new();
    void* variants = __tmp_802;
    while (1) {
        int64_t __tmp_803 = 0;
        if (p_at_block_end(s)) {
            goto brk_64;
        } else {
        }
        void* vname = p_expect_ident(s, ore_str_new("variant", 7));
        int64_t __tmp_804 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        void* __tmp_805 = ore_list_new();
        void* vfields = __tmp_805;
        int64_t __tmp_806 = 0;
        if (p_at(s, ore_str_new("(", 1))) {
            while (1) {
                int64_t __tmp_807 = 0;
                if (p_at(s, ore_str_new(")", 1))) {
                    goto brk_66;
                } else {
                }
                void* vfn = p_expect_ident(s, ore_str_new("field", 5));
                int64_t __tmp_808 = 0;
                if (s_has_error(s)) {
                    return (-(1LL));
                } else {
                }
                int64_t __tmp_809 = 0;
                if (s_has_error(s)) {
                    return (-(1LL));
                } else {
                }
                int64_t vtid = parse_type_expr(s);
                int64_t __tmp_810 = 0;
                if (s_has_error(s)) {
                    return (-(1LL));
                } else {
                }
                struct ore_enum_TypeExpr vty = get_type(s_types(s), vtid);
                struct ore_rec_FieldDef __tmp_811;
                __tmp_811.name = vfn;
                __tmp_811.ty = vty;
                ore_list_push(vfields, ({ struct ore_rec_FieldDef __v2i = __tmp_811; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_FieldDef)), &__v2i, sizeof(struct ore_rec_FieldDef)); }));
                int64_t __tmp_812 = 0;
                if (p_at(s, ore_str_new(",", 1))) {
                    __tmp_812 = (int64_t)(p_skip(s));
                } else {
                }
                cont_67: ;
            }
            brk_66: ;
            __tmp_806 = (int64_t)(p_expect(s, ore_str_new(")", 1)));
        } else {
        }
        struct ore_rec_VariantDef __tmp_813;
        __tmp_813.name = vname;
        __tmp_813.fields = vfields;
        ore_list_push(variants, ({ struct ore_rec_VariantDef __v2i = __tmp_813; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_VariantDef)), &__v2i, sizeof(struct ore_rec_VariantDef)); }));
        cont_65: ;
    }
    brk_64: ;
    int64_t __tmp_814 = 0;
    if (p_at(s, ore_str_new("dedent", 6))) {
        __tmp_814 = (int64_t)(p_skip(s));
    } else {
    }
    struct ore_rec_EnumDefItem __tmp_815;
    struct ore_rec_EnumDefNode __tmp_816;
    __tmp_816.name = name;
    __tmp_816.variants = variants;
    __tmp_815.enum_def = __tmp_816;
    return s_alloc_item(s, __tmp_815);
}

int64_t parse_use_item(void* s) {
    int64_t __tmp_817 = 0;
    if (p_peek(s).tag == 3) {
        int64_t path = p_peek(s).data[0];
        struct ore_rec_UseItem __tmp_818;
        __tmp_818.path = path;
        __tmp_817 = (int64_t)(s_alloc_item(s, __tmp_818));
    }
    else if (p_peek(s).tag == 2) {
        int64_t name = p_peek(s).data[0];
        struct ore_rec_UseItem __tmp_819;
        __tmp_819.path = ore_str_concat(ore_int_to_str(name), ore_str_new(".ore", 4));
        __tmp_817 = (int64_t)(s_alloc_item(s, __tmp_819));
    }
    else {
        __tmp_817 = (int64_t)((-(1LL)));
    }
    return __tmp_817;
}

int64_t parse_test_def(void* s) {
    int64_t __tmp_820 = 0;
    if (p_peek(s).tag == 3) {
        int64_t sv = p_peek(s).data[0];
        void* __tmp_821 = ore_list_new();
        void* bstmts = __tmp_821;
        struct ore_rec_TestDefItem __tmp_822;
        __tmp_822.name = sv;
        struct ore_rec_Block __tmp_823;
        __tmp_823.stmts = bstmts;
        __tmp_822.body = __tmp_823;
        __tmp_820 = (int64_t)(s_alloc_item(s, __tmp_822));
    }
    else {
        __tmp_820 = (int64_t)((-(1LL)));
    }
    return __tmp_820;
}

int64_t parse_item(void* s) {
    int64_t __tmp_824 = 0;
    if (s_has_error(s)) {
        return (-(1LL));
    } else {
    }
    int64_t __tmp_825 = 0;
    if (p_peek(s).tag == 7) {
        void* __tmp_826 = ore_list_new();
        void* fout = __tmp_826;
        int64_t __tmp_827 = 0;
        if (s_has_error(s)) {
            return (-(1LL));
        } else {
        }
        struct ore_rec_FnDefItem __tmp_828;
        int64_t __tmp_829 = ore_list_get(fout, 0LL);
        int8_t __tmp_830 = ore_list_get_kind(fout, 0LL);
        __tmp_828.fn_def = __tmp_829;
        __tmp_825 = (int64_t)(s_alloc_item(s, __tmp_828));
    }
    else if (p_peek(s).tag == 20) {
        __tmp_825 = (int64_t)(parse_type_or_enum(s));
    }
    else if (p_peek(s).tag == 27) {
        __tmp_825 = (int64_t)(parse_use_item(s));
    }
    else if (p_peek(s).tag == 32) {
        __tmp_825 = (int64_t)(parse_test_def(s));
    }
    else {
        __tmp_825 = (int64_t)((-(1LL)));
    }
    return __tmp_825;
}

void* s_lines(void* s) {
    return s_get_list(s, 8LL);
}

void* s_cols(void* s) {
    return s_get_list(s, 9LL);
}

void* make_parse_state_split(void* split_data) {
    void* __tmp_831 = ore_list_new();
    void* s = __tmp_831;
    ore_list_push(s, (int64_t)(intptr_t)(s_get_list(split_data, 0LL)));
    void* __tmp_832 = ore_list_new();
    ore_list_push(__tmp_832, (int64_t)(0LL));
    ore_list_push(s, (int64_t)(intptr_t)(__tmp_832));
    void* __tmp_833 = ore_list_new();
    ore_list_push(s, (int64_t)(intptr_t)(__tmp_833));
    void* __tmp_834 = ore_list_new();
    ore_list_push(s, (int64_t)(intptr_t)(__tmp_834));
    void* __tmp_835 = ore_list_new();
    ore_list_push(s, (int64_t)(intptr_t)(__tmp_835));
    void* __tmp_836 = ore_list_new();
    ore_list_push(s, (int64_t)(intptr_t)(__tmp_836));
    void* __tmp_837 = ore_list_new();
    ore_list_push(s, (int64_t)(intptr_t)(__tmp_837));
    void* __tmp_838 = ore_list_new();
    ore_list_push(s, (int64_t)(intptr_t)(__tmp_838));
    ore_list_push(s, (int64_t)(intptr_t)(s_get_list(split_data, 1LL)));
    ore_list_push(s, (int64_t)(intptr_t)(s_get_list(split_data, 2LL)));
    return s;
}

int8_t parse_to_lists(void* split_data, void* result_holder) {
    void* s = make_parse_state_split(split_data);
    void* __tmp_839 = ore_list_new();
    void* iids = __tmp_839;
    while (1) {
        int64_t __tmp_840 = 0;
        if (p_is_eof(p_peek(s))) {
            goto brk_68;
        } else {
        }
        int64_t iid = parse_item(s);
        int64_t __tmp_841 = 0;
        if (s_has_error(s)) {
            void* errs = s_errors(s);
            int64_t __tmp_842 = ore_list_get(errs, 0LL);
            int8_t __tmp_843 = ore_list_get_kind(errs, 0LL);
            ore_list_push(result_holder, (int64_t)(__tmp_842));
            return ((int8_t)0);
        } else {
        }
        ore_list_push(iids, (int64_t)(iid));
        cont_69: ;
    }
    brk_68: ;
    void* __tmp_844 = ore_list_new();
    void* items = __tmp_844;
    for (int64_t i = 0LL; i < ore_list_len(iids); i++) {
        int64_t __tmp_845 = ore_list_get(iids, i);
        int8_t __tmp_846 = ore_list_get_kind(iids, i);
        int64_t idx = __tmp_845;
        ore_list_push(items, ({ struct ore_enum_Item __v2i = get_item(s_items(s), idx); (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_Item)), &__v2i, sizeof(struct ore_enum_Item)); }));
        cont_71: ;
    }
    brk_70: ;
    ore_list_push(result_holder, (int64_t)(intptr_t)(items));
    ore_list_push(result_holder, (int64_t)(intptr_t)(s_exprs(s)));
    ore_list_push(result_holder, (int64_t)(intptr_t)(s_stmts(s)));
    return ((int8_t)1);
}

struct ore_rec_Program parse(void* split_data) {
    void* __tmp_847 = ore_list_new();
    void* holder = __tmp_847;
    int8_t ok = parse_to_lists(split_data, holder);
    int64_t __tmp_848 = 0;
    if (ok) {
        void* items = s_get_list(holder, 0LL);
        struct ore_rec_Program __tmp_849;
        __tmp_849.items = items;
        __tmp_848 = (int64_t)(__tmp_849);
    } else {
        struct ore_rec_Program __tmp_850;
        void* __tmp_851 = ore_list_new();
        __tmp_850.items = __tmp_851;
        __tmp_848 = (int64_t)(__tmp_850);
    }
    return __tmp_848;
}

void* type_to_str(struct ore_enum_OreType t) {
    void* __tmp_852 = 0;
    if (t.tag == 0) {
        __tmp_852 = (void*)(ore_str_new("Int", 3));
    }
    else if (t.tag == 1) {
        __tmp_852 = (void*)(ore_str_new("Float", 5));
    }
    else if (t.tag == 2) {
        __tmp_852 = (void*)(ore_str_new("Bool", 4));
    }
    else if (t.tag == 3) {
        __tmp_852 = (void*)(ore_str_new("Str", 3));
    }
    else if (t.tag == 4) {
        __tmp_852 = (void*)(ore_str_new("Void", 4));
    }
    else if (t.tag == 9) {
        __tmp_852 = (void*)(ore_str_new("Channel", 7));
    }
    else if (t.tag == 7) {
        __tmp_852 = (void*)(ore_str_new("Option", 6));
    }
    else if (t.tag == 8) {
        __tmp_852 = (void*)(ore_str_new("Result", 6));
    }
    else if (t.tag == 10) {
        int64_t name = t.data[0];
        __tmp_852 = (void*)(name);
    }
    else if (t.tag == 11) {
        int64_t name = t.data[0];
        __tmp_852 = (void*)(name);
    }
    else {
        __tmp_852 = (void*)(ore_str_new("unknown", 7));
    }
    return __tmp_852;
}

struct ore_enum_OreType type_expr_to_ore_type(struct ore_enum_TypeExpr te) {
    struct ore_enum_OreType __tmp_853 = {0};
    if (te.tag == 0) {
        int64_t name = te.data[0];
        int64_t __tmp_854 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070663564544), ore_str_new("Int", 3))) {
            struct ore_enum_OreType __tmp_855; __tmp_855.tag = 0;
            __tmp_854 = (int64_t)(__tmp_855);
        } else {
            int64_t __tmp_856 = 0;
            if (ore_str_eq((void*)(intptr_t)(96070663570272), ore_str_new("Float", 5))) {
                struct ore_enum_OreType __tmp_857; __tmp_857.tag = 1;
                __tmp_856 = (int64_t)(__tmp_857);
            } else {
                int64_t __tmp_858 = 0;
                if (ore_str_eq((void*)(intptr_t)(96070663576256), ore_str_new("Bool", 4))) {
                    struct ore_enum_OreType __tmp_859; __tmp_859.tag = 2;
                    __tmp_858 = (int64_t)(__tmp_859);
                } else {
                    int64_t __tmp_860 = 0;
                    if (ore_str_eq((void*)(intptr_t)(96070663582112), ore_str_new("Str", 3))) {
                        struct ore_enum_OreType __tmp_861; __tmp_861.tag = 3;
                        __tmp_860 = (int64_t)(__tmp_861);
                    } else {
                        int64_t __tmp_862 = 0;
                        if (ore_str_eq((void*)(intptr_t)(96070663587840), ore_str_new("Void", 4))) {
                            struct ore_enum_OreType __tmp_863; __tmp_863.tag = 4;
                            __tmp_862 = (int64_t)(__tmp_863);
                        } else {
                            int64_t __tmp_864 = 0;
                            if (ore_str_eq((void*)(intptr_t)(96070663593696), ore_str_new("Option", 6))) {
                                struct ore_enum_OreType __tmp_865; __tmp_865.tag = 7;
                                __tmp_864 = (int64_t)(__tmp_865);
                            } else {
                                int64_t __tmp_866 = 0;
                                if (ore_str_eq((void*)(intptr_t)(96070663599808), ore_str_new("Result", 6))) {
                                    struct ore_enum_OreType __tmp_867; __tmp_867.tag = 8;
                                    __tmp_866 = (int64_t)(__tmp_867);
                                } else {
                                    int64_t __tmp_868 = 0;
                                    if (ore_str_eq((void*)(intptr_t)(96070663605920), ore_str_new("Channel", 7))) {
                                        struct ore_enum_OreType __tmp_869; __tmp_869.tag = 9;
                                        __tmp_868 = (int64_t)(__tmp_869);
                                    } else {
                                        int64_t __tmp_870 = 0;
                                        if (ore_str_eq((void*)(intptr_t)(96070663612160), ore_str_new("List", 4))) {
                                            struct ore_rec_ListOreType __tmp_871;
                                            struct ore_enum_OreType __tmp_872; __tmp_872.tag = 0;
                                            __tmp_871.elem = __tmp_872;
                                            __tmp_870 = (int64_t)(__tmp_871);
                                        } else {
                                            int64_t __tmp_873 = 0;
                                            if (ore_str_eq((void*)(intptr_t)(96070663621520), ore_str_new("Map", 3))) {
                                                struct ore_rec_MapOreType __tmp_874;
                                                struct ore_enum_OreType __tmp_875; __tmp_875.tag = 3;
                                                __tmp_874.key = __tmp_875;
                                                struct ore_enum_OreType __tmp_876; __tmp_876.tag = 0;
                                                __tmp_874.val = __tmp_876;
                                                __tmp_873 = (int64_t)(__tmp_874);
                                            } else {
                                                struct ore_rec_RecordOreType __tmp_877;
                                                __tmp_877.name = name;
                                                __tmp_873 = (int64_t)(__tmp_877);
                                            }
                                            __tmp_870 = (int64_t)(__tmp_873);
                                        }
                                        __tmp_868 = (int64_t)(__tmp_870);
                                    }
                                    __tmp_866 = (int64_t)(__tmp_868);
                                }
                                __tmp_864 = (int64_t)(__tmp_866);
                            }
                            __tmp_862 = (int64_t)(__tmp_864);
                        }
                        __tmp_860 = (int64_t)(__tmp_862);
                    }
                    __tmp_858 = (int64_t)(__tmp_860);
                }
                __tmp_856 = (int64_t)(__tmp_858);
            }
            __tmp_854 = (int64_t)(__tmp_856);
        }
        __tmp_853 = __tmp_854;
    }
    else {
        struct ore_enum_OreType __tmp_878; __tmp_878.tag = 0;
        __tmp_853 = __tmp_878;
    }
    return __tmp_853;
}

int8_t types_equal(struct ore_enum_OreType a, struct ore_enum_OreType b) {
    return ore_str_eq(type_to_str(a), type_to_str(b));
}

struct ore_rec_VariantDef get_variant_def(void* pool, int64_t idx) {
    int64_t __tmp_879 = ore_list_get(pool, idx);
    int8_t __tmp_880 = ore_list_get_kind(pool, idx);
    return *(struct ore_rec_VariantDef*)(intptr_t)(__tmp_879);
}

struct ore_rec_ParamDef get_param_def(void* pool, int64_t idx) {
    int64_t __tmp_881 = ore_list_get(pool, idx);
    int8_t __tmp_882 = ore_list_get_kind(pool, idx);
    return *(struct ore_rec_ParamDef*)(intptr_t)(__tmp_881);
}

struct ore_rec_FnDef get_fn_def(void* pool, int64_t idx) {
    int64_t __tmp_883 = ore_list_get(pool, idx);
    int8_t __tmp_884 = ore_list_get_kind(pool, idx);
    return *(struct ore_rec_FnDef*)(intptr_t)(__tmp_883);
}

struct ore_rec_SpannedStmt get_sspanned(void* pool, int64_t idx) {
    int64_t __tmp_885 = ore_list_get(pool, idx);
    int8_t __tmp_886 = ore_list_get_kind(pool, idx);
    return *(struct ore_rec_SpannedStmt*)(intptr_t)(__tmp_885);
}

struct ore_rec_MatchArm get_match_arm_typed(void* pool, int64_t idx) {
    int64_t __tmp_887 = ore_list_get(pool, idx);
    int8_t __tmp_888 = ore_list_get_kind(pool, idx);
    return *(struct ore_rec_MatchArm*)(intptr_t)(__tmp_887);
}

struct ore_enum_Item get_item_typed(void* pool, int64_t idx) {
    int64_t __tmp_889 = ore_list_get(pool, idx);
    int8_t __tmp_890 = ore_list_get_kind(pool, idx);
    return *(struct ore_enum_Item*)(intptr_t)(__tmp_889);
}

struct ore_enum_StringPart get_string_part(void* pool, int64_t idx) {
    int64_t __tmp_891 = ore_list_get(pool, idx);
    int8_t __tmp_892 = ore_list_get_kind(pool, idx);
    return *(struct ore_enum_StringPart*)(intptr_t)(__tmp_891);
}

void* int_to_str(int64_t n) {
    return ore_int_to_str(n);
}

void* make_method_set() {
    void* __tmp_893 = ore_map_new();
    void* s = __tmp_893;
    void* __tmp_894 = ore_list_new();
    ore_list_push(__tmp_894, (int64_t)(intptr_t)(ore_str_new("map", 3)));
    ore_list_push(__tmp_894, (int64_t)(intptr_t)(ore_str_new("filter", 6)));
    ore_list_push(__tmp_894, (int64_t)(intptr_t)(ore_str_new("each", 4)));
    ore_list_push(__tmp_894, (int64_t)(intptr_t)(ore_str_new("reduce", 6)));
    ore_list_push(__tmp_894, (int64_t)(intptr_t)(ore_str_new("fold", 4)));
    ore_list_push(__tmp_894, (int64_t)(intptr_t)(ore_str_new("scan", 4)));
    ore_list_push(__tmp_894, (int64_t)(intptr_t)(ore_str_new("join", 4)));
    void* methods = __tmp_894;
    for (int64_t i = 0LL; i < ore_list_len(methods); i++) {
        int64_t __tmp_895 = ore_list_get(methods, i);
        int8_t __tmp_896 = ore_list_get_kind(methods, i);
        ore_map_set(s, __tmp_895, (int64_t)(((int8_t)1)));
        cont_73: ;
    }
    brk_72: ;
    void* __tmp_897 = ore_list_new();
    ore_list_push(__tmp_897, (int64_t)(intptr_t)(ore_str_new("sort", 4)));
    ore_list_push(__tmp_897, (int64_t)(intptr_t)(ore_str_new("sort_by", 7)));
    ore_list_push(__tmp_897, (int64_t)(intptr_t)(ore_str_new("sort_by_key", 11)));
    ore_list_push(__tmp_897, (int64_t)(intptr_t)(ore_str_new("reverse", 7)));
    ore_list_push(__tmp_897, (int64_t)(intptr_t)(ore_str_new("unique", 6)));
    ore_list_push(__tmp_897, (int64_t)(intptr_t)(ore_str_new("dedup", 5)));
    void* m2 = __tmp_897;
    for (int64_t i = 0LL; i < ore_list_len(m2); i++) {
        int64_t __tmp_898 = ore_list_get(m2, i);
        int8_t __tmp_899 = ore_list_get_kind(m2, i);
        ore_map_set(s, __tmp_898, (int64_t)(((int8_t)1)));
        cont_75: ;
    }
    brk_74: ;
    void* __tmp_900 = ore_list_new();
    ore_list_push(__tmp_900, (int64_t)(intptr_t)(ore_str_new("take", 4)));
    ore_list_push(__tmp_900, (int64_t)(intptr_t)(ore_str_new("skip", 4)));
    ore_list_push(__tmp_900, (int64_t)(intptr_t)(ore_str_new("take_while", 10)));
    ore_list_push(__tmp_900, (int64_t)(intptr_t)(ore_str_new("drop_while", 10)));
    ore_list_push(__tmp_900, (int64_t)(intptr_t)(ore_str_new("step", 4)));
    void* m3 = __tmp_900;
    for (int64_t i = 0LL; i < ore_list_len(m3); i++) {
        int64_t __tmp_901 = ore_list_get(m3, i);
        int8_t __tmp_902 = ore_list_get_kind(m3, i);
        ore_map_set(s, __tmp_901, (int64_t)(((int8_t)1)));
        cont_77: ;
    }
    brk_76: ;
    void* __tmp_903 = ore_list_new();
    ore_list_push(__tmp_903, (int64_t)(intptr_t)(ore_str_new("flatten", 7)));
    ore_list_push(__tmp_903, (int64_t)(intptr_t)(ore_str_new("flat_map", 8)));
    ore_list_push(__tmp_903, (int64_t)(intptr_t)(ore_str_new("zip", 3)));
    ore_list_push(__tmp_903, (int64_t)(intptr_t)(ore_str_new("zip_with", 8)));
    ore_list_push(__tmp_903, (int64_t)(intptr_t)(ore_str_new("enumerate", 9)));
    void* m4 = __tmp_903;
    for (int64_t i = 0LL; i < ore_list_len(m4); i++) {
        int64_t __tmp_904 = ore_list_get(m4, i);
        int8_t __tmp_905 = ore_list_get_kind(m4, i);
        ore_map_set(s, __tmp_904, (int64_t)(((int8_t)1)));
        cont_79: ;
    }
    brk_78: ;
    void* __tmp_906 = ore_list_new();
    ore_list_push(__tmp_906, (int64_t)(intptr_t)(ore_str_new("window", 6)));
    ore_list_push(__tmp_906, (int64_t)(intptr_t)(ore_str_new("chunks", 6)));
    ore_list_push(__tmp_906, (int64_t)(intptr_t)(ore_str_new("intersperse", 11)));
    ore_list_push(__tmp_906, (int64_t)(intptr_t)(ore_str_new("partition", 9)));
    ore_list_push(__tmp_906, (int64_t)(intptr_t)(ore_str_new("group_by", 8)));
    void* m5 = __tmp_906;
    for (int64_t i = 0LL; i < ore_list_len(m5); i++) {
        int64_t __tmp_907 = ore_list_get(m5, i);
        int8_t __tmp_908 = ore_list_get_kind(m5, i);
        ore_map_set(s, __tmp_907, (int64_t)(((int8_t)1)));
        cont_81: ;
    }
    brk_80: ;
    void* __tmp_909 = ore_list_new();
    ore_list_push(__tmp_909, (int64_t)(intptr_t)(ore_str_new("count_by", 8)));
    ore_list_push(__tmp_909, (int64_t)(intptr_t)(ore_str_new("frequencies", 11)));
    ore_list_push(__tmp_909, (int64_t)(intptr_t)(ore_str_new("to_map", 6)));
    ore_list_push(__tmp_909, (int64_t)(intptr_t)(ore_str_new("first", 5)));
    ore_list_push(__tmp_909, (int64_t)(intptr_t)(ore_str_new("last", 4)));
    void* m6 = __tmp_909;
    for (int64_t i = 0LL; i < ore_list_len(m6); i++) {
        int64_t __tmp_910 = ore_list_get(m6, i);
        int8_t __tmp_911 = ore_list_get_kind(m6, i);
        ore_map_set(s, __tmp_910, (int64_t)(((int8_t)1)));
        cont_83: ;
    }
    brk_82: ;
    void* __tmp_912 = ore_list_new();
    ore_list_push(__tmp_912, (int64_t)(intptr_t)(ore_str_new("sum", 3)));
    ore_list_push(__tmp_912, (int64_t)(intptr_t)(ore_str_new("product", 7)));
    ore_list_push(__tmp_912, (int64_t)(intptr_t)(ore_str_new("min", 3)));
    ore_list_push(__tmp_912, (int64_t)(intptr_t)(ore_str_new("max", 3)));
    ore_list_push(__tmp_912, (int64_t)(intptr_t)(ore_str_new("average", 7)));
    ore_list_push(__tmp_912, (int64_t)(intptr_t)(ore_str_new("any", 3)));
    ore_list_push(__tmp_912, (int64_t)(intptr_t)(ore_str_new("all", 3)));
    void* m7 = __tmp_912;
    for (int64_t i = 0LL; i < ore_list_len(m7); i++) {
        int64_t __tmp_913 = ore_list_get(m7, i);
        int8_t __tmp_914 = ore_list_get_kind(m7, i);
        ore_map_set(s, __tmp_913, (int64_t)(((int8_t)1)));
        cont_85: ;
    }
    brk_84: ;
    void* __tmp_915 = ore_list_new();
    ore_list_push(__tmp_915, (int64_t)(intptr_t)(ore_str_new("find", 4)));
    ore_list_push(__tmp_915, (int64_t)(intptr_t)(ore_str_new("find_index", 10)));
    ore_list_push(__tmp_915, (int64_t)(intptr_t)(ore_str_new("index_of", 8)));
    ore_list_push(__tmp_915, (int64_t)(intptr_t)(ore_str_new("contains", 8)));
    void* m8 = __tmp_915;
    for (int64_t i = 0LL; i < ore_list_len(m8); i++) {
        int64_t __tmp_916 = ore_list_get(m8, i);
        int8_t __tmp_917 = ore_list_get_kind(m8, i);
        ore_map_set(s, __tmp_916, (int64_t)(((int8_t)1)));
        cont_87: ;
    }
    brk_86: ;
    void* __tmp_918 = ore_list_new();
    ore_list_push(__tmp_918, (int64_t)(intptr_t)(ore_str_new("push", 4)));
    ore_list_push(__tmp_918, (int64_t)(intptr_t)(ore_str_new("pop", 3)));
    ore_list_push(__tmp_918, (int64_t)(intptr_t)(ore_str_new("insert", 6)));
    ore_list_push(__tmp_918, (int64_t)(intptr_t)(ore_str_new("remove_at", 9)));
    ore_list_push(__tmp_918, (int64_t)(intptr_t)(ore_str_new("clear", 5)));
    ore_list_push(__tmp_918, (int64_t)(intptr_t)(ore_str_new("set", 3)));
    void* m9 = __tmp_918;
    for (int64_t i = 0LL; i < ore_list_len(m9); i++) {
        int64_t __tmp_919 = ore_list_get(m9, i);
        int8_t __tmp_920 = ore_list_get_kind(m9, i);
        ore_map_set(s, __tmp_919, (int64_t)(((int8_t)1)));
        cont_89: ;
    }
    brk_88: ;
    void* __tmp_921 = ore_list_new();
    ore_list_push(__tmp_921, (int64_t)(intptr_t)(ore_str_new("get", 3)));
    ore_list_push(__tmp_921, (int64_t)(intptr_t)(ore_str_new("get_or", 6)));
    ore_list_push(__tmp_921, (int64_t)(intptr_t)(ore_str_new("len", 3)));
    ore_list_push(__tmp_921, (int64_t)(intptr_t)(ore_str_new("is_empty", 8)));
    ore_list_push(__tmp_921, (int64_t)(intptr_t)(ore_str_new("slice", 5)));
    void* m10 = __tmp_921;
    for (int64_t i = 0LL; i < ore_list_len(m10); i++) {
        int64_t __tmp_922 = ore_list_get(m10, i);
        int8_t __tmp_923 = ore_list_get_kind(m10, i);
        ore_map_set(s, __tmp_922, (int64_t)(((int8_t)1)));
        cont_91: ;
    }
    brk_90: ;
    void* __tmp_924 = ore_list_new();
    ore_list_push(__tmp_924, (int64_t)(intptr_t)(ore_str_new("to_upper", 8)));
    ore_list_push(__tmp_924, (int64_t)(intptr_t)(ore_str_new("to_lower", 8)));
    ore_list_push(__tmp_924, (int64_t)(intptr_t)(ore_str_new("trim", 4)));
    ore_list_push(__tmp_924, (int64_t)(intptr_t)(ore_str_new("trim_start", 10)));
    ore_list_push(__tmp_924, (int64_t)(intptr_t)(ore_str_new("trim_end", 8)));
    void* m11 = __tmp_924;
    for (int64_t i = 0LL; i < ore_list_len(m11); i++) {
        int64_t __tmp_925 = ore_list_get(m11, i);
        int8_t __tmp_926 = ore_list_get_kind(m11, i);
        ore_map_set(s, __tmp_925, (int64_t)(((int8_t)1)));
        cont_93: ;
    }
    brk_92: ;
    void* __tmp_927 = ore_list_new();
    ore_list_push(__tmp_927, (int64_t)(intptr_t)(ore_str_new("split", 5)));
    ore_list_push(__tmp_927, (int64_t)(intptr_t)(ore_str_new("replace", 7)));
    ore_list_push(__tmp_927, (int64_t)(intptr_t)(ore_str_new("starts_with", 11)));
    ore_list_push(__tmp_927, (int64_t)(intptr_t)(ore_str_new("ends_with", 9)));
    ore_list_push(__tmp_927, (int64_t)(intptr_t)(ore_str_new("repeat", 6)));
    void* m12 = __tmp_927;
    for (int64_t i = 0LL; i < ore_list_len(m12); i++) {
        int64_t __tmp_928 = ore_list_get(m12, i);
        int8_t __tmp_929 = ore_list_get_kind(m12, i);
        ore_map_set(s, __tmp_928, (int64_t)(((int8_t)1)));
        cont_95: ;
    }
    brk_94: ;
    void* __tmp_930 = ore_list_new();
    ore_list_push(__tmp_930, (int64_t)(intptr_t)(ore_str_new("capitalize", 10)));
    ore_list_push(__tmp_930, (int64_t)(intptr_t)(ore_str_new("count", 5)));
    ore_list_push(__tmp_930, (int64_t)(intptr_t)(ore_str_new("strip_prefix", 12)));
    ore_list_push(__tmp_930, (int64_t)(intptr_t)(ore_str_new("strip_suffix", 12)));
    void* m13 = __tmp_930;
    for (int64_t i = 0LL; i < ore_list_len(m13); i++) {
        int64_t __tmp_931 = ore_list_get(m13, i);
        int8_t __tmp_932 = ore_list_get_kind(m13, i);
        ore_map_set(s, __tmp_931, (int64_t)(((int8_t)1)));
        cont_97: ;
    }
    brk_96: ;
    void* __tmp_933 = ore_list_new();
    ore_list_push(__tmp_933, (int64_t)(intptr_t)(ore_str_new("substr", 6)));
    ore_list_push(__tmp_933, (int64_t)(intptr_t)(ore_str_new("chars", 5)));
    ore_list_push(__tmp_933, (int64_t)(intptr_t)(ore_str_new("char_at", 7)));
    ore_list_push(__tmp_933, (int64_t)(intptr_t)(ore_str_new("pad_left", 8)));
    ore_list_push(__tmp_933, (int64_t)(intptr_t)(ore_str_new("pad_right", 9)));
    void* m14 = __tmp_933;
    for (int64_t i = 0LL; i < ore_list_len(m14); i++) {
        int64_t __tmp_934 = ore_list_get(m14, i);
        int8_t __tmp_935 = ore_list_get_kind(m14, i);
        ore_map_set(s, __tmp_934, (int64_t)(((int8_t)1)));
        cont_99: ;
    }
    brk_98: ;
    void* __tmp_936 = ore_list_new();
    ore_list_push(__tmp_936, (int64_t)(intptr_t)(ore_str_new("words", 5)));
    ore_list_push(__tmp_936, (int64_t)(intptr_t)(ore_str_new("lines", 5)));
    ore_list_push(__tmp_936, (int64_t)(intptr_t)(ore_str_new("to_int", 6)));
    ore_list_push(__tmp_936, (int64_t)(intptr_t)(ore_str_new("to_float", 8)));
    ore_list_push(__tmp_936, (int64_t)(intptr_t)(ore_str_new("to_str", 6)));
    void* m15 = __tmp_936;
    for (int64_t i = 0LL; i < ore_list_len(m15); i++) {
        int64_t __tmp_937 = ore_list_get(m15, i);
        int8_t __tmp_938 = ore_list_get_kind(m15, i);
        ore_map_set(s, __tmp_937, (int64_t)(((int8_t)1)));
        cont_101: ;
    }
    brk_100: ;
    void* __tmp_939 = ore_list_new();
    ore_list_push(__tmp_939, (int64_t)(intptr_t)(ore_str_new("parse_int", 9)));
    ore_list_push(__tmp_939, (int64_t)(intptr_t)(ore_str_new("parse_float", 11)));
    ore_list_push(__tmp_939, (int64_t)(intptr_t)(ore_str_new("abs", 3)));
    ore_list_push(__tmp_939, (int64_t)(intptr_t)(ore_str_new("floor", 5)));
    ore_list_push(__tmp_939, (int64_t)(intptr_t)(ore_str_new("ceil", 4)));
    void* m16 = __tmp_939;
    for (int64_t i = 0LL; i < ore_list_len(m16); i++) {
        int64_t __tmp_940 = ore_list_get(m16, i);
        int8_t __tmp_941 = ore_list_get_kind(m16, i);
        ore_map_set(s, __tmp_940, (int64_t)(((int8_t)1)));
        cont_103: ;
    }
    brk_102: ;
    void* __tmp_942 = ore_list_new();
    ore_list_push(__tmp_942, (int64_t)(intptr_t)(ore_str_new("round", 5)));
    ore_list_push(__tmp_942, (int64_t)(intptr_t)(ore_str_new("sqrt", 4)));
    ore_list_push(__tmp_942, (int64_t)(intptr_t)(ore_str_new("pow", 3)));
    ore_list_push(__tmp_942, (int64_t)(intptr_t)(ore_str_new("clamp", 5)));
    ore_list_push(__tmp_942, (int64_t)(intptr_t)(ore_str_new("unwrap_or", 9)));
    void* m17 = __tmp_942;
    for (int64_t i = 0LL; i < ore_list_len(m17); i++) {
        int64_t __tmp_943 = ore_list_get(m17, i);
        int8_t __tmp_944 = ore_list_get_kind(m17, i);
        ore_map_set(s, __tmp_943, (int64_t)(((int8_t)1)));
        cont_105: ;
    }
    brk_104: ;
    void* __tmp_945 = ore_list_new();
    ore_list_push(__tmp_945, (int64_t)(intptr_t)(ore_str_new("merge", 5)));
    ore_list_push(__tmp_945, (int64_t)(intptr_t)(ore_str_new("keys", 4)));
    ore_list_push(__tmp_945, (int64_t)(intptr_t)(ore_str_new("values", 6)));
    ore_list_push(__tmp_945, (int64_t)(intptr_t)(ore_str_new("entries", 7)));
    ore_list_push(__tmp_945, (int64_t)(intptr_t)(ore_str_new("send", 4)));
    ore_list_push(__tmp_945, (int64_t)(intptr_t)(ore_str_new("recv", 4)));
    void* m18 = __tmp_945;
    for (int64_t i = 0LL; i < ore_list_len(m18); i++) {
        int64_t __tmp_946 = ore_list_get(m18, i);
        int8_t __tmp_947 = ore_list_get_kind(m18, i);
        ore_map_set(s, __tmp_946, (int64_t)(((int8_t)1)));
        cont_107: ;
    }
    brk_106: ;
    void* __tmp_948 = ore_list_new();
    ore_list_push(__tmp_948, (int64_t)(intptr_t)(ore_str_new("tap", 3)));
    ore_list_push(__tmp_948, (int64_t)(intptr_t)(ore_str_new("format", 6)));
    ore_list_push(__tmp_948, (int64_t)(intptr_t)(ore_str_new("is_some", 7)));
    ore_list_push(__tmp_948, (int64_t)(intptr_t)(ore_str_new("is_none", 7)));
    ore_list_push(__tmp_948, (int64_t)(intptr_t)(ore_str_new("is_ok", 5)));
    ore_list_push(__tmp_948, (int64_t)(intptr_t)(ore_str_new("is_err", 6)));
    void* m19 = __tmp_948;
    for (int64_t i = 0LL; i < ore_list_len(m19); i++) {
        int64_t __tmp_949 = ore_list_get(m19, i);
        int8_t __tmp_950 = ore_list_get_kind(m19, i);
        ore_map_set(s, __tmp_949, (int64_t)(((int8_t)1)));
        cont_109: ;
    }
    brk_108: ;
    void* __tmp_951 = ore_list_new();
    ore_list_push(__tmp_951, (int64_t)(intptr_t)(ore_str_new("map_with_index", 14)));
    ore_list_push(__tmp_951, (int64_t)(intptr_t)(ore_str_new("each_with_index", 15)));
    ore_list_push(__tmp_951, (int64_t)(intptr_t)(ore_str_new("par_map", 7)));
    ore_list_push(__tmp_951, (int64_t)(intptr_t)(ore_str_new("par_each", 8)));
    void* m20 = __tmp_951;
    for (int64_t i = 0LL; i < ore_list_len(m20); i++) {
        int64_t __tmp_952 = ore_list_get(m20, i);
        int8_t __tmp_953 = ore_list_get_kind(m20, i);
        ore_map_set(s, __tmp_952, (int64_t)(((int8_t)1)));
        cont_111: ;
    }
    brk_110: ;
    return s;
}

void* make_builtin_set() {
    void* __tmp_954 = ore_map_new();
    void* s = __tmp_954;
    void* __tmp_955 = ore_list_new();
    ore_list_push(__tmp_955, (int64_t)(intptr_t)(ore_str_new("print", 5)));
    ore_list_push(__tmp_955, (int64_t)(intptr_t)(ore_str_new("exit", 4)));
    ore_list_push(__tmp_955, (int64_t)(intptr_t)(ore_str_new("args", 4)));
    ore_list_push(__tmp_955, (int64_t)(intptr_t)(ore_str_new("input", 5)));
    ore_list_push(__tmp_955, (int64_t)(intptr_t)(ore_str_new("type_of", 7)));
    void* b1 = __tmp_955;
    for (int64_t i = 0LL; i < ore_list_len(b1); i++) {
        int64_t __tmp_956 = ore_list_get(b1, i);
        int8_t __tmp_957 = ore_list_get_kind(b1, i);
        ore_map_set(s, __tmp_956, (int64_t)(((int8_t)1)));
        cont_113: ;
    }
    brk_112: ;
    void* __tmp_958 = ore_list_new();
    ore_list_push(__tmp_958, (int64_t)(intptr_t)(ore_str_new("to_str", 6)));
    ore_list_push(__tmp_958, (int64_t)(intptr_t)(ore_str_new("to_int", 6)));
    ore_list_push(__tmp_958, (int64_t)(intptr_t)(ore_str_new("to_float", 8)));
    void* b2 = __tmp_958;
    for (int64_t i = 0LL; i < ore_list_len(b2); i++) {
        int64_t __tmp_959 = ore_list_get(b2, i);
        int8_t __tmp_960 = ore_list_get_kind(b2, i);
        ore_map_set(s, __tmp_959, (int64_t)(((int8_t)1)));
        cont_115: ;
    }
    brk_114: ;
    void* __tmp_961 = ore_list_new();
    ore_list_push(__tmp_961, (int64_t)(intptr_t)(ore_str_new("sqrt", 4)));
    ore_list_push(__tmp_961, (int64_t)(intptr_t)(ore_str_new("sin", 3)));
    ore_list_push(__tmp_961, (int64_t)(intptr_t)(ore_str_new("cos", 3)));
    ore_list_push(__tmp_961, (int64_t)(intptr_t)(ore_str_new("tan", 3)));
    ore_list_push(__tmp_961, (int64_t)(intptr_t)(ore_str_new("log", 3)));
    ore_list_push(__tmp_961, (int64_t)(intptr_t)(ore_str_new("log10", 5)));
    ore_list_push(__tmp_961, (int64_t)(intptr_t)(ore_str_new("exp", 3)));
    ore_list_push(__tmp_961, (int64_t)(intptr_t)(ore_str_new("pow", 3)));
    void* b3 = __tmp_961;
    for (int64_t i = 0LL; i < ore_list_len(b3); i++) {
        int64_t __tmp_962 = ore_list_get(b3, i);
        int8_t __tmp_963 = ore_list_get_kind(b3, i);
        ore_map_set(s, __tmp_962, (int64_t)(((int8_t)1)));
        cont_117: ;
    }
    brk_116: ;
    void* __tmp_964 = ore_list_new();
    ore_list_push(__tmp_964, (int64_t)(intptr_t)(ore_str_new("atan2", 5)));
    ore_list_push(__tmp_964, (int64_t)(intptr_t)(ore_str_new("pi", 2)));
    ore_list_push(__tmp_964, (int64_t)(intptr_t)(ore_str_new("euler", 5)));
    ore_list_push(__tmp_964, (int64_t)(intptr_t)(ore_str_new("e", 1)));
    ore_list_push(__tmp_964, (int64_t)(intptr_t)(ore_str_new("floor", 5)));
    ore_list_push(__tmp_964, (int64_t)(intptr_t)(ore_str_new("ceil", 4)));
    ore_list_push(__tmp_964, (int64_t)(intptr_t)(ore_str_new("round", 5)));
    ore_list_push(__tmp_964, (int64_t)(intptr_t)(ore_str_new("abs", 3)));
    void* b4 = __tmp_964;
    for (int64_t i = 0LL; i < ore_list_len(b4); i++) {
        int64_t __tmp_965 = ore_list_get(b4, i);
        int8_t __tmp_966 = ore_list_get_kind(b4, i);
        ore_map_set(s, __tmp_965, (int64_t)(((int8_t)1)));
        cont_119: ;
    }
    brk_118: ;
    void* __tmp_967 = ore_list_new();
    ore_list_push(__tmp_967, (int64_t)(intptr_t)(ore_str_new("rand_int", 8)));
    ore_list_push(__tmp_967, (int64_t)(intptr_t)(ore_str_new("time_now", 8)));
    ore_list_push(__tmp_967, (int64_t)(intptr_t)(ore_str_new("time_ms", 7)));
    ore_list_push(__tmp_967, (int64_t)(intptr_t)(ore_str_new("sleep", 5)));
    ore_list_push(__tmp_967, (int64_t)(intptr_t)(ore_str_new("spawn", 5)));
    ore_list_push(__tmp_967, (int64_t)(intptr_t)(ore_str_new("channel", 7)));
    void* b5 = __tmp_967;
    for (int64_t i = 0LL; i < ore_list_len(b5); i++) {
        int64_t __tmp_968 = ore_list_get(b5, i);
        int8_t __tmp_969 = ore_list_get_kind(b5, i);
        ore_map_set(s, __tmp_968, (int64_t)(((int8_t)1)));
        cont_121: ;
    }
    brk_120: ;
    void* __tmp_970 = ore_list_new();
    ore_list_push(__tmp_970, (int64_t)(intptr_t)(ore_str_new("file_read", 9)));
    ore_list_push(__tmp_970, (int64_t)(intptr_t)(ore_str_new("file_write", 10)));
    ore_list_push(__tmp_970, (int64_t)(intptr_t)(ore_str_new("file_exists", 11)));
    ore_list_push(__tmp_970, (int64_t)(intptr_t)(ore_str_new("exec", 4)));
    void* b6 = __tmp_970;
    for (int64_t i = 0LL; i < ore_list_len(b6); i++) {
        int64_t __tmp_971 = ore_list_get(b6, i);
        int8_t __tmp_972 = ore_list_get_kind(b6, i);
        ore_map_set(s, __tmp_971, (int64_t)(((int8_t)1)));
        cont_123: ;
    }
    brk_122: ;
    void* __tmp_973 = ore_list_new();
    ore_list_push(__tmp_973, (int64_t)(intptr_t)(ore_str_new("env_get", 7)));
    ore_list_push(__tmp_973, (int64_t)(intptr_t)(ore_str_new("env_set", 7)));
    ore_list_push(__tmp_973, (int64_t)(intptr_t)(ore_str_new("json_parse", 10)));
    ore_list_push(__tmp_973, (int64_t)(intptr_t)(ore_str_new("json_stringify", 14)));
    void* b7 = __tmp_973;
    for (int64_t i = 0LL; i < ore_list_len(b7); i++) {
        int64_t __tmp_974 = ore_list_get(b7, i);
        int8_t __tmp_975 = ore_list_get_kind(b7, i);
        ore_map_set(s, __tmp_974, (int64_t)(((int8_t)1)));
        cont_125: ;
    }
    brk_124: ;
    void* __tmp_976 = ore_list_new();
    ore_list_push(__tmp_976, (int64_t)(intptr_t)(ore_str_new("ord", 3)));
    ore_list_push(__tmp_976, (int64_t)(intptr_t)(ore_str_new("chr", 3)));
    ore_list_push(__tmp_976, (int64_t)(intptr_t)(ore_str_new("Some", 4)));
    ore_list_push(__tmp_976, (int64_t)(intptr_t)(ore_str_new("None", 4)));
    ore_list_push(__tmp_976, (int64_t)(intptr_t)(ore_str_new("Ok", 2)));
    ore_list_push(__tmp_976, (int64_t)(intptr_t)(ore_str_new("Err", 3)));
    void* b8 = __tmp_976;
    for (int64_t i = 0LL; i < ore_list_len(b8); i++) {
        int64_t __tmp_977 = ore_list_get(b8, i);
        int8_t __tmp_978 = ore_list_get_kind(b8, i);
        ore_map_set(s, __tmp_977, (int64_t)(((int8_t)1)));
        cont_127: ;
    }
    brk_126: ;
    void* __tmp_979 = ore_list_new();
    ore_list_push(__tmp_979, (int64_t)(intptr_t)(ore_str_new("assert", 6)));
    ore_list_push(__tmp_979, (int64_t)(intptr_t)(ore_str_new("assert_eq", 9)));
    ore_list_push(__tmp_979, (int64_t)(intptr_t)(ore_str_new("assert_ne", 9)));
    void* b9 = __tmp_979;
    for (int64_t i = 0LL; i < ore_list_len(b9); i++) {
        int64_t __tmp_980 = ore_list_get(b9, i);
        int8_t __tmp_981 = ore_list_get_kind(b9, i);
        ore_map_set(s, __tmp_980, (int64_t)(((int8_t)1)));
        cont_129: ;
    }
    brk_128: ;
    return s;
}

int64_t scope_push(void* scopes) {
    void* __tmp_982 = ore_map_new();
    ore_list_push(scopes, (int64_t)(intptr_t)(__tmp_982));
}

int64_t scope_pop(void* scopes) {
    return ore_list_pop(scopes);
}

int64_t scope_define(void* scopes, void* name, void* type_name) {
    int64_t __tmp_983 = ore_list_get(scopes, (96070743712176 - 96070743712416));
    int8_t __tmp_984 = ore_list_get_kind(scopes, (96070743712176 - 96070743712416));
    int64_t top = __tmp_983;
    return top.set(name, type_name);
}

void* scope_lookup(void* scopes, void* name) {
    int64_t i = (96070743730960 - 96070743731200);
    while (1) {
        int64_t __tmp_985 = 0;
        if ((96070664500880 < 96070743736064)) {
            return ore_str_new("", 0);
        } else {
        }
        int64_t __tmp_986 = ore_list_get(scopes, i);
        int8_t __tmp_987 = ore_list_get_kind(scopes, i);
        int64_t scope = __tmp_986;
        int64_t __tmp_988 = 0;
        if (scope.contains(name)) {
            return scope.get(name);
        } else {
        }
        i = (96070664522480 - 96070743757552);
        cont_131: ;
    }
    brk_130: ;
    return ore_str_new("", 0);
}

void* tc_new() {
    void* __tmp_989 = ore_list_new();
    void* tc = __tmp_989;
    void* __tmp_990 = ore_map_new();
    ore_list_push(tc, (int64_t)(intptr_t)(__tmp_990));
    void* __tmp_991 = ore_map_new();
    ore_list_push(tc, (int64_t)(intptr_t)(__tmp_991));
    void* __tmp_992 = ore_map_new();
    ore_list_push(tc, (int64_t)(intptr_t)(__tmp_992));
    void* __tmp_993 = ore_map_new();
    ore_list_push(tc, (int64_t)(intptr_t)(__tmp_993));
    void* __tmp_994 = ore_map_new();
    ore_list_push(tc, (int64_t)(intptr_t)(__tmp_994));
    void* __tmp_995 = ore_list_new();
    ore_list_push(tc, (int64_t)(intptr_t)(__tmp_995));
    void* __tmp_996 = ore_list_new();
    void* scopes = __tmp_996;
    void* __tmp_997 = ore_map_new();
    ore_list_push(scopes, (int64_t)(intptr_t)(__tmp_997));
    ore_list_push(tc, (int64_t)(intptr_t)(scopes));
    ore_list_push(tc, (int64_t)(intptr_t)(make_method_set()));
    ore_list_push(tc, (int64_t)(intptr_t)(make_builtin_set()));
    return tc;
}

void* tc_fns(void* tc) {
    int64_t __tmp_998 = ore_list_get(tc, 0LL);
    int8_t __tmp_999 = ore_list_get_kind(tc, 0LL);
    return __tmp_998;
}

void* tc_fn_req(void* tc) {
    int64_t __tmp_1000 = ore_list_get(tc, 1LL);
    int8_t __tmp_1001 = ore_list_get_kind(tc, 1LL);
    return __tmp_1000;
}

void* tc_records(void* tc) {
    int64_t __tmp_1002 = ore_list_get(tc, 2LL);
    int8_t __tmp_1003 = ore_list_get_kind(tc, 2LL);
    return __tmp_1002;
}

void* tc_enums(void* tc) {
    int64_t __tmp_1004 = ore_list_get(tc, 3LL);
    int8_t __tmp_1005 = ore_list_get_kind(tc, 3LL);
    return __tmp_1004;
}

void* tc_variant_to_enum(void* tc) {
    int64_t __tmp_1006 = ore_list_get(tc, 4LL);
    int8_t __tmp_1007 = ore_list_get_kind(tc, 4LL);
    return __tmp_1006;
}

void* tc_errors(void* tc) {
    int64_t __tmp_1008 = ore_list_get(tc, 5LL);
    int8_t __tmp_1009 = ore_list_get_kind(tc, 5LL);
    return __tmp_1008;
}

void* tc_scopes(void* tc) {
    int64_t __tmp_1010 = ore_list_get(tc, 6LL);
    int8_t __tmp_1011 = ore_list_get_kind(tc, 6LL);
    return __tmp_1010;
}

void* tc_methods(void* tc) {
    int64_t __tmp_1012 = ore_list_get(tc, 7LL);
    int8_t __tmp_1013 = ore_list_get_kind(tc, 7LL);
    return __tmp_1012;
}

void* tc_builtins(void* tc) {
    int64_t __tmp_1014 = ore_list_get(tc, 8LL);
    int8_t __tmp_1015 = ore_list_get_kind(tc, 8LL);
    return __tmp_1014;
}

int64_t tc_add_error(void* tc, void* msg) {
    void* errs = tc_errors(tc);
    ore_list_push(errs, (int64_t)(intptr_t)(msg));
}

int8_t is_line_end(struct ore_enum_Token t) {
    int8_t __tmp_1016 = 0;
    if (t.tag == 72) {
        __tmp_1016 = (int8_t)(((int8_t)1));
    }
    else if (t.tag == 73) {
        __tmp_1016 = (int8_t)(((int8_t)1));
    }
    else if (t.tag == 49) {
        __tmp_1016 = (int8_t)(((int8_t)1));
    }
    else if (t.tag == 75) {
        __tmp_1016 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_1016 = (int8_t)(((int8_t)0));
    }
    return __tmp_1016;
}

int8_t is_colon(struct ore_enum_Token t) {
    int8_t __tmp_1017 = 0;
    if (t.tag == 52) {
        __tmp_1017 = (int8_t)(((int8_t)1));
    }
    else {
        __tmp_1017 = (int8_t)(((int8_t)0));
    }
    return __tmp_1017;
}

void* token_ident_name(struct ore_enum_Token t) {
    int64_t __tmp_1018 = 0;
    if (t.tag == 2) {
        int64_t n = t.data[0];
        __tmp_1018 = (int64_t)(n);
    }
    else {
        __tmp_1018 = (int64_t)(ore_str_new("", 0));
    }
    return __tmp_1018;
}

int64_t count_fn_params(void* tokens, int64_t start, int64_t len) {
    int64_t pcount = 0LL;
    int64_t j = start;
    while (1) {
        int64_t __tmp_1019 = 0;
        if ((96070664769040 >= 96070664770704)) {
            goto brk_132;
        } else {
        }
        struct ore_enum_Token pt = get_token(tokens, j);
        int64_t __tmp_1020 = 0;
        if (is_line_end(pt)) {
            goto brk_132;
        } else {
        }
        void* pname = token_ident_name(pt);
        int64_t __tmp_1021 = 0;
        if ((!ore_str_eq(pname, ore_str_new("", 0)))) {
            int64_t __tmp_1022 = 0;
            if ((96070744025088 < 96070664796576)) {
                struct ore_enum_Token ct = get_token(tokens, (96070664803312 + 96070744031424));
                int64_t __tmp_1023 = 0;
                if (is_colon(ct)) {
                    pcount = (96070664811792 + 96070744044368);
                } else {
                }
                __tmp_1022 = (int64_t)(__tmp_1023);
            } else {
            }
            __tmp_1021 = (int64_t)(__tmp_1022);
        } else {
        }
        j = (96070664815680 + 96070744052352);
        cont_133: ;
    }
    brk_132: ;
    return pcount;
}

int64_t collect_fn_defs(void* tc, void* tokens) {
    int64_t i = 0LL;
    int64_t len = ore_list_len(tokens);
    while (1) {
        int64_t __tmp_1024 = 0;
        if ((96070664840432 >= 96070664842096)) {
            goto brk_134;
        } else {
        }
        struct ore_enum_Token tok = get_token(tokens, i);
        int64_t __tmp_1025 = 0;
        if (tok.tag == 7) {
            int64_t __tmp_1026 = 0;
            if ((96070744093872 < 96070664859632)) {
                void* fname = token_ident_name(get_token(tokens, (96070664868448 + 96070744100224)));
                int64_t __tmp_1027 = 0;
                if ((!ore_str_eq(fname, ore_str_new("", 0)))) {
                    int64_t pcount = count_fn_params(tokens, (96070664879552 + 96070744115696), len);
                    void* fns = tc_fns(tc);
                    ore_map_set(fns, fname, (int64_t)(int_to_str(pcount)));
                    void* freq = tc_fn_req(tc);
                    ore_map_set(freq, fname, (int64_t)(int_to_str(pcount)));
                } else {
                }
                __tmp_1026 = (int64_t)(__tmp_1027);
            } else {
            }
            __tmp_1025 = (int64_t)(__tmp_1026);
        }
        else {
            __tmp_1025 = (int64_t)(0LL);
        }
        i = (96070664915696 + 96070744161776);
        cont_135: ;
    }
    brk_134: ;
}

int64_t collect_type_defs(void* tc, void* tokens) {
    int64_t i = 0LL;
    int64_t len = ore_list_len(tokens);
    while (1) {
        int64_t __tmp_1028 = 0;
        if ((96070664939312 >= 96070664940976)) {
            goto brk_136;
        } else {
        }
        struct ore_enum_Token tok = get_token(tokens, i);
        int64_t __tmp_1029 = 0;
        if (tok.tag == 20) {
            int64_t __tmp_1030 = 0;
            if ((96070744202032 < 96070664958576)) {
                void* tname = token_ident_name(get_token(tokens, (96070664967392 + 96070744208384)));
                int64_t __tmp_1031 = 0;
                if ((!ore_str_eq(tname, ore_str_new("", 0)))) {
                    int8_t is_enum = ((int8_t)0);
                    int64_t j = (96070664981152 + 96070744224640);
                    while (1) {
                        int64_t __tmp_1032 = 0;
                        if ((96070664985472 >= 96070664987136)) {
                            goto brk_138;
                        } else {
                        }
                        struct ore_enum_Token nt = get_token(tokens, j);
                        int64_t __tmp_1033 = 0;
                        if (nt.tag == 67) {
                            goto brk_138;
                        }
                        else if (nt.tag == 72) {
                            int64_t __tmp_1034 = 0;
                            if ((96070744268096 < 96070665010576)) {
                                int64_t __tmp_1035 = 0;
                                if (get_token(tokens, (96070665017360 + 96070744275376)).tag == 73) {
                                    is_enum = ((int8_t)1);
                                }
                                else {
                                    __tmp_1035 = (int64_t)(0LL);
                                }
                                __tmp_1034 = (int64_t)(__tmp_1035);
                            } else {
                            }
                            goto brk_138;
                            __tmp_1033 = (int64_t)(__tmp_1034);
                        }
                        else {
                            goto brk_138;
                        }
                        j = (96070665036240 + 96070744306576);
                        cont_139: ;
                    }
                    brk_138: ;
                    int64_t __tmp_1036 = 0;
                    if (is_enum) {
                        void* enums = tc_enums(tc);
                        ore_map_set(enums, tname, (int64_t)(ore_str_new("1", 1)));
                        __tmp_1036 = (int64_t)(collect_enum_variants(tc, tokens, (96070665057904 + 96070744331344), len, tname));
                    } else {
                        void* recs = tc_records(tc);
                        ore_map_set(recs, tname, (int64_t)(ore_str_new("1", 1)));
                    }
                    __tmp_1031 = (int64_t)(__tmp_1036);
                } else {
                }
                __tmp_1030 = (int64_t)(__tmp_1031);
            } else {
            }
            __tmp_1029 = (int64_t)(__tmp_1030);
        }
        else {
            __tmp_1029 = (int64_t)(0LL);
        }
        i = (96070665080048 + 96070744360528);
        cont_137: ;
    }
    brk_136: ;
}

int64_t collect_enum_variants(void* tc, void* tokens, int64_t start, int64_t len, void* tname) {
    int64_t k = start;
    while (1) {
        int64_t __tmp_1037 = 0;
        if ((96070665108128 >= 96070665109792)) {
            goto brk_140;
        } else {
        }
        struct ore_enum_Token vt = get_token(tokens, k);
        int64_t __tmp_1038 = 0;
        if (vt.tag == 74) {
            goto brk_140;
        }
        else {
            __tmp_1038 = (int64_t)(0LL);
        }
        void* vname = token_ident_name(vt);
        int64_t __tmp_1039 = 0;
        if ((!ore_str_eq(vname, ore_str_new("", 0)))) {
            int64_t first_ch = ore_ord(ore_str_char_at(vname, 0LL));
            int64_t __tmp_1040 = 0;
            if ((96070744423568 && 96070744426096)) {
                void* v2e = tc_variant_to_enum(tc);
                ore_map_set(v2e, vname, (int64_t)(tname));
            } else {
            }
            __tmp_1039 = (int64_t)(__tmp_1040);
        } else {
        }
        k = (96070665163824 + 96070744443792);
        cont_141: ;
    }
    brk_140: ;
}

int8_t is_known_name(void* tc, void* name) {
    int64_t __tmp_1041 = 0;
    if (ore_map_contains(tc_fns(tc), name)) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1042 = 0;
    if (ore_map_contains(tc_builtins(tc), name)) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1043 = 0;
    if (ore_map_contains(tc_records(tc), name)) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1044 = 0;
    if (ore_map_contains(tc_enums(tc), name)) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1045 = 0;
    if (ore_map_contains(tc_variant_to_enum(tc), name)) {
        return ((int8_t)1);
    } else {
    }
    return ((int8_t)0);
}

int64_t check_call(void* tc, int64_t func_id, void* call_args, void* exprs, void* stmts) {
    struct ore_enum_Expr func_e = get_expr(exprs, func_id);
    for (int64_t j = 0LL; j < ore_list_len(call_args); j++) {
        int64_t __tmp_1046 = ore_list_get(call_args, j);
        int8_t __tmp_1047 = ore_list_get_kind(call_args, j);
        cont_143: ;
    }
    brk_142: ;
    return check_call_target(tc, func_e, call_args);
}

int64_t check_call_target(void* tc, struct ore_enum_Expr func_e, void* call_args) {
    int64_t __tmp_1048 = 0;
    if (func_e.tag == 4) {
        int64_t fname = func_e.data[0];
        int64_t __tmp_1049 = 0;
        if ((!(is_known_name(tc, fname)))) {
            void* scopes = tc_scopes(tc);
            void* found = scope_lookup(scopes, fname);
            int64_t __tmp_1050 = 0;
            if (ore_str_eq(found, ore_str_new("", 0))) {
                __tmp_1050 = (int64_t)(tc_add_error(tc, ore_str_concat(ore_str_new("undefined function: ", 20), (void*)(intptr_t)(96070665328448))));
            } else {
            }
            __tmp_1049 = (int64_t)(__tmp_1050);
        } else {
        }
        __tmp_1048 = (int64_t)(check_call_arity(tc, fname, ore_list_len(call_args)));
    }
    else {
        __tmp_1048 = (int64_t)(0LL);
    }
    return __tmp_1048;
}

int64_t check_call_arity(void* tc, void* fname, int64_t nargs) {
    void* fns = tc_fns(tc);
    int64_t __tmp_1051 = 0;
    if ((!(ore_map_contains(fns, fname)))) {
        return;
    } else {
    }
    int64_t max_arity = ore_map_get(fns, fname).to_int();
    void* freq = tc_fn_req(tc);
    int64_t min_arity = max_arity;
    int64_t __tmp_1052 = 0;
    if (ore_map_contains(freq, fname)) {
        min_arity = ore_map_get(freq, fname).to_int();
    } else {
    }
    int64_t __tmp_1053 = 0;
    if ((96070665399088 < 96070665400912)) {
        __tmp_1053 = (int64_t)(tc_add_error(tc, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("function '", 10), fname), ore_str_new("' expects at least ", 19)), int_to_str(min_arity)), ore_str_new(" arguments, got ", 16)), int_to_str(nargs))));
    } else {
    }
    int64_t __tmp_1054 = 0;
    if ((96070665421600 > 96070665423424)) {
        __tmp_1054 = (int64_t)(tc_add_error(tc, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("function '", 10), fname), ore_str_new("' expects at most ", 18)), int_to_str(max_arity)), ore_str_new(" arguments, got ", 16)), int_to_str(nargs))));
    } else {
    }
    return __tmp_1054;
}

int64_t check_ident(void* tc, void* name) {
    void* scopes = tc_scopes(tc);
    void* found = scope_lookup(scopes, name);
    int64_t __tmp_1055 = 0;
    if (ore_str_eq(found, ore_str_new("", 0))) {
        int64_t __tmp_1056 = 0;
        if ((!(is_known_name(tc, name)))) {
            __tmp_1056 = (int64_t)(tc_add_error(tc, ore_str_concat(ore_str_new("undefined variable: ", 20), name)));
        } else {
        }
        __tmp_1055 = (int64_t)(__tmp_1056);
    } else {
    }
    return __tmp_1055;
}

int64_t check_expr(void* tc, int64_t expr_id, void* exprs, void* stmts) {
    int64_t __tmp_1057 = 0;
    if ((96070665499888 < 96070744790800)) {
        return;
    } else {
    }
    struct ore_enum_Expr e = get_expr(exprs, expr_id);
    int64_t __tmp_1058 = 0;
    if (e.tag == 4) {
        int64_t name = e.data[0];
        __tmp_1058 = (int64_t)(check_ident(tc, name));
    }
    else if (e.tag == 8) {
        int64_t func_id = e.data[0];
        int64_t args = e.data[1];
        __tmp_1058 = (int64_t)(check_call(tc, func_id, args, exprs, stmts));
    }
    else if (e.tag == 5) {
        int64_t op = e.data[0];
        int64_t left = e.data[1];
        int64_t right = e.data[2];
        __tmp_1058 = (int64_t)(check_expr(tc, right, exprs, stmts));
    }
    else if (e.tag == 6) {
        int64_t inner = e.data[0];
        __tmp_1058 = (int64_t)(check_expr(tc, inner, exprs, stmts));
    }
    else if (e.tag == 7) {
        int64_t inner = e.data[0];
        __tmp_1058 = (int64_t)(check_expr(tc, inner, exprs, stmts));
    }
    else if (e.tag == 10) {
        int64_t cond = e.data[0];
        int64_t then_block = e.data[1];
        int64_t else_block = e.data[2];
        __tmp_1058 = (int64_t)(check_expr(tc, cond, exprs, stmts));
    }
    else if (e.tag == 18) {
        int64_t object = e.data[0];
        int64_t method = e.data[1];
        int64_t args = e.data[2];
        for (int64_t j = 0LL; j < args.len(); j++) {
            cont_145: ;
        }
        brk_144: ;
        __tmp_1058 = (int64_t)(check_expr(tc, object, exprs, stmts));
    }
    else if (e.tag == 17) {
        int64_t object = e.data[0];
        int64_t field = e.data[1];
        __tmp_1058 = (int64_t)(check_expr(tc, object, exprs, stmts));
    }
    else if (e.tag == 22) {
        int64_t object = e.data[0];
        int64_t index = e.data[1];
        __tmp_1058 = (int64_t)(check_expr(tc, index, exprs, stmts));
    }
    else if (e.tag == 15) {
        int64_t params = e.data[0];
        int64_t body = e.data[1];
        void* scopes = tc_scopes(tc);
        for (int64_t j = 0LL; j < params.len(); j++) {
            cont_147: ;
        }
        brk_146: ;
        __tmp_1058 = (int64_t)(scope_pop(scopes));
    }
    else if (e.tag == 19) {
        int64_t elements = e.data[0];
        for (int64_t j = 0LL; j < elements.len(); j++) {
            cont_149: ;
        }
        brk_148: ;
    }
    else if (e.tag == 21) {
        int64_t entries = e.data[0];
        for (int64_t j = 0LL; j < entries.len(); j++) {
            cont_151: ;
        }
        brk_150: ;
    }
    else if (e.tag == 20) {
        int64_t expr = e.data[0];
        int64_t var_name = e.data[1];
        int64_t iterable = e.data[2];
        int64_t cond = e.data[3];
        void* scopes = tc_scopes(tc);
        int64_t __tmp_1059 = 0;
        if ((96070665797392 >= 96070745130368)) {
            __tmp_1059 = (int64_t)(check_expr(tc, cond, exprs, stmts));
        } else {
        }
        __tmp_1058 = (int64_t)(scope_pop(scopes));
    }
    else if (e.tag == 25) {
        int64_t inner = e.data[0];
        __tmp_1058 = (int64_t)(check_expr(tc, inner, exprs, stmts));
    }
    else if (e.tag == 26) {
        int64_t inner = e.data[0];
        __tmp_1058 = (int64_t)(check_expr(tc, inner, exprs, stmts));
    }
    else if (e.tag == 27) {
        int64_t inner = e.data[0];
        __tmp_1058 = (int64_t)(check_expr(tc, inner, exprs, stmts));
    }
    else if (e.tag == 28) {
        int64_t inner = e.data[0];
        __tmp_1058 = (int64_t)(check_expr(tc, inner, exprs, stmts));
    }
    else if (e.tag == 13) {
        int64_t parts = e.data[0];
        __tmp_1058 = (int64_t)(check_string_interp(tc, parts, exprs, stmts));
    }
    else if (e.tag == 12) {
        int64_t subject = e.data[0];
        int64_t arms = e.data[1];
        __tmp_1058 = (int64_t)(check_match_arms(tc, arms, exprs, stmts));
    }
    else if (e.tag == 16) {
        int64_t type_name = e.data[0];
        int64_t fields = e.data[1];
        void* recs = tc_records(tc);
        void* v2e = tc_variant_to_enum(tc);
        int64_t __tmp_1060 = 0;
        if ((96070745275792 && 96070745280000)) {
            __tmp_1060 = (int64_t)(tc_add_error(tc, ore_str_concat(ore_str_new("undefined type: ", 16), (void*)(intptr_t)(96070665933456))));
        } else {
        }
        __tmp_1058 = (int64_t)(__tmp_1060);
    }
    else if (e.tag == 14) {
        int64_t block = e.data[0];
        __tmp_1058 = (int64_t)(0LL);
    }
    else if (e.tag == 11) {
        int64_t cond = e.data[0];
        int64_t then_expr = e.data[1];
        int64_t else_expr = e.data[2];
        __tmp_1058 = (int64_t)(check_expr(tc, else_expr, exprs, stmts));
    }
    else if (e.tag == 9) {
        int64_t inner = e.data[0];
        __tmp_1058 = (int64_t)(check_expr(tc, inner, exprs, stmts));
    }
    else {
        __tmp_1058 = (int64_t)(0LL);
    }
    return __tmp_1058;
}

int64_t check_string_interp(void* tc, void* parts, void* exprs, void* stmts) {
    for (int64_t j = 0LL; j < ore_list_len(parts); j++) {
        struct ore_enum_StringPart p = get_string_part(parts, j);
        int64_t __tmp_1061 = 0;
        if (p.tag == 1) {
            int64_t expr_id = p.data[0];
            __tmp_1061 = (int64_t)(check_expr(tc, expr_id, exprs, stmts));
        }
        else {
            __tmp_1061 = (int64_t)(0LL);
        }
        cont_153: ;
    }
    brk_152: ;
}

int64_t check_match_arms(void* tc, void* arms, void* exprs, void* stmts) {
    for (int64_t j = 0LL; j < ore_list_len(arms); j++) {
        struct ore_rec_MatchArm arm = get_match_arm_typed(arms, j);
        void* scopes = tc_scopes(tc);
        int64_t __tmp_1062 = 0;
        if ((96070745437552 >= 96070745437744)) {
            __tmp_1062 = (int64_t)(check_expr(tc, arm.guard, exprs, stmts));
        } else {
        }
        cont_155: ;
    }
    brk_154: ;
}

int64_t add_pattern_bindings(void* tc, struct ore_enum_Pattern pat) {
    void* scopes = tc_scopes(tc);
    int64_t __tmp_1063 = 0;
    if (pat.tag == 0) {
        int64_t name = pat.data[0];
        int64_t bindings = pat.data[1];
        for (int64_t j = 0LL; j < bindings.len(); j++) {
            cont_157: ;
        }
        brk_156: ;
    }
    else {
        __tmp_1063 = (int64_t)(0LL);
    }
    return __tmp_1063;
}

int64_t check_stmt(void* tc, int64_t stmt_id, void* exprs, void* stmts) {
    int64_t __tmp_1064 = 0;
    if ((96070666192944 < 96070745513792)) {
        return;
    } else {
    }
    struct ore_enum_Stmt st = get_stmt(stmts, stmt_id);
    int64_t __tmp_1065 = 0;
    if (st.tag == 0) {
        int64_t name = st.data[0];
        int64_t mutable = st.data[1];
        int64_t value = st.data[2];
        void* scopes = tc_scopes(tc);
        __tmp_1065 = (int64_t)(scope_define(scopes, name, ore_str_new("any", 3)));
    }
    else if (st.tag == 2) {
        int64_t name = st.data[0];
        int64_t value = st.data[1];
        __tmp_1065 = (int64_t)(check_expr(tc, value, exprs, stmts));
    }
    else if (st.tag == 5) {
        int64_t expr_id = st.data[0];
        __tmp_1065 = (int64_t)(check_expr(tc, expr_id, exprs, stmts));
    }
    else if (st.tag == 6) {
        int64_t value = st.data[0];
        int64_t __tmp_1066 = 0;
        if ((96070666262432 >= 96070745591808)) {
            __tmp_1066 = (int64_t)(check_expr(tc, value, exprs, stmts));
        } else {
        }
        __tmp_1065 = (int64_t)(__tmp_1066);
    }
    else if (st.tag == 7) {
        int64_t var_name = st.data[0];
        int64_t start = st.data[1];
        int64_t end = st.data[2];
        int64_t step = st.data[3];
        int64_t body = st.data[4];
        void* scopes = tc_scopes(tc);
        __tmp_1065 = (int64_t)(scope_define(scopes, var_name, ore_str_new("Int", 3)));
    }
    else if (st.tag == 9) {
        int64_t var_name = st.data[0];
        int64_t iterable = st.data[1];
        int64_t body = st.data[2];
        void* scopes = tc_scopes(tc);
        __tmp_1065 = (int64_t)(scope_define(scopes, var_name, ore_str_new("any", 3)));
    }
    else if (st.tag == 3) {
        int64_t object = st.data[0];
        int64_t index = st.data[1];
        int64_t value = st.data[2];
        __tmp_1065 = (int64_t)(check_expr(tc, value, exprs, stmts));
    }
    else if (st.tag == 4) {
        int64_t object = st.data[0];
        int64_t field = st.data[1];
        int64_t value = st.data[2];
        __tmp_1065 = (int64_t)(check_expr(tc, value, exprs, stmts));
    }
    else if (st.tag == 14) {
        int64_t expr_id = st.data[0];
        __tmp_1065 = (int64_t)(check_expr(tc, expr_id, exprs, stmts));
    }
    else if (st.tag == 1) {
        int64_t names = st.data[0];
        int64_t value = st.data[1];
        void* scopes = tc_scopes(tc);
        for (int64_t j = 0LL; j < names.len(); j++) {
            cont_159: ;
        }
        brk_158: ;
        __tmp_1065 = (int64_t)(check_expr(tc, value, exprs, stmts));
    }
    else {
        __tmp_1065 = (int64_t)(0LL);
    }
    return __tmp_1065;
}

int64_t check_block(void* tc, struct ore_rec_Block block, void* exprs, void* stmts) {
    for (int64_t i = 0LL; i < ore_list_len(block.stmts); i++) {
        struct ore_rec_SpannedStmt ss = get_sspanned(block.stmts, i);
        cont_161: ;
    }
    brk_160: ;
}

int64_t check_all_stmts(void* tc, void* exprs, void* stmts) {
    for (int64_t i = 0LL; i < ore_list_len(stmts); i++) {
        cont_163: ;
    }
    brk_162: ;
}

void* typecheck_with_scopes(void* tokens, void* exprs, void* stmts) {
    void* tc = tc_new();
    return tc_errors(tc);
}

void* typecheck(void* tokens, void* exprs, void* stmts) {
    return typecheck_with_scopes(tokens, exprs, stmts);
}

int64_t add_fn_params_from_tokens(void* tc, void* tokens) {
    void* scopes = tc_scopes(tc);
    int64_t i = 0LL;
    int64_t len = ore_list_len(tokens);
    while (1) {
        int64_t __tmp_1067 = 0;
        if ((96070666614064 >= 96070666615728)) {
            goto brk_164;
        } else {
        }
        struct ore_enum_Token tok = get_token(tokens, i);
        int64_t __tmp_1068 = 0;
        if (tok.tag == 7) {
            __tmp_1068 = (int64_t)(add_params_for_fn(scopes, tokens, (96070666635888 + 96070745953856), len));
        }
        else if (tok.tag == 15) {
            int64_t __tmp_1069 = 0;
            if ((96070745968800 < 96070666646720)) {
                void* vname = token_ident_name(get_token(tokens, (96070666655536 + 96070745975152)));
                int64_t __tmp_1070 = 0;
                if ((!ore_str_eq(vname, ore_str_new("", 0)))) {
                    __tmp_1070 = (int64_t)(scope_define(scopes, vname, ore_str_new("any", 3)));
                } else {
                }
                __tmp_1069 = (int64_t)(__tmp_1070);
            } else {
            }
            __tmp_1068 = (int64_t)(__tmp_1069);
        }
        else if (tok.tag == 49) {
            __tmp_1068 = (int64_t)(add_match_arm_bindings(scopes, tokens, i));
        }
        else {
            __tmp_1068 = (int64_t)(0LL);
        }
        i = (96070666686192 + 96070746016704);
        cont_165: ;
    }
    brk_164: ;
}

int64_t add_match_arm_bindings(void* scopes, void* tokens, int64_t arrow_pos) {
    int64_t j = (96070666719008 - 96070746029616);
    while (1) {
        int64_t __tmp_1071 = 0;
        if ((96070666722720 < 96070746034480)) {
            goto brk_166;
        } else {
        }
        struct ore_enum_Token t = get_token(tokens, j);
        void* name = token_ident_name(t);
        int64_t __tmp_1072 = 0;
        if (ore_str_eq(name, ore_str_new("", 0))) {
            goto brk_166;
        } else {
        }
        int64_t __tmp_1073 = 0;
        if (ore_str_eq(name, ore_str_new("_", 1))) {
            goto brk_166;
        } else {
        }
        int64_t first_ch = ore_ord(ore_str_char_at(name, 0LL));
        int64_t __tmp_1074 = 0;
        if ((96070746070272 && 96070746072800)) {
            int64_t k = (96070666766896 + 96070746076000);
            while (1) {
                int64_t __tmp_1075 = 0;
                if ((96070666770832 >= 96070666772688)) {
                    goto brk_168;
                } else {
                }
                void* bname = token_ident_name(get_token(tokens, k));
                int64_t __tmp_1076 = 0;
                if ((96070746098960 && 96070746101728)) {
                    __tmp_1076 = (int64_t)(scope_define(scopes, bname, ore_str_new("any", 3)));
                } else {
                }
                k = (96070666798512 + 96070746113840);
                cont_169: ;
            }
            brk_168: ;
            goto brk_166;
        } else {
        }
        j = (96070666803568 - 96070746120128);
        cont_167: ;
    }
    brk_166: ;
}

int64_t add_params_for_fn(void* scopes, void* tokens, int64_t start, int64_t len) {
    int64_t j = start;
    while (1) {
        int64_t __tmp_1077 = 0;
        if ((96070666829168 >= 96070666830832)) {
            goto brk_170;
        } else {
        }
        struct ore_enum_Token pt = get_token(tokens, j);
        int64_t __tmp_1078 = 0;
        if (is_line_end(pt)) {
            goto brk_170;
        } else {
        }
        void* pname = token_ident_name(pt);
        int64_t __tmp_1079 = 0;
        if ((!ore_str_eq(pname, ore_str_new("", 0)))) {
            int64_t __tmp_1080 = 0;
            if ((96070746169520 < 96070666856704)) {
                struct ore_enum_Token ct = get_token(tokens, (96070666863440 + 96070746175856));
                int64_t __tmp_1081 = 0;
                if (is_colon(ct)) {
                    __tmp_1081 = (int64_t)(scope_define(scopes, pname, ore_str_new("any", 3)));
                } else {
                }
                __tmp_1080 = (int64_t)(__tmp_1081);
            } else {
            }
            __tmp_1079 = (int64_t)(__tmp_1080);
        } else {
        }
        j = (96070666877840 + 96070746200160);
        cont_171: ;
    }
    brk_170: ;
}

void* cg_new() {
    void* __tmp_1082 = ore_list_new();
    void* st = __tmp_1082;
    void* __tmp_1083 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1083));
    void* __tmp_1084 = ore_list_new();
    ore_list_push(__tmp_1084, (int64_t)(0LL));
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1084));
    void* __tmp_1085 = ore_list_new();
    ore_list_push(__tmp_1085, (int64_t)(0LL));
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1085));
    void* __tmp_1086 = ore_list_new();
    ore_list_push(__tmp_1086, (int64_t)(0LL));
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1086));
    void* __tmp_1087 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1087));
    void* __tmp_1088 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1088));
    void* __tmp_1089 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1089));
    void* __tmp_1090 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1090));
    void* __tmp_1091 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1091));
    void* __tmp_1092 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1092));
    void* __tmp_1093 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1093));
    void* __tmp_1094 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1094));
    void* __tmp_1095 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1095));
    void* __tmp_1096 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1096));
    void* __tmp_1097 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1097));
    void* __tmp_1098 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1098));
    void* __tmp_1099 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1099));
    void* __tmp_1100 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1100));
    void* __tmp_1101 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1101));
    void* __tmp_1102 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1102));
    void* __tmp_1103 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1103));
    void* __tmp_1104 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1104));
    void* __tmp_1105 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1105));
    void* __tmp_1106 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1106));
    void* __tmp_1107 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1107));
    void* __tmp_1108 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1108));
    void* __tmp_1109 = ore_list_new();
    ore_list_push(st, (int64_t)(intptr_t)(__tmp_1109));
    return st;
}

void* cg_list(void* st, int64_t idx) {
    int64_t __tmp_1110 = ore_list_get(st, idx);
    int8_t __tmp_1111 = ore_list_get_kind(st, idx);
    return __tmp_1110;
}

void* str_at(void* lst, int64_t idx) {
    int64_t __tmp_1112 = ore_list_get(lst, idx);
    int8_t __tmp_1113 = ore_list_get_kind(lst, idx);
    return __tmp_1112;
}

void* cg_lines(void* st) {
    return cg_list(st, 0LL);
}

int64_t cg_indent(void* st) {
    void* p = cg_list(st, 1LL);
    int64_t __tmp_1114 = ore_list_get(p, 0LL);
    int8_t __tmp_1115 = ore_list_get_kind(p, 0LL);
    return __tmp_1114;
}

int64_t cg_set_indent(void* st, int64_t val) {
    void* p = cg_list(st, 1LL);
    ore_list_set(p, 0LL, (int64_t)(val));
}

int64_t cg_inc_indent(void* st) {
    return cg_set_indent(st, (96070746381952 + 96070746382144));
}

int64_t cg_dec_indent(void* st) {
    int64_t n = cg_indent(st);
    int64_t __tmp_1116 = 0;
    if ((96070667267712 > 96070746398656)) {
        __tmp_1116 = (int64_t)(cg_set_indent(st, (96070667273296 - 96070746403552)));
    } else {
    }
    return __tmp_1116;
}

void* cg_tmp(void* st) {
    void* p = cg_list(st, 2LL);
    int64_t __tmp_1117 = ore_list_get(p, 0LL);
    int8_t __tmp_1118 = ore_list_get_kind(p, 0LL);
    int64_t n = __tmp_1117;
    ore_list_set(p, 0LL, (int64_t)((96070667293920 + 96070746428720)));
    return ore_str_concat(ore_str_new("__tmp_", 6), ore_dynamic_to_str(n, __tmp_1118));
}

void* cg_label(void* st, void* prefix) {
    void* p = cg_list(st, 3LL);
    int64_t __tmp_1119 = ore_list_get(p, 0LL);
    int8_t __tmp_1120 = ore_list_get_kind(p, 0LL);
    int64_t n = __tmp_1119;
    ore_list_set(p, 0LL, (int64_t)((96070667319840 + 96070746456688)));
    return ore_str_concat(ore_str_concat(prefix, ore_str_new("_", 1)), ore_dynamic_to_str(n, __tmp_1120));
}

void* cg_errors(void* st) {
    return cg_list(st, 10LL);
}

int64_t cg_error(void* st, void* msg) {
    void* errs = cg_errors(st);
    ore_list_push(errs, (int64_t)(intptr_t)(msg));
}

void* indent_str(int64_t n) {
    int64_t __tmp_1121 = 0;
    if ((96070667363888 <= 96070746496160)) {
        return ore_str_new("", 0);
    } else {
    }
    int64_t __tmp_1122 = 0;
    if ((96070667367888 == 96070746501280)) {
        return ore_str_new("    ", 4);
    } else {
    }
    int64_t __tmp_1123 = 0;
    if ((96070667372208 == 96070746507248)) {
        return ore_str_new("        ", 8);
    } else {
    }
    int64_t __tmp_1124 = 0;
    if ((96070667376912 == 96070746514048)) {
        return ore_str_new("            ", 12);
    } else {
    }
    int64_t __tmp_1125 = 0;
    if ((96070667382000 == 96070746521760)) {
        return ore_str_new("                ", 16);
    } else {
    }
    void* s = ore_str_new("", 0);
    for (int64_t i = 0LL; i < n; i++) {
        s = ore_str_concat(s, ore_str_new("    ", 4));
        cont_173: ;
    }
    brk_172: ;
    return s;
}

int64_t emit(void* st, void* line) {
    void* ind = indent_str(cg_indent(st));
    ore_list_push(cg_lines(st), (int64_t)(intptr_t)(ore_str_concat(ind, line)));
}

int64_t emit_raw(void* st, void* line) {
    ore_list_push(cg_lines(st), (int64_t)(intptr_t)(line));
}

int64_t cg_set_var(void* st, void* name, void* kind) {
    void* names = cg_list(st, 4LL);
    void* kinds = cg_list(st, 5LL);
    for (int64_t i = 0LL; i < ore_list_len(names); i++) {
        int64_t __tmp_1126 = 0;
        if (ore_str_eq(str_at(names, i), name)) {
            ore_list_set(kinds, i, (int64_t)(kind));
            return;
        } else {
        }
        cont_175: ;
    }
    brk_174: ;
    ore_list_push(names, (int64_t)(intptr_t)(name));
    ore_list_push(kinds, (int64_t)(intptr_t)(kind));
}

void* cg_get_var_kind(void* st, void* name) {
    void* names = cg_list(st, 4LL);
    void* kinds = cg_list(st, 5LL);
    int64_t i = (96070746653296 - 96070746653536);
    while ((96070667526384 >= 96070746658240)) {
        int64_t __tmp_1127 = 0;
        if (ore_str_eq(str_at(names, i), name)) {
            return str_at(kinds, i);
        } else {
        }
        i = (96070667543168 - 96070746676992);
        cont_177: ;
    }
    brk_176: ;
    return ore_str_new("int", 3);
}

int8_t cg_has_var(void* st, void* name) {
    void* names = cg_list(st, 4LL);
    for (int64_t i = 0LL; i < ore_list_len(names); i++) {
        int64_t __tmp_1128 = 0;
        if (ore_str_eq(str_at(names, i), name)) {
            return ((int8_t)1);
        } else {
        }
        cont_179: ;
    }
    brk_178: ;
    return ((int8_t)0);
}

int64_t cg_set_fn(void* st, void* name, void* ret_kind) {
    void* fns = cg_list(st, 6LL);
    void* rets = cg_list(st, 7LL);
    ore_list_push(fns, (int64_t)(intptr_t)(name));
    ore_list_push(rets, (int64_t)(intptr_t)(ret_kind));
}

void* cg_get_fn_ret(void* st, void* name) {
    void* fns = cg_list(st, 6LL);
    void* rets = cg_list(st, 7LL);
    int64_t i = (96070746762192 - 96070746762432);
    while ((96070667640992 >= 96070746767136)) {
        int64_t __tmp_1129 = 0;
        if (ore_str_eq(str_at(fns, i), name)) {
            return str_at(rets, i);
        } else {
        }
        i = (96070667657680 - 96070746785840);
        cont_181: ;
    }
    brk_180: ;
    return ore_str_new("int64_t", 7);
}

int8_t cg_has_fn(void* st, void* name) {
    void* fns = cg_list(st, 6LL);
    for (int64_t i = 0LL; i < ore_list_len(fns); i++) {
        int64_t __tmp_1130 = 0;
        if (ore_str_eq(str_at(fns, i), name)) {
            return ((int8_t)1);
        } else {
        }
        cont_183: ;
    }
    brk_182: ;
    return ((int8_t)0);
}

int64_t cg_add_generic_fn(void* st, void* name, struct ore_rec_FnDef fd) {
    ore_list_push(cg_list(st, 20LL), (int64_t)(intptr_t)(name));
    ore_list_push(cg_list(st, 21LL), ({ struct ore_rec_FnDef __v2i = fd; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_FnDef)), &__v2i, sizeof(struct ore_rec_FnDef)); }));
}

void* cg_get_generic_fn(void* st, void* name) {
    void* names = cg_list(st, 20LL);
    void* defs = cg_list(st, 21LL);
    for (int64_t i = 0LL; i < ore_list_len(names); i++) {
        int64_t __tmp_1131 = 0;
        if (ore_str_eq(str_at(names, i), name)) {
            void* __tmp_1132 = ore_list_new();
            int64_t __tmp_1133 = ore_list_get(defs, i);
            int8_t __tmp_1134 = ore_list_get_kind(defs, i);
            ore_list_push(__tmp_1132, (int64_t)(__tmp_1133));
            return __tmp_1132;
        } else {
        }
        cont_185: ;
    }
    brk_184: ;
    void* __tmp_1135 = ore_list_new();
    return __tmp_1135;
}

int8_t cg_has_generic_fn(void* st, void* name) {
    void* names = cg_list(st, 20LL);
    for (int64_t i = 0LL; i < ore_list_len(names); i++) {
        int64_t __tmp_1136 = 0;
        if (ore_str_eq(str_at(names, i), name)) {
            return ((int8_t)1);
        } else {
        }
        cont_187: ;
    }
    brk_186: ;
    return ((int8_t)0);
}

int8_t cg_has_mono(void* st, void* mono_name) {
    void* cache = cg_list(st, 22LL);
    for (int64_t i = 0LL; i < ore_list_len(cache); i++) {
        int64_t __tmp_1137 = 0;
        if (ore_str_eq(str_at(cache, i), mono_name)) {
            return ((int8_t)1);
        } else {
        }
        cont_189: ;
    }
    brk_188: ;
    return ((int8_t)0);
}

int64_t cg_add_mono(void* st, void* mono_name) {
    ore_list_push(cg_list(st, 22LL), (int64_t)(intptr_t)(mono_name));
}

int64_t cg_add_record(void* st, void* name, void* fields_str, int64_t count, void* field_kinds_str) {
    ore_list_push(cg_list(st, 11LL), (int64_t)(intptr_t)(name));
    ore_list_push(cg_list(st, 12LL), (int64_t)(intptr_t)(fields_str));
    ore_list_push(cg_list(st, 13LL), (int64_t)(count));
    ore_list_push(cg_list(st, 26LL), (int64_t)(intptr_t)(field_kinds_str));
}

int8_t cg_is_record(void* st, void* name) {
    void* recs = cg_list(st, 11LL);
    for (int64_t i = 0LL; i < ore_list_len(recs); i++) {
        int64_t __tmp_1138 = 0;
        if (ore_str_eq(str_at(recs, i), name)) {
            return ((int8_t)1);
        } else {
        }
        cont_191: ;
    }
    brk_190: ;
    return ((int8_t)0);
}

void* cg_get_record_field_kind(void* st, void* rec_name, void* field) {
    void* recs = cg_list(st, 11LL);
    void* fields_list = cg_list(st, 12LL);
    void* kinds_list = cg_list(st, 26LL);
    for (int64_t i = 0LL; i < ore_list_len(recs); i++) {
        int64_t __tmp_1139 = 0;
        if (ore_str_eq(str_at(recs, i), rec_name)) {
            void* fnames = ore_str_split(str_at(fields_list, i), ore_str_new(",", 1));
            void* fkinds = ore_str_split(str_at(kinds_list, i), ore_str_new(",", 1));
            for (int64_t j = 0LL; j < ore_list_len(fnames); j++) {
                int64_t __tmp_1140 = 0;
                if (ore_str_eq(str_at(fnames, j), field)) {
                    return str_at(fkinds, j);
                } else {
                }
                cont_195: ;
            }
            brk_194: ;
        } else {
        }
        cont_193: ;
    }
    brk_192: ;
    return ore_str_new("int", 3);
}

int64_t cg_add_enum(void* st, void* name, void* variants_str, int64_t count) {
    ore_list_push(cg_list(st, 14LL), (int64_t)(intptr_t)(name));
    ore_list_push(cg_list(st, 15LL), (int64_t)(intptr_t)(variants_str));
    ore_list_push(cg_list(st, 16LL), (int64_t)(count));
}

int8_t cg_is_enum(void* st, void* name) {
    void* enums = cg_list(st, 14LL);
    for (int64_t i = 0LL; i < ore_list_len(enums); i++) {
        int64_t __tmp_1141 = 0;
        if (ore_str_eq(str_at(enums, i), name)) {
            return ((int8_t)1);
        } else {
        }
        cont_197: ;
    }
    brk_196: ;
    return ((int8_t)0);
}

int64_t cg_add_variant_map(void* st, void* variant, void* enum_name) {
    ore_list_push(cg_list(st, 17LL), (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(variant, ore_str_new(":", 1)), enum_name)));
}

void* cg_variant_enum(void* st, void* variant) {
    void* entries = cg_list(st, 17LL);
    void* prefix = ore_str_concat(variant, ore_str_new(":", 1));
    for (int64_t i = 0LL; i < ore_list_len(entries); i++) {
        void* e = str_at(entries, i);
        int64_t __tmp_1142 = 0;
        if (ore_str_starts_with(e, prefix)) {
            return ore_str_substr(e, ore_str_len(prefix), (96070747251856 - 96070747253760));
        } else {
        }
        cont_199: ;
    }
    brk_198: ;
    return ore_str_new("", 0);
}

void* kind_to_c_type(void* kind) {
    int64_t __tmp_1143 = 0;
    if (ore_str_eq(kind, ore_str_new("int", 3))) {
        return ore_str_new("int64_t", 7);
    } else {
    }
    int64_t __tmp_1144 = 0;
    if (ore_str_eq(kind, ore_str_new("float", 5))) {
        return ore_str_new("double", 6);
    } else {
    }
    int64_t __tmp_1145 = 0;
    if (ore_str_eq(kind, ore_str_new("bool", 4))) {
        return ore_str_new("int8_t", 6);
    } else {
    }
    int64_t __tmp_1146 = 0;
    if (ore_str_eq(kind, ore_str_new("str", 3))) {
        return ore_str_new("void*", 5);
    } else {
    }
    int64_t __tmp_1147 = 0;
    if (ore_str_eq(kind, ore_str_new("list", 4))) {
        return ore_str_new("void*", 5);
    } else {
    }
    int64_t __tmp_1148 = 0;
    if (ore_str_eq(kind, ore_str_new("map", 3))) {
        return ore_str_new("void*", 5);
    } else {
    }
    int64_t __tmp_1149 = 0;
    if (ore_str_eq(kind, ore_str_new("void", 4))) {
        return ore_str_new("void", 4);
    } else {
    }
    int64_t __tmp_1150 = 0;
    if (ore_str_eq(kind, ore_str_new("option", 6))) {
        return ore_str_new("OreTaggedUnion", 14);
    } else {
    }
    int64_t __tmp_1151 = 0;
    if (ore_str_eq(kind, ore_str_new("result", 6))) {
        return ore_str_new("OreTaggedUnion", 14);
    } else {
    }
    int64_t __tmp_1152 = 0;
    if (ore_str_starts_with(kind, ore_str_new("rec:", 4))) {
        void* n = ore_str_substr(kind, 4LL, (96070747339072 - 96070747339312));
        return ore_str_concat(ore_str_new("struct ore_rec_", 15), n);
    } else {
    }
    int64_t __tmp_1153 = 0;
    if (ore_str_starts_with(kind, ore_str_new("enum:", 5))) {
        void* n = ore_str_substr(kind, 5LL, (96070747359024 - 96070747359264));
        return ore_str_concat(ore_str_new("struct ore_enum_", 16), n);
    } else {
    }
    return ore_str_new("int64_t", 7);
}

void* value_to_i64_expr(void* st, void* val, void* kind) {
    int64_t __tmp_1154 = 0;
    if (ore_str_starts_with(kind, ore_str_new("enum:", 5))) {
        void* ct = kind_to_c_type(kind);
        return ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("({ ", 3), ct), ore_str_new(" __v2i = ", 9)), val), ore_str_new("; (int64_t)(intptr_t)memcpy(malloc(sizeof(", 42)), ct), ore_str_new(")), &__v2i, sizeof(", 19)), ct), ore_str_new(")); })", 6));
    } else {
    }
    int64_t __tmp_1155 = 0;
    if (ore_str_starts_with(kind, ore_str_new("rec:", 4))) {
        void* ct = kind_to_c_type(kind);
        return ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("({ ", 3), ct), ore_str_new(" __v2i = ", 9)), val), ore_str_new("; (int64_t)(intptr_t)memcpy(malloc(sizeof(", 42)), ct), ore_str_new(")), &__v2i, sizeof(", 19)), ct), ore_str_new(")); })", 6));
    } else {
    }
    int64_t __tmp_1156 = 0;
    if ((96070747481984 || 96070747485088)) {
        return ore_str_concat(ore_str_concat(ore_str_new("(int64_t)(intptr_t)(", 20), val), ore_str_new(")", 1));
    } else {
    }
    int64_t __tmp_1157 = 0;
    if (ore_str_eq(kind, ore_str_new("bool", 4))) {
        return ore_str_concat(ore_str_concat(ore_str_new("(int64_t)(", 10), val), ore_str_new(")", 1));
    } else {
    }
    int64_t __tmp_1158 = 0;
    if (ore_str_eq(kind, ore_str_new("float", 5))) {
        return ore_str_concat(ore_str_concat(ore_str_new("*(int64_t*)&(double){", 21), val), ore_str_new("}", 1));
    } else {
    }
    return ore_str_concat(ore_str_concat(ore_str_new("(int64_t)(", 10), val), ore_str_new(")", 1));
}

void* coerce_from_i64_expr(void* val, void* kind) {
    int64_t __tmp_1159 = 0;
    if (ore_str_starts_with(kind, ore_str_new("enum:", 5))) {
        void* ct = kind_to_c_type(kind);
        return ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("*(", 2), ct), ore_str_new("*)(intptr_t)(", 13)), val), ore_str_new(")", 1));
    } else {
    }
    int64_t __tmp_1160 = 0;
    if (ore_str_starts_with(kind, ore_str_new("rec:", 4))) {
        void* ct = kind_to_c_type(kind);
        return ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("*(", 2), ct), ore_str_new("*)(intptr_t)(", 13)), val), ore_str_new(")", 1));
    } else {
    }
    int64_t __tmp_1161 = 0;
    if ((96070747588624 || 96070747591728)) {
        return ore_str_concat(ore_str_concat(ore_str_new("(void*)(intptr_t)(", 18), val), ore_str_new(")", 1));
    } else {
    }
    int64_t __tmp_1162 = 0;
    if (ore_str_eq(kind, ore_str_new("bool", 4))) {
        return ore_str_concat(ore_str_concat(ore_str_new("(int8_t)(", 9), val), ore_str_new(")", 1));
    } else {
    }
    int64_t __tmp_1163 = 0;
    if (ore_str_eq(kind, ore_str_new("float", 5))) {
        return ore_str_concat(ore_str_concat(ore_str_new("*(double*)&(int64_t){", 21), val), ore_str_new("}", 1));
    } else {
    }
    return val;
}

void* type_expr_to_kind_str(void* st, struct ore_enum_TypeExpr te) {
    int64_t __tmp_1164 = 0;
    if (te.tag == 0) {
        int64_t name = te.data[0];
        int64_t __tmp_1165 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668453664), ore_str_new("Int", 3))) {
            return ore_str_new("int", 3);
        } else {
        }
        int64_t __tmp_1166 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668458464), ore_str_new("Float", 5))) {
            return ore_str_new("float", 5);
        } else {
        }
        int64_t __tmp_1167 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668463648), ore_str_new("Bool", 4))) {
            return ore_str_new("bool", 4);
        } else {
        }
        int64_t __tmp_1168 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668468640), ore_str_new("Str", 3))) {
            return ore_str_new("str", 3);
        } else {
        }
        int64_t __tmp_1169 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668473440), ore_str_new("Void", 4))) {
            return ore_str_new("void", 4);
        } else {
        }
        int64_t __tmp_1170 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668478432), ore_str_new("List", 4))) {
            return ore_str_new("list", 4);
        } else {
        }
        int64_t __tmp_1171 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668483424), ore_str_new("Map", 3))) {
            return ore_str_new("map", 3);
        } else {
        }
        int64_t __tmp_1172 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668488224), ore_str_new("Option", 6))) {
            return ore_str_new("option", 6);
        } else {
        }
        int64_t __tmp_1173 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668493600), ore_str_new("Result", 6))) {
            return ore_str_new("result", 6);
        } else {
        }
        int64_t __tmp_1174 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668498976), ore_str_new("Channel", 7))) {
            return ore_str_new("int64_t", 7);
        } else {
        }
        int64_t __tmp_1175 = 0;
        if (cg_is_record(st, name)) {
            return ore_str_concat(ore_str_new("rec:", 4), ore_int_to_str(name));
        } else {
        }
        int64_t __tmp_1176 = 0;
        if (cg_is_enum(st, name)) {
            return ore_str_concat(ore_str_new("enum:", 5), ore_int_to_str(name));
        } else {
        }
        return ore_str_new("int", 3);
        __tmp_1164 = (int64_t)(__tmp_1176);
    }
    else {
        return ore_str_new("int", 3);
    }
    return ore_str_new("int", 3);
}

void* kind_to_suffix(void* kind) {
    int64_t __tmp_1177 = 0;
    if (ore_str_eq(kind, ore_str_new("int", 3))) {
        return ore_str_new("Int", 3);
    } else {
    }
    int64_t __tmp_1178 = 0;
    if (ore_str_eq(kind, ore_str_new("float", 5))) {
        return ore_str_new("Float", 5);
    } else {
    }
    int64_t __tmp_1179 = 0;
    if (ore_str_eq(kind, ore_str_new("bool", 4))) {
        return ore_str_new("Bool", 4);
    } else {
    }
    int64_t __tmp_1180 = 0;
    if (ore_str_eq(kind, ore_str_new("str", 3))) {
        return ore_str_new("Str", 3);
    } else {
    }
    int64_t __tmp_1181 = 0;
    if (ore_str_eq(kind, ore_str_new("void", 4))) {
        return ore_str_new("Void", 4);
    } else {
    }
    int64_t __tmp_1182 = 0;
    if (ore_str_eq(kind, ore_str_new("list", 4))) {
        return ore_str_new("List", 4);
    } else {
    }
    int64_t __tmp_1183 = 0;
    if (ore_str_eq(kind, ore_str_new("map", 3))) {
        return ore_str_new("Map", 3);
    } else {
    }
    int64_t __tmp_1184 = 0;
    if (ore_str_eq(kind, ore_str_new("option", 6))) {
        return ore_str_new("Option", 6);
    } else {
    }
    int64_t __tmp_1185 = 0;
    if (ore_str_eq(kind, ore_str_new("result", 6))) {
        return ore_str_new("Result", 6);
    } else {
    }
    int64_t __tmp_1186 = 0;
    if (ore_str_starts_with(kind, ore_str_new("rec:", 4))) {
        void* n = ore_str_substr(kind, 4LL, (96070747821472 - 96070747821712));
        return ore_str_concat(ore_str_new("Rec_", 4), n);
    } else {
    }
    int64_t __tmp_1187 = 0;
    if (ore_str_starts_with(kind, ore_str_new("enum:", 5))) {
        void* n = ore_str_substr(kind, 5LL, (96070747839088 - 96070747839328));
        return ore_str_concat(ore_str_new("Enum_", 5), n);
    } else {
    }
    return ore_str_new("Int", 3);
}

void* kind_str_to_type_name(void* kind) {
    int64_t __tmp_1188 = 0;
    if (ore_str_eq(kind, ore_str_new("int", 3))) {
        return ore_str_new("Int", 3);
    } else {
    }
    int64_t __tmp_1189 = 0;
    if (ore_str_eq(kind, ore_str_new("float", 5))) {
        return ore_str_new("Float", 5);
    } else {
    }
    int64_t __tmp_1190 = 0;
    if (ore_str_eq(kind, ore_str_new("bool", 4))) {
        return ore_str_new("Bool", 4);
    } else {
    }
    int64_t __tmp_1191 = 0;
    if (ore_str_eq(kind, ore_str_new("str", 3))) {
        return ore_str_new("Str", 3);
    } else {
    }
    int64_t __tmp_1192 = 0;
    if (ore_str_eq(kind, ore_str_new("void", 4))) {
        return ore_str_new("Void", 4);
    } else {
    }
    int64_t __tmp_1193 = 0;
    if (ore_str_eq(kind, ore_str_new("list", 4))) {
        return ore_str_new("List", 4);
    } else {
    }
    int64_t __tmp_1194 = 0;
    if (ore_str_eq(kind, ore_str_new("map", 3))) {
        return ore_str_new("Map", 3);
    } else {
    }
    int64_t __tmp_1195 = 0;
    if (ore_str_eq(kind, ore_str_new("option", 6))) {
        return ore_str_new("Option", 6);
    } else {
    }
    int64_t __tmp_1196 = 0;
    if (ore_str_eq(kind, ore_str_new("result", 6))) {
        return ore_str_new("Result", 6);
    } else {
    }
    return ore_str_new("Int", 3);
}

int8_t is_c_reserved(void* name) {
    int64_t __tmp_1197 = 0;
    if (ore_str_eq(name, ore_str_new("auto", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1198 = 0;
    if (ore_str_eq(name, ore_str_new("break", 5))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1199 = 0;
    if (ore_str_eq(name, ore_str_new("case", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1200 = 0;
    if (ore_str_eq(name, ore_str_new("char", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1201 = 0;
    if (ore_str_eq(name, ore_str_new("const", 5))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1202 = 0;
    if (ore_str_eq(name, ore_str_new("continue", 8))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1203 = 0;
    if (ore_str_eq(name, ore_str_new("default", 7))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1204 = 0;
    if (ore_str_eq(name, ore_str_new("do", 2))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1205 = 0;
    if (ore_str_eq(name, ore_str_new("double", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1206 = 0;
    if (ore_str_eq(name, ore_str_new("else", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1207 = 0;
    if (ore_str_eq(name, ore_str_new("enum", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1208 = 0;
    if (ore_str_eq(name, ore_str_new("extern", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1209 = 0;
    if (ore_str_eq(name, ore_str_new("float", 5))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1210 = 0;
    if (ore_str_eq(name, ore_str_new("for", 3))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1211 = 0;
    if (ore_str_eq(name, ore_str_new("goto", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1212 = 0;
    if (ore_str_eq(name, ore_str_new("if", 2))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1213 = 0;
    if (ore_str_eq(name, ore_str_new("int", 3))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1214 = 0;
    if (ore_str_eq(name, ore_str_new("long", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1215 = 0;
    if (ore_str_eq(name, ore_str_new("register", 8))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1216 = 0;
    if (ore_str_eq(name, ore_str_new("return", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1217 = 0;
    if (ore_str_eq(name, ore_str_new("short", 5))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1218 = 0;
    if (ore_str_eq(name, ore_str_new("signed", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1219 = 0;
    if (ore_str_eq(name, ore_str_new("sizeof", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1220 = 0;
    if (ore_str_eq(name, ore_str_new("static", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1221 = 0;
    if (ore_str_eq(name, ore_str_new("struct", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1222 = 0;
    if (ore_str_eq(name, ore_str_new("switch", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1223 = 0;
    if (ore_str_eq(name, ore_str_new("typedef", 7))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1224 = 0;
    if (ore_str_eq(name, ore_str_new("union", 5))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1225 = 0;
    if (ore_str_eq(name, ore_str_new("unsigned", 8))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1226 = 0;
    if (ore_str_eq(name, ore_str_new("void", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1227 = 0;
    if (ore_str_eq(name, ore_str_new("volatile", 8))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1228 = 0;
    if (ore_str_eq(name, ore_str_new("while", 5))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1229 = 0;
    if (ore_str_eq(name, ore_str_new("malloc", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1230 = 0;
    if (ore_str_eq(name, ore_str_new("free", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1231 = 0;
    if (ore_str_eq(name, ore_str_new("printf", 6))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1232 = 0;
    if (ore_str_eq(name, ore_str_new("exit", 4))) {
        return ((int8_t)1);
    } else {
    }
    int64_t __tmp_1233 = 0;
    if (ore_str_eq(name, ore_str_new("main", 4))) {
        return ((int8_t)1);
    } else {
    }
    return ((int8_t)0);
}

void* mangle_fn(void* name) {
    int64_t __tmp_1234 = 0;
    if (ore_str_eq(name, ore_str_new("main", 4))) {
        return name;
    } else {
    }
    int64_t __tmp_1235 = 0;
    if (is_c_reserved(name)) {
        return ore_str_concat(ore_str_new("ore_fn_", 7), name);
    } else {
    }
    return name;
}

void* mangle_var(void* name) {
    int64_t __tmp_1236 = 0;
    if (is_c_reserved(name)) {
        return ore_str_concat(ore_str_new("ore_v_", 6), name);
    } else {
    }
    return name;
}

void* c_escape(void* s) {
    void* out = ore_str_new("", 0);
    void* chars = ore_str_chars(s);
    for (int64_t i = 0LL; i < ore_list_len(chars); i++) {
        int64_t __tmp_1237 = ore_list_get(chars, i);
        int8_t __tmp_1238 = ore_list_get_kind(chars, i);
        int64_t c = __tmp_1237;
        int64_t __tmp_1239 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070668922080), ore_str_new("\\", 1))) {
            out = ore_str_concat(out, ore_str_new("\\\\", 2));
        } else {
            int64_t __tmp_1240 = 0;
            if (ore_str_eq((void*)(intptr_t)(96070668929568), ore_str_new("\"", 1))) {
                out = ore_str_concat(out, ore_str_new("\\\"", 2));
            } else {
                int64_t __tmp_1241 = 0;
                if (ore_str_eq((void*)(intptr_t)(96070668937056), ore_str_new("\n", 1))) {
                    out = ore_str_concat(out, ore_str_new("\\n", 2));
                } else {
                    int64_t __tmp_1242 = 0;
                    if (ore_str_eq((void*)(intptr_t)(96070668944512), ore_str_new("\t", 1))) {
                        out = ore_str_concat(out, ore_str_new("\\t", 2));
                    } else {
                        out = ore_str_concat(out, (void*)(intptr_t)(96070668955152));
                    }
                    __tmp_1241 = (int64_t)(__tmp_1242);
                }
                __tmp_1240 = (int64_t)(__tmp_1241);
            }
            __tmp_1239 = (int64_t)(__tmp_1240);
        }
        cont_201: ;
    }
    brk_200: ;
    return out;
}

int64_t emit_runtime_decls(void* st) {
    return emit_raw(st, ore_str_new("", 0));
}

void* compile_expr(void* st, void* exprs, void* stmts, int64_t expr_id) {
    struct ore_enum_Expr e = get_expr(exprs, expr_id);
    return compile_expr_node(st, exprs, stmts, e);
}

void* compile_expr_node(void* st, void* exprs, void* stmts, struct ore_enum_Expr e) {
    int64_t __tmp_1243 = 0;
    if (e.tag == 0) {
        int64_t value = e.data[0];
        void* __tmp_1244 = ore_list_new();
        ore_list_push(__tmp_1244, (int64_t)(intptr_t)(ore_str_concat(ore_int_to_str(value), ore_str_new("LL", 2))));
        ore_list_push(__tmp_1244, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1244;
    }
    else if (e.tag == 1) {
        int64_t value = e.data[0];
        void* __tmp_1245 = ore_list_new();
        ore_list_push(__tmp_1245, (int64_t)(intptr_t)(ore_int_to_str(value)));
        ore_list_push(__tmp_1245, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1245;
    }
    else if (e.tag == 2) {
        int64_t value = e.data[0];
        int64_t __tmp_1246 = 0;
        if (value) {
            void* __tmp_1247 = ore_list_new();
            ore_list_push(__tmp_1247, (int64_t)(intptr_t)(ore_str_new("((int8_t)1)", 11)));
            ore_list_push(__tmp_1247, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
            return __tmp_1247;
        } else {
            void* __tmp_1248 = ore_list_new();
            ore_list_push(__tmp_1248, (int64_t)(intptr_t)(ore_str_new("((int8_t)0)", 11)));
            ore_list_push(__tmp_1248, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
            return __tmp_1248;
        }
        __tmp_1243 = (int64_t)(__tmp_1246);
    }
    else if (e.tag == 3) {
        int64_t value = e.data[0];
        return compile_string_lit(st, value);
    }
    else if (e.tag == 4) {
        int64_t name = e.data[0];
        return compile_ident(st, name);
    }
    else if (e.tag == 5) {
        int64_t op = e.data[0];
        int64_t left = e.data[1];
        int64_t right = e.data[2];
        return compile_binop(st, exprs, stmts, op, left, right);
    }
    else if (e.tag == 6) {
        int64_t inner = e.data[0];
        void* r = compile_expr(st, exprs, stmts, inner);
        void* __tmp_1249 = ore_list_new();
        int64_t __tmp_1250 = ore_list_get(r, 0LL);
        int8_t __tmp_1251 = ore_list_get_kind(r, 0LL);
        ore_list_push(__tmp_1249, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("(-(", 3), ore_dynamic_to_str(__tmp_1250, __tmp_1251)), ore_str_new("))", 2))));
        int64_t __tmp_1252 = ore_list_get(r, 1LL);
        int8_t __tmp_1253 = ore_list_get_kind(r, 1LL);
        ore_list_push(__tmp_1249, (int64_t)(__tmp_1252));
        return __tmp_1249;
    }
    else if (e.tag == 7) {
        int64_t inner = e.data[0];
        void* r = compile_expr(st, exprs, stmts, inner);
        void* __tmp_1254 = ore_list_new();
        int64_t __tmp_1255 = ore_list_get(r, 0LL);
        int8_t __tmp_1256 = ore_list_get_kind(r, 0LL);
        ore_list_push(__tmp_1254, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("(!(", 3), ore_dynamic_to_str(__tmp_1255, __tmp_1256)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1254, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1254;
    }
    else if (e.tag == 8) {
        int64_t func = e.data[0];
        int64_t args = e.data[1];
        return compile_call(st, exprs, stmts, func, args);
    }
    else if (e.tag == 9) {
        int64_t inner = e.data[0];
        return compile_print(st, exprs, stmts, inner);
    }
    else if (e.tag == 10) {
        int64_t cond = e.data[0];
        int64_t then_block = e.data[1];
        int64_t else_block = e.data[2];
        return compile_if_else(st, exprs, stmts, cond, then_block, else_block);
    }
    else if (e.tag == 13) {
        int64_t parts = e.data[0];
        return compile_string_interp(st, exprs, stmts, parts);
    }
    else if (e.tag == 19) {
        int64_t elements = e.data[0];
        return compile_list_lit(st, exprs, stmts, elements);
    }
    else if (e.tag == 21) {
        int64_t entries = e.data[0];
        return compile_map_lit(st, exprs, stmts, entries);
    }
    else if (e.tag == 22) {
        int64_t object = e.data[0];
        int64_t index = e.data[1];
        return compile_index(st, exprs, stmts, object, index);
    }
    else if (e.tag == 17) {
        int64_t object = e.data[0];
        int64_t field = e.data[1];
        return compile_field_access(st, exprs, stmts, object, field);
    }
    else if (e.tag == 18) {
        int64_t object = e.data[0];
        int64_t method = e.data[1];
        int64_t args = e.data[2];
        return compile_method_call(st, exprs, stmts, object, method, args);
    }
    else if (e.tag == 25) {
        int64_t inner = e.data[0];
        void* r = compile_expr(st, exprs, stmts, inner);
        void* t = cg_tmp(st);
        int64_t __tmp_1257 = ore_list_get(r, 0LL);
        int8_t __tmp_1258 = ore_list_get_kind(r, 0LL);
        void* __tmp_1259 = ore_list_new();
        ore_list_push(__tmp_1259, (int64_t)(intptr_t)(t));
        ore_list_push(__tmp_1259, (int64_t)(intptr_t)(ore_str_new("option", 6)));
        return __tmp_1259;
        __tmp_1243 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("OreTaggedUnion ", 15), t), ore_str_new("; ", 2)), t), ore_str_new(".tag = 1; ", 10)), t), ore_str_new(".value = (int64_t)(", 19)), ore_dynamic_to_str(__tmp_1257, __tmp_1258)), ore_str_new(");", 2))));
    }
    else if (e.tag == 24) {
        void* t = cg_tmp(st);
        void* __tmp_1260 = ore_list_new();
        ore_list_push(__tmp_1260, (int64_t)(intptr_t)(t));
        ore_list_push(__tmp_1260, (int64_t)(intptr_t)(ore_str_new("option", 6)));
        return __tmp_1260;
        __tmp_1243 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("OreTaggedUnion ", 15), t), ore_str_new("; ", 2)), t), ore_str_new(".tag = 0; ", 10)), t), ore_str_new(".value = 0;", 11))));
    }
    else if (e.tag == 26) {
        int64_t inner = e.data[0];
        void* r = compile_expr(st, exprs, stmts, inner);
        void* t = cg_tmp(st);
        int64_t __tmp_1261 = ore_list_get(r, 0LL);
        int8_t __tmp_1262 = ore_list_get_kind(r, 0LL);
        void* __tmp_1263 = ore_list_new();
        ore_list_push(__tmp_1263, (int64_t)(intptr_t)(t));
        ore_list_push(__tmp_1263, (int64_t)(intptr_t)(ore_str_new("result", 6)));
        return __tmp_1263;
        __tmp_1243 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("OreTaggedUnion ", 15), t), ore_str_new("; ", 2)), t), ore_str_new(".tag = 1; ", 10)), t), ore_str_new(".value = (int64_t)(", 19)), ore_dynamic_to_str(__tmp_1261, __tmp_1262)), ore_str_new(");", 2))));
    }
    else if (e.tag == 27) {
        int64_t inner = e.data[0];
        void* r = compile_expr(st, exprs, stmts, inner);
        void* t = cg_tmp(st);
        int64_t __tmp_1264 = ore_list_get(r, 0LL);
        int8_t __tmp_1265 = ore_list_get_kind(r, 0LL);
        void* __tmp_1266 = ore_list_new();
        ore_list_push(__tmp_1266, (int64_t)(intptr_t)(t));
        ore_list_push(__tmp_1266, (int64_t)(intptr_t)(ore_str_new("result", 6)));
        return __tmp_1266;
        __tmp_1243 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("OreTaggedUnion ", 15), t), ore_str_new("; ", 2)), t), ore_str_new(".tag = 0; ", 10)), t), ore_str_new(".value = (int64_t)(", 19)), ore_dynamic_to_str(__tmp_1264, __tmp_1265)), ore_str_new(");", 2))));
    }
    else if (e.tag == 23) {
        void* __tmp_1267 = ore_list_new();
        ore_list_push(__tmp_1267, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1267, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1267;
    }
    else if (e.tag == 14) {
        int64_t block = e.data[0];
        return compile_block_expr(st, exprs, stmts, block);
    }
    else if (e.tag == 12) {
        int64_t subject = e.data[0];
        int64_t arms = e.data[1];
        return compile_match(st, exprs, stmts, subject, arms);
    }
    else if (e.tag == 15) {
        int64_t params = e.data[0];
        int64_t body = e.data[1];
        return compile_lambda(st, exprs, stmts, params, body);
    }
    else if (e.tag == 16) {
        int64_t type_name = e.data[0];
        int64_t fields = e.data[1];
        return compile_record_construct(st, exprs, stmts, type_name, fields);
    }
    else if (e.tag == 32) {
        int64_t cond = e.data[0];
        int64_t message = e.data[1];
        void* r = compile_expr(st, exprs, stmts, cond);
        void* msg = compile_string_lit(st, message);
        int64_t __tmp_1268 = ore_list_get(r, 0LL);
        int8_t __tmp_1269 = ore_list_get_kind(r, 0LL);
        int64_t __tmp_1270 = ore_list_get(msg, 0LL);
        int8_t __tmp_1271 = ore_list_get_kind(msg, 0LL);
        void* __tmp_1272 = ore_list_new();
        ore_list_push(__tmp_1272, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1272, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1272;
        __tmp_1243 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_assert(", 11), ore_dynamic_to_str(__tmp_1268, __tmp_1269)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1270, __tmp_1271)), ore_str_new(", 0);", 5))));
    }
    else if (e.tag == 33) {
        int64_t left = e.data[0];
        int64_t right = e.data[1];
        int64_t message = e.data[2];
        void* l = compile_expr(st, exprs, stmts, left);
        void* r = compile_expr(st, exprs, stmts, right);
        void* msg = compile_string_lit(st, message);
        int64_t __tmp_1273 = ore_list_get(l, 1LL);
        int8_t __tmp_1274 = ore_list_get_kind(l, 1LL);
        int64_t __tmp_1275 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070750950448), ore_str_new("str", 3))) {
            int64_t __tmp_1276 = ore_list_get(l, 0LL);
            int8_t __tmp_1277 = ore_list_get_kind(l, 0LL);
            int64_t __tmp_1278 = ore_list_get(r, 0LL);
            int8_t __tmp_1279 = ore_list_get_kind(r, 0LL);
            int64_t __tmp_1280 = ore_list_get(msg, 0LL);
            int8_t __tmp_1281 = ore_list_get_kind(msg, 0LL);
            __tmp_1275 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_assert_eq_str(", 18), ore_dynamic_to_str(__tmp_1276, __tmp_1277)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1278, __tmp_1279)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1280, __tmp_1281)), ore_str_new(", 0);", 5))));
        } else {
            int64_t __tmp_1282 = ore_list_get(l, 1LL);
            int8_t __tmp_1283 = ore_list_get_kind(l, 1LL);
            int64_t __tmp_1284 = 0;
            if (ore_str_eq((void*)(intptr_t)(96070750988112), ore_str_new("float", 5))) {
                int64_t __tmp_1285 = ore_list_get(l, 0LL);
                int8_t __tmp_1286 = ore_list_get_kind(l, 0LL);
                int64_t __tmp_1287 = ore_list_get(r, 0LL);
                int8_t __tmp_1288 = ore_list_get_kind(r, 0LL);
                int64_t __tmp_1289 = ore_list_get(msg, 0LL);
                int8_t __tmp_1290 = ore_list_get_kind(msg, 0LL);
                __tmp_1284 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_assert_eq_float(", 20), ore_dynamic_to_str(__tmp_1285, __tmp_1286)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1287, __tmp_1288)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1289, __tmp_1290)), ore_str_new(", 0);", 5))));
            } else {
                int64_t __tmp_1291 = ore_list_get(l, 0LL);
                int8_t __tmp_1292 = ore_list_get_kind(l, 0LL);
                int64_t __tmp_1293 = ore_list_get(r, 0LL);
                int8_t __tmp_1294 = ore_list_get_kind(r, 0LL);
                int64_t __tmp_1295 = ore_list_get(msg, 0LL);
                int8_t __tmp_1296 = ore_list_get_kind(msg, 0LL);
                __tmp_1284 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_assert_eq_int(", 18), ore_dynamic_to_str(__tmp_1291, __tmp_1292)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1293, __tmp_1294)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1295, __tmp_1296)), ore_str_new(", 0);", 5))));
            }
            __tmp_1275 = (int64_t)(__tmp_1284);
        }
        void* __tmp_1297 = ore_list_new();
        ore_list_push(__tmp_1297, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1297, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1297;
        __tmp_1243 = (int64_t)(__tmp_1275);
    }
    else if (e.tag == 34) {
        int64_t left = e.data[0];
        int64_t right = e.data[1];
        int64_t message = e.data[2];
        void* l = compile_expr(st, exprs, stmts, left);
        void* r = compile_expr(st, exprs, stmts, right);
        void* msg = compile_string_lit(st, message);
        int64_t __tmp_1298 = ore_list_get(l, 1LL);
        int8_t __tmp_1299 = ore_list_get_kind(l, 1LL);
        int64_t __tmp_1300 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070751105488), ore_str_new("str", 3))) {
            int64_t __tmp_1301 = ore_list_get(l, 0LL);
            int8_t __tmp_1302 = ore_list_get_kind(l, 0LL);
            int64_t __tmp_1303 = ore_list_get(r, 0LL);
            int8_t __tmp_1304 = ore_list_get_kind(r, 0LL);
            int64_t __tmp_1305 = ore_list_get(msg, 0LL);
            int8_t __tmp_1306 = ore_list_get_kind(msg, 0LL);
            __tmp_1300 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_assert_ne_str(", 18), ore_dynamic_to_str(__tmp_1301, __tmp_1302)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1303, __tmp_1304)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1305, __tmp_1306)), ore_str_new(", 0);", 5))));
        } else {
            int64_t __tmp_1307 = ore_list_get(l, 0LL);
            int8_t __tmp_1308 = ore_list_get_kind(l, 0LL);
            int64_t __tmp_1309 = ore_list_get(r, 0LL);
            int8_t __tmp_1310 = ore_list_get_kind(r, 0LL);
            int64_t __tmp_1311 = ore_list_get(msg, 0LL);
            int8_t __tmp_1312 = ore_list_get_kind(msg, 0LL);
            __tmp_1300 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_assert_ne_int(", 18), ore_dynamic_to_str(__tmp_1307, __tmp_1308)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1309, __tmp_1310)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1311, __tmp_1312)), ore_str_new(", 0);", 5))));
        }
        void* __tmp_1313 = ore_list_new();
        ore_list_push(__tmp_1313, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1313, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1313;
        __tmp_1243 = (int64_t)(__tmp_1300);
    }
    else if (e.tag == 29) {
        int64_t inner = e.data[0];
        void* r = compile_expr(st, exprs, stmts, inner);
        int64_t __tmp_1314 = ore_list_get(r, 0LL);
        int8_t __tmp_1315 = ore_list_get_kind(r, 0LL);
        void* __tmp_1316 = ore_list_new();
        ore_list_push(__tmp_1316, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1316, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1316;
        __tmp_1243 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_sleep(", 10), ore_dynamic_to_str(__tmp_1314, __tmp_1315)), ore_str_new(");", 2))));
    }
    else {
        void* __tmp_1317 = ore_list_new();
        ore_list_push(__tmp_1317, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1317, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1317;
        __tmp_1243 = (int64_t)(cg_error(st, ore_str_new("unsupported expression type", 27)));
    }
    return __tmp_1243;
}

void* compile_string_lit(void* st, void* s) {
    void* esc = c_escape(s);
    void* __tmp_1318 = ore_list_new();
    ore_list_push(__tmp_1318, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_new(\"", 13), esc), ore_str_new("\", ", 3)), ore_int_to_str(ore_str_len(s))), ore_str_new(")", 1))));
    ore_list_push(__tmp_1318, (int64_t)(intptr_t)(ore_str_new("str", 3)));
    return __tmp_1318;
}

void* compile_ident(void* st, void* name) {
    void* en = cg_variant_enum(st, name);
    int64_t __tmp_1319 = 0;
    if ((!ore_str_eq(en, ore_str_new("", 0)))) {
        return compile_variant_zero_arg(st, name, en);
    } else {
    }
    int64_t __tmp_1320 = 0;
    if ((!(cg_has_var(st, name)))) {
        int64_t __tmp_1321 = 0;
        if (cg_has_fn(st, name)) {
            void* mn = mangle_fn(name);
            void* __tmp_1322 = ore_list_new();
            ore_list_push(__tmp_1322, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("(void*)&", 8), mn)));
            ore_list_push(__tmp_1322, (int64_t)(intptr_t)(ore_str_new("int", 3)));
            return __tmp_1322;
        } else {
        }
        __tmp_1320 = (int64_t)(__tmp_1321);
    } else {
    }
    void* mn = mangle_var(name);
    void* kind = cg_get_var_kind(st, name);
    void* __tmp_1323 = ore_list_new();
    ore_list_push(__tmp_1323, (int64_t)(intptr_t)(mn));
    ore_list_push(__tmp_1323, (int64_t)(intptr_t)(kind));
    return __tmp_1323;
}

void* compile_variant_zero_arg(void* st, void* variant, void* enum_name) {
    int64_t tag = find_variant_tag(st, enum_name, variant);
    void* t = cg_tmp(st);
    void* c_type = ore_str_concat(ore_str_new("struct ore_enum_", 16), enum_name);
    void* __tmp_1324 = ore_list_new();
    ore_list_push(__tmp_1324, (int64_t)(intptr_t)(t));
    ore_list_push(__tmp_1324, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("enum:", 5), enum_name)));
    return __tmp_1324;
}

int64_t find_variant_tag(void* st, void* enum_name, void* variant) {
    void* variants_list = cg_list(st, 15LL);
    void* names_list = cg_list(st, 14LL);
    for (int64_t i = 0LL; i < ore_list_len(names_list); i++) {
        int64_t __tmp_1325 = 0;
        if (ore_str_eq(str_at(names_list, i), enum_name)) {
            void* entry = str_at(variants_list, i);
            int64_t colon_pos = ore_str_index_of(entry, ore_str_new(":", 1));
            int64_t __tmp_1326 = 0;
            if ((96070671054512 >= 96070751448016)) {
                void* vs = ore_str_substr(entry, (96070671061952 + 96070751452800), (96070751457920 - 96070751458144));
                void* parts = ore_str_split(vs, ore_str_new(",", 1));
                for (int64_t j = 0LL; j < ore_list_len(parts); j++) {
                    void* pname = str_at(parts, j);
                    int64_t paren = ore_str_index_of(pname, ore_str_new("(", 1));
                    int64_t __tmp_1327 = 0;
                    if ((96070671102112 >= 96070751487728)) {
                        pname = ore_str_substr(pname, 0LL, paren);
                    } else {
                    }
                    int64_t __tmp_1328 = 0;
                    if (ore_str_eq(pname, variant)) {
                        return j;
                    } else {
                    }
                    cont_205: ;
                }
                brk_204: ;
            } else {
            }
            __tmp_1325 = (int64_t)(__tmp_1326);
        } else {
        }
        cont_203: ;
    }
    brk_202: ;
    return 0LL;
}

void* compile_binop(void* st, void* exprs, void* stmts, struct ore_enum_BinOp op, int64_t left, int64_t right) {
    int64_t __tmp_1329 = 0;
    if (op.tag == 13) {
        return compile_pipe(st, exprs, stmts, left, right);
    }
    else {
        void* l = compile_expr(st, exprs, stmts, left);
        void* r = compile_expr(st, exprs, stmts, right);
        return compile_binop_values(st, op, l, r);
    }
    return __tmp_1329;
}

void* compile_binop_values(void* st, struct ore_enum_BinOp op, void* l, void* r) {
    int64_t __tmp_1330 = ore_list_get(l, 0LL);
    int8_t __tmp_1331 = ore_list_get_kind(l, 0LL);
    int64_t lv = __tmp_1330;
    int64_t __tmp_1332 = ore_list_get(r, 0LL);
    int8_t __tmp_1333 = ore_list_get_kind(r, 0LL);
    int64_t rv = __tmp_1332;
    int64_t __tmp_1334 = ore_list_get(l, 1LL);
    int8_t __tmp_1335 = ore_list_get_kind(l, 1LL);
    int64_t lk = __tmp_1334;
    int64_t __tmp_1336 = ore_list_get(r, 1LL);
    int8_t __tmp_1337 = ore_list_get_kind(r, 1LL);
    int64_t rk = __tmp_1336;
    int8_t l_is_str = ore_str_eq((void*)(intptr_t)(96070671236336), ore_str_new("str", 3));
    int8_t r_is_str = ore_str_eq((void*)(intptr_t)(96070671240992), ore_str_new("str", 3));
    int64_t __tmp_1338 = 0;
    if ((96070671244016 || 96070671246224)) {
        int64_t sl = lv;
        int64_t sr = rv;
        int64_t __tmp_1339 = 0;
        if ((!(l_is_str))) {
            sl = ore_str_concat(ore_str_concat(ore_str_new("(void*)(intptr_t)(", 18), ore_dynamic_to_str(lv, __tmp_1331)), ore_str_new(")", 1));
        } else {
        }
        int64_t __tmp_1340 = 0;
        if ((!(r_is_str))) {
            sr = ore_str_concat(ore_str_concat(ore_str_new("(void*)(intptr_t)(", 18), ore_dynamic_to_str(rv, __tmp_1333)), ore_str_new(")", 1));
        } else {
        }
        return compile_str_binop(st, op, sl, sr);
        __tmp_1338 = (int64_t)(__tmp_1340);
    } else {
    }
    int64_t __tmp_1341 = 0;
    if (ore_str_eq((void*)(intptr_t)(96070671283504), ore_str_new("list", 4))) {
        int64_t __tmp_1342 = 0;
        if (op.tag == 0) {
            void* __tmp_1343 = ore_list_new();
            ore_list_push(__tmp_1343, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_concat(", 16), ore_dynamic_to_str(lv, __tmp_1331)), ore_str_new(", ", 2)), ore_dynamic_to_str(rv, __tmp_1333)), ore_str_new(")", 1))));
            ore_list_push(__tmp_1343, (int64_t)(intptr_t)(ore_str_new("list", 4)));
            return __tmp_1343;
        }
        else {
            void* __tmp_1344 = ore_list_new();
            ore_list_push(__tmp_1344, (int64_t)(intptr_t)(ore_str_new("0", 1)));
            ore_list_push(__tmp_1344, (int64_t)(intptr_t)(ore_str_new("int", 3)));
            return __tmp_1344;
        }
        __tmp_1341 = (int64_t)(__tmp_1342);
    } else {
    }
    int8_t is_float = (96070751700896 || 96070751704800);
    int64_t lc = lv;
    int64_t rc = rv;
    int64_t __tmp_1345 = 0;
    if (is_float) {
        int64_t __tmp_1346 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070671322128), ore_str_new("int", 3))) {
            lc = ore_str_concat(ore_str_concat(ore_str_new("(double)(", 9), ore_dynamic_to_str(lv, __tmp_1331)), ore_str_new(")", 1));
        } else {
        }
        int64_t __tmp_1347 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070671329760), ore_str_new("int", 3))) {
            rc = ore_str_concat(ore_str_concat(ore_str_new("(double)(", 9), ore_dynamic_to_str(rv, __tmp_1333)), ore_str_new(")", 1));
        } else {
        }
        __tmp_1345 = (int64_t)(__tmp_1347);
    } else {
    }
    int64_t result_kind = lk;
    int64_t __tmp_1348 = 0;
    if (is_float) {
        result_kind = ore_str_new("float", 5);
    } else {
    }
    int64_t __tmp_1349 = 0;
    if (op.tag == 5) {
        result_kind = ore_str_new("bool", 4);
    }
    else if (op.tag == 6) {
        result_kind = ore_str_new("bool", 4);
    }
    else if (op.tag == 7) {
        result_kind = ore_str_new("bool", 4);
    }
    else if (op.tag == 8) {
        result_kind = ore_str_new("bool", 4);
    }
    else if (op.tag == 9) {
        result_kind = ore_str_new("bool", 4);
    }
    else if (op.tag == 10) {
        result_kind = ore_str_new("bool", 4);
    }
    else if (op.tag == 11) {
        result_kind = ore_str_new("bool", 4);
    }
    else if (op.tag == 12) {
        result_kind = ore_str_new("bool", 4);
    }
    else {
        result_kind = result_kind;
    }
    void* c_op = binop_to_c(st, op, is_float, lc, rc);
    int64_t __tmp_1350 = 0;
    if (op.tag == 3) {
        int64_t __tmp_1351 = 0;
        if ((!(is_float))) {
            void* lt = cg_tmp(st);
            void* rt = cg_tmp(st);
            void* __tmp_1352 = ore_list_new();
            ore_list_push(__tmp_1352, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(", 1), lt), ore_str_new(" / ", 3)), rt), ore_str_new(")", 1))));
            ore_list_push(__tmp_1352, (int64_t)(intptr_t)(result_kind));
            return __tmp_1352;
            __tmp_1351 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("if (", 4), rt), ore_str_new(" == 0) ore_div_by_zero();", 25))));
        } else {
        }
        __tmp_1350 = (int64_t)(__tmp_1351);
    }
    else if (op.tag == 4) {
        int64_t __tmp_1353 = 0;
        if ((!(is_float))) {
            void* lt = cg_tmp(st);
            void* rt = cg_tmp(st);
            void* __tmp_1354 = ore_list_new();
            ore_list_push(__tmp_1354, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(", 1), lt), ore_str_new(" % ", 3)), rt), ore_str_new(")", 1))));
            ore_list_push(__tmp_1354, (int64_t)(intptr_t)(result_kind));
            return __tmp_1354;
            __tmp_1353 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("if (", 4), rt), ore_str_new(" == 0) ore_div_by_zero();", 25))));
        } else {
            void* __tmp_1355 = ore_list_new();
            ore_list_push(__tmp_1355, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("fmod(", 5), lc), ore_str_new(", ", 2)), rc), ore_str_new(")", 1))));
            ore_list_push(__tmp_1355, (int64_t)(intptr_t)(ore_str_new("float", 5)));
            return __tmp_1355;
        }
        __tmp_1350 = (int64_t)(__tmp_1353);
    }
    else {
        void* __tmp_1356 = ore_list_new();
        ore_list_push(__tmp_1356, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(", 1), lc), ore_str_new(" ", 1)), c_op), ore_str_new(" ", 1)), rc), ore_str_new(")", 1))));
        ore_list_push(__tmp_1356, (int64_t)(intptr_t)(result_kind));
        return __tmp_1356;
    }
    return __tmp_1350;
}

void* binop_to_c(void* st, struct ore_enum_BinOp op, int8_t is_float, void* lc, void* rc) {
    void* __tmp_1357 = 0;
    if (op.tag == 0) {
        __tmp_1357 = (void*)(ore_str_new("+", 1));
    }
    else if (op.tag == 1) {
        __tmp_1357 = (void*)(ore_str_new("-", 1));
    }
    else if (op.tag == 2) {
        __tmp_1357 = (void*)(ore_str_new("*", 1));
    }
    else if (op.tag == 3) {
        __tmp_1357 = (void*)(ore_str_new("/", 1));
    }
    else if (op.tag == 4) {
        __tmp_1357 = (void*)(ore_str_new("%", 1));
    }
    else if (op.tag == 5) {
        __tmp_1357 = (void*)(ore_str_new("==", 2));
    }
    else if (op.tag == 6) {
        __tmp_1357 = (void*)(ore_str_new("!=", 2));
    }
    else if (op.tag == 7) {
        __tmp_1357 = (void*)(ore_str_new("<", 1));
    }
    else if (op.tag == 8) {
        __tmp_1357 = (void*)(ore_str_new(">", 1));
    }
    else if (op.tag == 9) {
        __tmp_1357 = (void*)(ore_str_new("<=", 2));
    }
    else if (op.tag == 10) {
        __tmp_1357 = (void*)(ore_str_new(">=", 2));
    }
    else if (op.tag == 11) {
        __tmp_1357 = (void*)(ore_str_new("&&", 2));
    }
    else if (op.tag == 12) {
        __tmp_1357 = (void*)(ore_str_new("||", 2));
    }
    else {
        __tmp_1357 = (void*)(ore_str_new("+", 1));
    }
    return __tmp_1357;
}

void* compile_str_binop(void* st, struct ore_enum_BinOp op, void* lv, void* rv) {
    int64_t __tmp_1358 = 0;
    if (op.tag == 0) {
        void* __tmp_1359 = ore_list_new();
        ore_list_push(__tmp_1359, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_concat(", 15), lv), ore_str_new(", ", 2)), rv), ore_str_new(")", 1))));
        ore_list_push(__tmp_1359, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1359;
    }
    else if (op.tag == 5) {
        void* __tmp_1360 = ore_list_new();
        ore_list_push(__tmp_1360, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_eq(", 11), lv), ore_str_new(", ", 2)), rv), ore_str_new(")", 1))));
        ore_list_push(__tmp_1360, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1360;
    }
    else if (op.tag == 6) {
        void* __tmp_1361 = ore_list_new();
        ore_list_push(__tmp_1361, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(!ore_str_eq(", 13), lv), ore_str_new(", ", 2)), rv), ore_str_new("))", 2))));
        ore_list_push(__tmp_1361, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1361;
    }
    else if (op.tag == 7) {
        void* __tmp_1362 = ore_list_new();
        ore_list_push(__tmp_1362, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(ore_str_cmp(", 13), lv), ore_str_new(", ", 2)), rv), ore_str_new(") < 0)", 6))));
        ore_list_push(__tmp_1362, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1362;
    }
    else if (op.tag == 8) {
        void* __tmp_1363 = ore_list_new();
        ore_list_push(__tmp_1363, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(ore_str_cmp(", 13), lv), ore_str_new(", ", 2)), rv), ore_str_new(") > 0)", 6))));
        ore_list_push(__tmp_1363, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1363;
    }
    else if (op.tag == 9) {
        void* __tmp_1364 = ore_list_new();
        ore_list_push(__tmp_1364, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(ore_str_cmp(", 13), lv), ore_str_new(", ", 2)), rv), ore_str_new(") <= 0)", 7))));
        ore_list_push(__tmp_1364, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1364;
    }
    else if (op.tag == 10) {
        void* __tmp_1365 = ore_list_new();
        ore_list_push(__tmp_1365, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(ore_str_cmp(", 13), lv), ore_str_new(", ", 2)), rv), ore_str_new(") >= 0)", 7))));
        ore_list_push(__tmp_1365, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1365;
    }
    else if (op.tag == 2) {
        void* __tmp_1366 = ore_list_new();
        ore_list_push(__tmp_1366, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_repeat(", 15), lv), ore_str_new(", ", 2)), rv), ore_str_new(")", 1))));
        ore_list_push(__tmp_1366, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1366;
    }
    else {
        void* __tmp_1367 = ore_list_new();
        ore_list_push(__tmp_1367, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1367, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1367;
    }
    return __tmp_1358;
}

void* compile_pipe(void* st, void* exprs, void* stmts, int64_t left, int64_t right) {
    void* lhs = compile_expr(st, exprs, stmts, left);
    struct ore_enum_Expr re = get_expr(exprs, right);
    int64_t __tmp_1368 = 0;
    if (re.tag == 8) {
        int64_t func = re.data[0];
        int64_t args = re.data[1];
        struct ore_enum_Expr fe = get_expr(exprs, func);
        int64_t __tmp_1369 = 0;
        if (fe.tag == 4) {
            int64_t fname = fe.data[0];
            void* __tmp_1370 = ore_list_new();
            int64_t __tmp_1371 = ore_list_get(lhs, 0LL);
            int8_t __tmp_1372 = ore_list_get_kind(lhs, 0LL);
            ore_list_push(__tmp_1370, (int64_t)(__tmp_1371));
            void* arg_strs = __tmp_1370;
            void* __tmp_1373 = ore_list_new();
            int64_t __tmp_1374 = ore_list_get(lhs, 1LL);
            int8_t __tmp_1375 = ore_list_get_kind(lhs, 1LL);
            ore_list_push(__tmp_1373, (int64_t)(__tmp_1374));
            void* arg_kinds = __tmp_1373;
            for (int64_t i = 0LL; i < args.len(); i++) {
                void* a = compile_expr(st, exprs, stmts, ((args)[i]));
                int64_t __tmp_1376 = ore_list_get(a, 0LL);
                int8_t __tmp_1377 = ore_list_get_kind(a, 0LL);
                ore_list_push(arg_strs, (int64_t)(__tmp_1376));
                int64_t __tmp_1378 = ore_list_get(a, 1LL);
                int8_t __tmp_1379 = ore_list_get_kind(a, 1LL);
                ore_list_push(arg_kinds, (int64_t)(__tmp_1378));
                cont_207: ;
            }
            brk_206: ;
            return compile_fn_call_with_args(st, fname, arg_strs, arg_kinds);
        }
        else {
            __tmp_1369 = (int64_t)(cg_error(st, ore_str_new("pipe: expected function name", 28)));
        }
        return lhs;
        __tmp_1368 = (int64_t)(__tmp_1369);
    }
    else if (re.tag == 4) {
        int64_t fname = re.data[0];
        void* __tmp_1380 = ore_list_new();
        int64_t __tmp_1381 = ore_list_get(lhs, 0LL);
        int8_t __tmp_1382 = ore_list_get_kind(lhs, 0LL);
        ore_list_push(__tmp_1380, (int64_t)(__tmp_1381));
        void* __tmp_1383 = ore_list_new();
        int64_t __tmp_1384 = ore_list_get(lhs, 1LL);
        int8_t __tmp_1385 = ore_list_get_kind(lhs, 1LL);
        ore_list_push(__tmp_1383, (int64_t)(__tmp_1384));
        return compile_fn_call_with_args(st, fname, __tmp_1380, __tmp_1383);
    }
    else {
        return lhs;
        __tmp_1368 = (int64_t)(cg_error(st, ore_str_new("pipe: expected call expression", 30)));
    }
    return __tmp_1368;
}

void* compile_call(void* st, void* exprs, void* stmts, int64_t func, void* args) {
    struct ore_enum_Expr fe = get_expr(exprs, func);
    int64_t __tmp_1386 = 0;
    if (fe.tag == 4) {
        int64_t fname = fe.data[0];
        void* __tmp_1387 = ore_list_new();
        void* arg_strs = __tmp_1387;
        void* __tmp_1388 = ore_list_new();
        void* arg_kinds = __tmp_1388;
        for (int64_t i = 0LL; i < ore_list_len(args); i++) {
            int64_t __tmp_1389 = ore_list_get(args, i);
            int8_t __tmp_1390 = ore_list_get_kind(args, i);
            void* a = compile_expr(st, exprs, stmts, __tmp_1389);
            int64_t __tmp_1391 = ore_list_get(a, 0LL);
            int8_t __tmp_1392 = ore_list_get_kind(a, 0LL);
            ore_list_push(arg_strs, (int64_t)(__tmp_1391));
            int64_t __tmp_1393 = ore_list_get(a, 1LL);
            int8_t __tmp_1394 = ore_list_get_kind(a, 1LL);
            ore_list_push(arg_kinds, (int64_t)(__tmp_1393));
            cont_209: ;
        }
        brk_208: ;
        return compile_fn_call_with_args(st, fname, arg_strs, arg_kinds);
    }
    else {
        void* __tmp_1395 = ore_list_new();
        ore_list_push(__tmp_1395, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1395, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1395;
        __tmp_1386 = (int64_t)(cg_error(st, ore_str_new("call: expected function name", 28)));
    }
    return __tmp_1386;
}

void* compile_fn_call_with_args(void* st, void* fname, void* arg_strs, void* arg_kinds) {
    void* r = try_builtin(st, fname, arg_strs, arg_kinds);
    int64_t __tmp_1396 = 0;
    if ((96070740232768 > 96070740232992)) {
        return r;
    } else {
    }
    void* en = cg_variant_enum(st, fname);
    int64_t __tmp_1397 = 0;
    if ((!ore_str_eq(en, ore_str_new("", 0)))) {
        return compile_variant_call(st, fname, en, arg_strs, arg_kinds);
    } else {
    }
    int64_t __tmp_1398 = 0;
    if (cg_has_generic_fn(st, fname)) {
        return compile_generic_call(st, fname, arg_strs, arg_kinds);
    } else {
    }
    void* mn = mangle_fn(fname);
    void* all_args = ore_list_join(arg_strs, ore_str_new(", ", 2));
    void* ret_kind = cg_get_fn_ret(st, fname);
    void* __tmp_1399 = ore_list_new();
    ore_list_push(__tmp_1399, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(mn, ore_str_new("(", 1)), all_args), ore_str_new(")", 1))));
    ore_list_push(__tmp_1399, (int64_t)(intptr_t)(ret_kind));
    return __tmp_1399;
}

void* compile_generic_call(void* st, void* fname, void* arg_strs, void* arg_kinds) {
    void* gf_list = cg_get_generic_fn(st, fname);
    int64_t __tmp_1400 = 0;
    if ((96070752542848 == 96070752543088)) {
        void* __tmp_1401 = ore_list_new();
        ore_list_push(__tmp_1401, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1401, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1401;
        __tmp_1400 = (int64_t)(cg_error(st, ore_str_concat(ore_str_new("generic function not found: ", 28), fname)));
    } else {
    }
    struct ore_rec_FnDef gf = get_fn_def(gf_list, 0LL);
    void* tp_list = gf.type_params;
    void* __tmp_1402 = ore_list_new();
    void* tp_names = __tmp_1402;
    for (int64_t i = 0LL; i < ore_list_len(tp_list); i++) {
        struct ore_rec_TypeParamDef tpd = get_type_param_def(tp_list, i);
        ore_list_push(tp_names, (int64_t)(intptr_t)(tpd.name));
        cont_211: ;
    }
    brk_210: ;
    void* __tmp_1403 = ore_list_new();
    void* type_map_keys = __tmp_1403;
    void* __tmp_1404 = ore_list_new();
    void* type_map_vals = __tmp_1404;
    void* params = gf.params;
    for (int64_t i = 0LL; i < ore_list_len(params); i++) {
        int64_t __tmp_1405 = 0;
        if ((96070672139840 < 96070752614672)) {
            struct ore_rec_ParamDef p = get_param_def(params, i);
            int64_t __tmp_1406 = 0;
            if (p.ty.tag == 0) {
                int64_t ptn = p.ty.data[0];
                int64_t __tmp_1407 = 0;
                if (list_contains_str(tp_names, ptn)) {
                    int64_t __tmp_1408 = 0;
                    if ((!(list_contains_str(type_map_keys, ptn)))) {
                        ore_list_push(type_map_keys, (int64_t)(ptn));
                        ore_list_push(type_map_vals, (int64_t)(intptr_t)(str_at(arg_kinds, i)));
                    } else {
                    }
                    __tmp_1407 = (int64_t)(__tmp_1408);
                } else {
                }
                __tmp_1406 = (int64_t)(__tmp_1407);
            }
            else {
                __tmp_1406 = (int64_t)(0LL);
            }
            __tmp_1405 = (int64_t)(__tmp_1406);
        } else {
        }
        cont_213: ;
    }
    brk_212: ;
    void* __tmp_1409 = ore_list_new();
    void* suffixes = __tmp_1409;
    for (int64_t i = 0LL; i < ore_list_len(tp_names); i++) {
        void* tpn = str_at(tp_names, i);
        void* found_kind = ore_str_new("int", 3);
        for (int64_t j = 0LL; j < ore_list_len(type_map_keys); j++) {
            int64_t __tmp_1410 = 0;
            if (ore_str_eq(str_at(type_map_keys, j), tpn)) {
                found_kind = str_at(type_map_vals, j);
            } else {
            }
            cont_217: ;
        }
        brk_216: ;
        ore_list_push(suffixes, (int64_t)(intptr_t)(kind_to_suffix(found_kind)));
        cont_215: ;
    }
    brk_214: ;
    void* uscore = ore_str_new("_", 1);
    void* mono_name = ore_str_concat(ore_str_concat(fname, ore_str_new("__", 2)), ore_list_join(suffixes, uscore));
    int64_t __tmp_1411 = 0;
    if ((!(cg_has_mono(st, mono_name)))) {
        void* __tmp_1412 = ore_list_new();
        void* mono_params = __tmp_1412;
        for (int64_t i = 0LL; i < ore_list_len(params); i++) {
            struct ore_rec_ParamDef p = get_param_def(params, i);
            struct ore_enum_TypeExpr new_ty = p.ty;
            int64_t __tmp_1413 = 0;
            if (p.ty.tag == 0) {
                int64_t ptn = p.ty.data[0];
                void* resolved = resolve_type_param(ptn, type_map_keys, type_map_vals);
                int64_t __tmp_1414 = 0;
                if ((!ore_str_eq(resolved, (void*)(intptr_t)(96070672316144)))) {
                    struct ore_rec_NamedType __tmp_1415;
                    __tmp_1415.name = resolved;
                    new_ty = __tmp_1415;
                } else {
                }
                __tmp_1413 = (int64_t)(__tmp_1414);
            }
            else {
                __tmp_1413 = (int64_t)(0LL);
            }
            struct ore_rec_ParamDef __tmp_1416;
            __tmp_1416.name = p.name;
            __tmp_1416.ty = new_ty;
            __tmp_1416.default_expr = p.default_expr;
            ore_list_push(mono_params, ({ struct ore_rec_ParamDef __v2i = __tmp_1416; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_ParamDef)), &__v2i, sizeof(struct ore_rec_ParamDef)); }));
            cont_219: ;
        }
        brk_218: ;
        struct ore_enum_TypeExpr mono_ret = gf.ret_type;
        int64_t __tmp_1417 = 0;
        if (gf.ret_type.tag == 0) {
            int64_t rtn = gf.ret_type.data[0];
            void* resolved = resolve_type_param(rtn, type_map_keys, type_map_vals);
            int64_t __tmp_1418 = 0;
            if ((!ore_str_eq(resolved, (void*)(intptr_t)(96070672374064)))) {
                struct ore_rec_NamedType __tmp_1419;
                __tmp_1419.name = resolved;
                mono_ret = __tmp_1419;
            } else {
            }
            __tmp_1417 = (int64_t)(__tmp_1418);
        }
        else {
            __tmp_1417 = (int64_t)(0LL);
        }
        struct ore_rec_FnDef __tmp_1420;
        __tmp_1420.name = mono_name;
        void* __tmp_1421 = ore_list_new();
        __tmp_1420.type_params = __tmp_1421;
        __tmp_1420.params = mono_params;
        __tmp_1420.ret_type = mono_ret;
        __tmp_1420.body = gf.body;
        struct ore_rec_FnDef mono_fn = __tmp_1420;
        void* saved_lines = cg_list(st, 0LL);
        int64_t saved_indent = cg_indent(st);
        void* saved_var_names = cg_list(st, 4LL);
        void* saved_var_kinds = cg_list(st, 5LL);
        void* __tmp_1422 = ore_list_new();
        ore_list_set(st, 0LL, (int64_t)(__tmp_1422));
        void* __tmp_1423 = ore_list_new();
        ore_list_set(st, 4LL, (int64_t)(__tmp_1423));
        void* __tmp_1424 = ore_list_new();
        ore_list_set(st, 5LL, (int64_t)(__tmp_1424));
        void* exprs = cg_list(st, 23LL);
        void* stmts = cg_list(st, 24LL);
        void* mono_lines = cg_list(st, 0LL);
        void* deferred = cg_list(st, 25LL);
        for (int64_t i = 0LL; i < ore_list_len(mono_lines); i++) {
            int64_t __tmp_1425 = ore_list_get(mono_lines, i);
            int8_t __tmp_1426 = ore_list_get_kind(mono_lines, i);
            ore_list_push(deferred, (int64_t)(__tmp_1425));
            cont_221: ;
        }
        brk_220: ;
        ore_list_set(st, 0LL, (int64_t)(saved_lines));
        ore_list_set(st, 4LL, (int64_t)(saved_var_names));
        ore_list_set(st, 5LL, (int64_t)(saved_var_kinds));
        __tmp_1411 = (int64_t)(cg_set_indent(st, saved_indent));
    } else {
    }
    void* mn = mangle_fn(mono_name);
    void* all_args = ore_list_join(arg_strs, ore_str_new(", ", 2));
    void* ret_kind = cg_get_fn_ret(st, mono_name);
    void* __tmp_1427 = ore_list_new();
    ore_list_push(__tmp_1427, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(mn, ore_str_new("(", 1)), all_args), ore_str_new(")", 1))));
    ore_list_push(__tmp_1427, (int64_t)(intptr_t)(ret_kind));
    return __tmp_1427;
}

void* resolve_type_param(void* name, void* keys, void* vals) {
    for (int64_t i = 0LL; i < ore_list_len(keys); i++) {
        int64_t __tmp_1428 = 0;
        if (ore_str_eq(str_at(keys, i), name)) {
            return kind_str_to_type_name(str_at(vals, i));
        } else {
        }
        cont_223: ;
    }
    brk_222: ;
    return name;
}

int8_t list_contains_str(void* lst, void* s) {
    for (int64_t i = 0LL; i < ore_list_len(lst); i++) {
        int64_t __tmp_1429 = 0;
        if (ore_str_eq(str_at(lst, i), s)) {
            return ((int8_t)1);
        } else {
        }
        cont_225: ;
    }
    brk_224: ;
    return ((int8_t)0);
}

void* compile_variant_call(void* st, void* variant, void* enum_name, void* arg_strs, void* arg_kinds) {
    int64_t tag = find_variant_tag(st, enum_name, variant);
    void* t = cg_tmp(st);
    void* c_type = ore_str_concat(ore_str_new("struct ore_enum_", 16), enum_name);
    for (int64_t i = 0LL; i < ore_list_len(arg_strs); i++) {
        int64_t __tmp_1430 = ore_list_get(arg_strs, i);
        int8_t __tmp_1431 = ore_list_get_kind(arg_strs, i);
        cont_227: ;
    }
    brk_226: ;
    void* __tmp_1432 = ore_list_new();
    ore_list_push(__tmp_1432, (int64_t)(intptr_t)(t));
    ore_list_push(__tmp_1432, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("enum:", 5), enum_name)));
    void* __ret_1 = __tmp_1432;
    return __ret_1;
}

void* try_builtin(void* st, void* fname, void* args, void* kinds) {
    int64_t __tmp_1433 = 0;
    if (ore_str_eq(fname, ore_str_new("sqrt", 4))) {
        void* __tmp_1434 = ore_list_new();
        int64_t __tmp_1435 = ore_list_get(args, 0LL);
        int8_t __tmp_1436 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1434, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_math_sqrt((double)(", 23), ore_dynamic_to_str(__tmp_1435, __tmp_1436)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1434, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1434;
    } else {
    }
    int64_t __tmp_1437 = 0;
    if (ore_str_eq(fname, ore_str_new("sin", 3))) {
        void* __tmp_1438 = ore_list_new();
        int64_t __tmp_1439 = ore_list_get(args, 0LL);
        int8_t __tmp_1440 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1438, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_math_sin((double)(", 22), ore_dynamic_to_str(__tmp_1439, __tmp_1440)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1438, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1438;
    } else {
    }
    int64_t __tmp_1441 = 0;
    if (ore_str_eq(fname, ore_str_new("cos", 3))) {
        void* __tmp_1442 = ore_list_new();
        int64_t __tmp_1443 = ore_list_get(args, 0LL);
        int8_t __tmp_1444 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1442, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_math_cos((double)(", 22), ore_dynamic_to_str(__tmp_1443, __tmp_1444)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1442, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1442;
    } else {
    }
    int64_t __tmp_1445 = 0;
    if (ore_str_eq(fname, ore_str_new("log", 3))) {
        void* __tmp_1446 = ore_list_new();
        int64_t __tmp_1447 = ore_list_get(args, 0LL);
        int8_t __tmp_1448 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1446, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_math_log((double)(", 22), ore_dynamic_to_str(__tmp_1447, __tmp_1448)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1446, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1446;
    } else {
    }
    int64_t __tmp_1449 = 0;
    if (ore_str_eq(fname, ore_str_new("pow", 3))) {
        int64_t __tmp_1450 = 0;
        if ((96070753240128 >= 96070753240368)) {
            int64_t __tmp_1451 = ore_list_get(kinds, 0LL);
            int8_t __tmp_1452 = ore_list_get_kind(kinds, 0LL);
            int64_t __tmp_1453 = ore_list_get(kinds, 1LL);
            int8_t __tmp_1454 = ore_list_get_kind(kinds, 1LL);
            int64_t __tmp_1455 = 0;
            if ((96070753247712 || 96070753253552)) {
                void* __tmp_1456 = ore_list_new();
                int64_t __tmp_1457 = ore_list_get(args, 0LL);
                int8_t __tmp_1458 = ore_list_get_kind(args, 0LL);
                int64_t __tmp_1459 = ore_list_get(args, 1LL);
                int8_t __tmp_1460 = ore_list_get_kind(args, 1LL);
                ore_list_push(__tmp_1456, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_math_pow((double)(", 22), ore_dynamic_to_str(__tmp_1457, __tmp_1458)), ore_str_new("), (double)(", 12)), ore_dynamic_to_str(__tmp_1459, __tmp_1460)), ore_str_new("))", 2))));
                ore_list_push(__tmp_1456, (int64_t)(intptr_t)(ore_str_new("float", 5)));
                return __tmp_1456;
            } else {
            }
            __tmp_1450 = (int64_t)(__tmp_1455);
        } else {
        }
        void* __tmp_1461 = ore_list_new();
        int64_t __tmp_1462 = ore_list_get(args, 0LL);
        int8_t __tmp_1463 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1464 = ore_list_get(args, 1LL);
        int8_t __tmp_1465 = ore_list_get_kind(args, 1LL);
        ore_list_push(__tmp_1461, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_int_pow(", 12), ore_dynamic_to_str(__tmp_1462, __tmp_1463)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1464, __tmp_1465)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1461, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1461;
        __tmp_1449 = (int64_t)(__tmp_1450);
    } else {
    }
    int64_t __tmp_1466 = 0;
    if (ore_str_eq(fname, ore_str_new("abs", 3))) {
        void* __tmp_1467 = ore_list_new();
        int64_t __tmp_1468 = ore_list_get(args, 0LL);
        int8_t __tmp_1469 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1467, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_math_abs((double)(", 22), ore_dynamic_to_str(__tmp_1468, __tmp_1469)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1467, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1467;
    } else {
    }
    int64_t __tmp_1470 = 0;
    if (ore_str_eq(fname, ore_str_new("floor", 5))) {
        void* __tmp_1471 = ore_list_new();
        int64_t __tmp_1472 = ore_list_get(args, 0LL);
        int8_t __tmp_1473 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1471, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("(int64_t)ore_math_floor((double)(", 33), ore_dynamic_to_str(__tmp_1472, __tmp_1473)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1471, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1471;
    } else {
    }
    int64_t __tmp_1474 = 0;
    if (ore_str_eq(fname, ore_str_new("ceil", 4))) {
        void* __tmp_1475 = ore_list_new();
        int64_t __tmp_1476 = ore_list_get(args, 0LL);
        int8_t __tmp_1477 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1475, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("(int64_t)ore_math_ceil((double)(", 32), ore_dynamic_to_str(__tmp_1476, __tmp_1477)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1475, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1475;
    } else {
    }
    int64_t __tmp_1478 = 0;
    if (ore_str_eq(fname, ore_str_new("round", 5))) {
        int64_t __tmp_1479 = 0;
        if ((96070753377520 == 96070753377760)) {
            void* __tmp_1480 = ore_list_new();
            int64_t __tmp_1481 = ore_list_get(args, 0LL);
            int8_t __tmp_1482 = ore_list_get_kind(args, 0LL);
            int64_t __tmp_1483 = ore_list_get(args, 1LL);
            int8_t __tmp_1484 = ore_list_get_kind(args, 1LL);
            ore_list_push(__tmp_1480, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_float_round_to((double)(", 28), ore_dynamic_to_str(__tmp_1481, __tmp_1482)), ore_str_new("), ", 3)), ore_dynamic_to_str(__tmp_1483, __tmp_1484)), ore_str_new(")", 1))));
            ore_list_push(__tmp_1480, (int64_t)(intptr_t)(ore_str_new("float", 5)));
            return __tmp_1480;
        } else {
        }
        void* __tmp_1485 = ore_list_new();
        int64_t __tmp_1486 = ore_list_get(args, 0LL);
        int8_t __tmp_1487 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1485, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("(int64_t)ore_math_round((double)(", 33), ore_dynamic_to_str(__tmp_1486, __tmp_1487)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1485, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1485;
        __tmp_1478 = (int64_t)(__tmp_1479);
    } else {
    }
    int64_t __tmp_1488 = 0;
    if (ore_str_eq(fname, ore_str_new("pi", 2))) {
        void* __tmp_1489 = ore_list_new();
        ore_list_push(__tmp_1489, (int64_t)(intptr_t)(ore_str_new("ore_math_pi()", 13)));
        ore_list_push(__tmp_1489, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1489;
    } else {
    }
    int64_t __tmp_1490 = 0;
    if ((96070753440336 || 96070753443088)) {
        void* __tmp_1491 = ore_list_new();
        ore_list_push(__tmp_1491, (int64_t)(intptr_t)(ore_str_new("ore_math_e()", 12)));
        ore_list_push(__tmp_1491, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1491;
    } else {
    }
    int64_t __tmp_1492 = 0;
    if (ore_str_eq(fname, ore_str_new("rand_int", 8))) {
        void* __tmp_1493 = ore_list_new();
        int64_t __tmp_1494 = ore_list_get(args, 0LL);
        int8_t __tmp_1495 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1496 = ore_list_get(args, 1LL);
        int8_t __tmp_1497 = ore_list_get_kind(args, 1LL);
        ore_list_push(__tmp_1493, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_rand_int(", 13), ore_dynamic_to_str(__tmp_1494, __tmp_1495)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1496, __tmp_1497)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1493, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1493;
    } else {
    }
    int64_t __tmp_1498 = 0;
    if (ore_str_eq(fname, ore_str_new("time_now", 8))) {
        void* __tmp_1499 = ore_list_new();
        ore_list_push(__tmp_1499, (int64_t)(intptr_t)(ore_str_new("ore_time_now()", 14)));
        ore_list_push(__tmp_1499, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1499;
    } else {
    }
    int64_t __tmp_1500 = 0;
    if (ore_str_eq(fname, ore_str_new("time_ms", 7))) {
        void* __tmp_1501 = ore_list_new();
        ore_list_push(__tmp_1501, (int64_t)(intptr_t)(ore_str_new("ore_time_ms()", 13)));
        ore_list_push(__tmp_1501, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1501;
    } else {
    }
    int64_t __tmp_1502 = 0;
    if (ore_str_eq(fname, ore_str_new("exit", 4))) {
        int64_t __tmp_1503 = ore_list_get(args, 0LL);
        int8_t __tmp_1504 = ore_list_get_kind(args, 0LL);
        void* __tmp_1505 = ore_list_new();
        ore_list_push(__tmp_1505, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1505, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1505;
        __tmp_1502 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_exit(", 9), ore_dynamic_to_str(__tmp_1503, __tmp_1504)), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1506 = 0;
    if (ore_str_eq(fname, ore_str_new("input", 5))) {
        void* __tmp_1507 = ore_list_new();
        ore_list_push(__tmp_1507, (int64_t)(intptr_t)(ore_str_new("ore_readln()", 12)));
        ore_list_push(__tmp_1507, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1507;
    } else {
    }
    int64_t __tmp_1508 = 0;
    if (ore_str_eq(fname, ore_str_new("file_read", 9))) {
        void* __tmp_1509 = ore_list_new();
        int64_t __tmp_1510 = ore_list_get(args, 0LL);
        int8_t __tmp_1511 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1509, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_file_read(", 14), ore_dynamic_to_str(__tmp_1510, __tmp_1511)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1509, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1509;
    } else {
    }
    int64_t __tmp_1512 = 0;
    if (ore_str_eq(fname, ore_str_new("file_read_lines", 15))) {
        void* __tmp_1513 = ore_list_new();
        int64_t __tmp_1514 = ore_list_get(args, 0LL);
        int8_t __tmp_1515 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1513, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_file_read_lines(", 20), ore_dynamic_to_str(__tmp_1514, __tmp_1515)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1513, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1513;
    } else {
    }
    int64_t __tmp_1516 = 0;
    if (ore_str_eq(fname, ore_str_new("file_write", 10))) {
        void* __tmp_1517 = ore_list_new();
        int64_t __tmp_1518 = ore_list_get(args, 0LL);
        int8_t __tmp_1519 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1520 = ore_list_get(args, 1LL);
        int8_t __tmp_1521 = ore_list_get_kind(args, 1LL);
        ore_list_push(__tmp_1517, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_file_write(", 15), ore_dynamic_to_str(__tmp_1518, __tmp_1519)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1520, __tmp_1521)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1517, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1517;
    } else {
    }
    int64_t __tmp_1522 = 0;
    if (ore_str_eq(fname, ore_str_new("file_exists", 11))) {
        void* __tmp_1523 = ore_list_new();
        int64_t __tmp_1524 = ore_list_get(args, 0LL);
        int8_t __tmp_1525 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1523, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_file_exists(", 16), ore_dynamic_to_str(__tmp_1524, __tmp_1525)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1523, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1523;
    } else {
    }
    int64_t __tmp_1526 = 0;
    if (ore_str_eq(fname, ore_str_new("args", 4))) {
        void* __tmp_1527 = ore_list_new();
        ore_list_push(__tmp_1527, (int64_t)(intptr_t)(ore_str_new("ore_args()", 10)));
        ore_list_push(__tmp_1527, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1527;
    } else {
    }
    int64_t __tmp_1528 = 0;
    if (ore_str_eq(fname, ore_str_new("exec", 4))) {
        void* __tmp_1529 = ore_list_new();
        int64_t __tmp_1530 = ore_list_get(args, 0LL);
        int8_t __tmp_1531 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1529, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_exec(", 9), ore_dynamic_to_str(__tmp_1530, __tmp_1531)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1529, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1529;
    } else {
    }
    int64_t __tmp_1532 = 0;
    if (ore_str_eq(fname, ore_str_new("env_get", 7))) {
        void* __tmp_1533 = ore_list_new();
        int64_t __tmp_1534 = ore_list_get(args, 0LL);
        int8_t __tmp_1535 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1533, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_env_get(", 12), ore_dynamic_to_str(__tmp_1534, __tmp_1535)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1533, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1533;
    } else {
    }
    int64_t __tmp_1536 = 0;
    if (ore_str_eq(fname, ore_str_new("env_set", 7))) {
        int64_t __tmp_1537 = ore_list_get(args, 0LL);
        int8_t __tmp_1538 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1539 = ore_list_get(args, 1LL);
        int8_t __tmp_1540 = ore_list_get_kind(args, 1LL);
        void* __tmp_1541 = ore_list_new();
        ore_list_push(__tmp_1541, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1541, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1541;
        __tmp_1536 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_env_set(", 12), ore_dynamic_to_str(__tmp_1537, __tmp_1538)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1539, __tmp_1540)), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1542 = 0;
    if (ore_str_eq(fname, ore_str_new("type_of", 7))) {
        void* __tmp_1543 = ore_list_new();
        ore_list_push(__tmp_1543, (int64_t)(intptr_t)(ore_str_new("ore_type_of(0)", 14)));
        ore_list_push(__tmp_1543, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1543;
    } else {
    }
    int64_t __tmp_1544 = 0;
    if (ore_str_eq(fname, ore_str_new("ord", 3))) {
        void* __tmp_1545 = ore_list_new();
        int64_t __tmp_1546 = ore_list_get(args, 0LL);
        int8_t __tmp_1547 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1545, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_ord(", 8), ore_dynamic_to_str(__tmp_1546, __tmp_1547)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1545, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1545;
    } else {
    }
    int64_t __tmp_1548 = 0;
    if (ore_str_eq(fname, ore_str_new("chr", 3))) {
        void* __tmp_1549 = ore_list_new();
        int64_t __tmp_1550 = ore_list_get(args, 0LL);
        int8_t __tmp_1551 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1549, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_chr(", 8), ore_dynamic_to_str(__tmp_1550, __tmp_1551)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1549, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1549;
    } else {
    }
    int64_t __tmp_1552 = 0;
    if (ore_str_eq(fname, ore_str_new("sleep", 5))) {
        int64_t __tmp_1553 = ore_list_get(args, 0LL);
        int8_t __tmp_1554 = ore_list_get_kind(args, 0LL);
        void* __tmp_1555 = ore_list_new();
        ore_list_push(__tmp_1555, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1555, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1555;
        __tmp_1552 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_sleep(", 10), ore_dynamic_to_str(__tmp_1553, __tmp_1554)), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1556 = 0;
    if (ore_str_eq(fname, ore_str_new("channel_new", 11))) {
        void* __tmp_1557 = ore_list_new();
        ore_list_push(__tmp_1557, (int64_t)(intptr_t)(ore_str_new("ore_channel_new()", 17)));
        ore_list_push(__tmp_1557, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1557;
    } else {
    }
    int64_t __tmp_1558 = 0;
    if (ore_str_eq(fname, ore_str_new("channel_send", 12))) {
        int64_t __tmp_1559 = ore_list_get(args, 0LL);
        int8_t __tmp_1560 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1561 = ore_list_get(args, 1LL);
        int8_t __tmp_1562 = ore_list_get_kind(args, 1LL);
        void* __tmp_1563 = ore_list_new();
        ore_list_push(__tmp_1563, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1563, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1563;
        __tmp_1558 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_channel_send(", 17), ore_dynamic_to_str(__tmp_1559, __tmp_1560)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1561, __tmp_1562)), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1564 = 0;
    if (ore_str_eq(fname, ore_str_new("channel_recv", 12))) {
        void* __tmp_1565 = ore_list_new();
        int64_t __tmp_1566 = ore_list_get(args, 0LL);
        int8_t __tmp_1567 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1565, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_channel_recv(", 17), ore_dynamic_to_str(__tmp_1566, __tmp_1567)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1565, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1565;
    } else {
    }
    void* __tmp_1568 = ore_list_new();
    void* empty = __tmp_1568;
    return empty;
}

void* compile_print(void* st, void* exprs, void* stmts, int64_t inner) {
    void* r = compile_expr(st, exprs, stmts, inner);
    int64_t __tmp_1569 = ore_list_get(r, 0LL);
    int8_t __tmp_1570 = ore_list_get_kind(r, 0LL);
    int64_t val = __tmp_1569;
    void* kind = str_at(r, 1LL);
    int64_t __tmp_1571 = 0;
    if (ore_str_starts_with(kind, ore_str_new("dynamic:", 8))) {
        void* kind_var = ore_str_substr(kind, 8LL, (96070753899840 - 96070753900080));
        void* dt = cg_tmp(st);
        __tmp_1571 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_str_release(", 16), dt), ore_str_new(");", 2))));
    } else {
        int64_t __tmp_1572 = 0;
        if (ore_str_eq(kind, ore_str_new("int", 3))) {
            __tmp_1572 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_print_int(", 14), ore_dynamic_to_str(val, __tmp_1570)), ore_str_new(");", 2))));
        } else {
            int64_t __tmp_1573 = 0;
            if (ore_str_eq(kind, ore_str_new("float", 5))) {
                __tmp_1573 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_print_float(", 16), ore_dynamic_to_str(val, __tmp_1570)), ore_str_new(");", 2))));
            } else {
                int64_t __tmp_1574 = 0;
                if (ore_str_eq(kind, ore_str_new("bool", 4))) {
                    __tmp_1574 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_print_bool(", 15), ore_dynamic_to_str(val, __tmp_1570)), ore_str_new(");", 2))));
                } else {
                    int64_t __tmp_1575 = 0;
                    if (ore_str_eq(kind, ore_str_new("str", 3))) {
                        __tmp_1575 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_str_print(", 14), ore_dynamic_to_str(val, __tmp_1570)), ore_str_new(");", 2))));
                    } else {
                        int64_t __tmp_1576 = 0;
                        if (ore_str_eq(kind, ore_str_new("list", 4))) {
                            __tmp_1576 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_list_print(", 15), ore_dynamic_to_str(val, __tmp_1570)), ore_str_new(");", 2))));
                        } else {
                            int64_t __tmp_1577 = 0;
                            if (ore_str_eq(kind, ore_str_new("map", 3))) {
                                __tmp_1577 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_map_print(", 14), ore_dynamic_to_str(val, __tmp_1570)), ore_str_new(");", 2))));
                            } else {
                                __tmp_1577 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_print_int((int64_t)(", 24), ore_dynamic_to_str(val, __tmp_1570)), ore_str_new("));", 3))));
                            }
                            __tmp_1576 = (int64_t)(__tmp_1577);
                        }
                        __tmp_1575 = (int64_t)(__tmp_1576);
                    }
                    __tmp_1574 = (int64_t)(__tmp_1575);
                }
                __tmp_1573 = (int64_t)(__tmp_1574);
            }
            __tmp_1572 = (int64_t)(__tmp_1573);
        }
        __tmp_1571 = (int64_t)(__tmp_1572);
    }
    void* __tmp_1578 = ore_list_new();
    ore_list_push(__tmp_1578, (int64_t)(intptr_t)(ore_str_new("0", 1)));
    ore_list_push(__tmp_1578, (int64_t)(intptr_t)(ore_str_new("void", 4)));
    void* result = __tmp_1578;
    return result;
}

void* compile_if_else(void* st, void* exprs, void* stmts, int64_t cond, struct ore_rec_Block then_block, struct ore_rec_Block else_block) {
    void* cr = compile_expr(st, exprs, stmts, cond);
    void* t = cg_tmp(st);
    int64_t __tmp_1579 = ore_list_get(cr, 0LL);
    int8_t __tmp_1580 = ore_list_get_kind(cr, 0LL);
    void* tr = compile_block(st, exprs, stmts, then_block);
    int64_t __tmp_1581 = ore_list_get(tr, 0LL);
    int8_t __tmp_1582 = ore_list_get_kind(tr, 0LL);
    int64_t __tmp_1583 = 0;
    if ((!ore_str_eq((void*)(intptr_t)(96070754177136), ore_str_new("", 0)))) {
        int64_t __tmp_1584 = ore_list_get(tr, 0LL);
        int8_t __tmp_1585 = ore_list_get_kind(tr, 0LL);
        __tmp_1583 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(t, ore_str_new(" = (int64_t)(", 13)), ore_dynamic_to_str(__tmp_1584, __tmp_1585)), ore_str_new(");", 2))));
    } else {
    }
    void* er = compile_block(st, exprs, stmts, else_block);
    int64_t __tmp_1586 = ore_list_get(er, 0LL);
    int8_t __tmp_1587 = ore_list_get_kind(er, 0LL);
    int64_t __tmp_1588 = 0;
    if ((!ore_str_eq((void*)(intptr_t)(96070754227200), ore_str_new("", 0)))) {
        int64_t __tmp_1589 = ore_list_get(er, 0LL);
        int8_t __tmp_1590 = ore_list_get_kind(er, 0LL);
        __tmp_1588 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(t, ore_str_new(" = (int64_t)(", 13)), ore_dynamic_to_str(__tmp_1589, __tmp_1590)), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1591 = ore_list_get(tr, 1LL);
    int8_t __tmp_1592 = ore_list_get_kind(tr, 1LL);
    int64_t kind = __tmp_1591;
    int64_t __tmp_1593 = 0;
    if ((96070754267776 || 96070754270592)) {
        int64_t __tmp_1594 = ore_list_get(er, 1LL);
        int8_t __tmp_1595 = ore_list_get_kind(er, 1LL);
        kind = __tmp_1594;
    } else {
    }
    int64_t __tmp_1596 = 0;
    if ((96070754280864 || 96070754284544)) {
        kind = ore_str_new("int", 3);
    } else {
    }
    void* __tmp_1597 = ore_list_new();
    ore_list_push(__tmp_1597, (int64_t)(intptr_t)(t));
    ore_list_push(__tmp_1597, (int64_t)(intptr_t)(kind));
    void* __ret_2 = __tmp_1597;
    return __ret_2;
}

void* compile_block(void* st, void* exprs, void* stmts, struct ore_rec_Block block) {
    void* ss = block.stmts;
    void* last_val = ore_str_new("", 0);
    void* last_kind = ore_str_new("void", 4);
    for (int64_t i = 0LL; i < ore_list_len(ss); i++) {
        struct ore_rec_SpannedStmt sp = get_sstmt(ss, i);
        int64_t sid = sp.stmt_id;
        void* r = compile_stmt_by_id(st, exprs, stmts, sid);
        int64_t __tmp_1598 = ore_list_get(r, 0LL);
        int8_t __tmp_1599 = ore_list_get_kind(r, 0LL);
        int64_t __tmp_1600 = 0;
        if ((!ore_str_eq((void*)(intptr_t)(96070754348832), ore_str_new("", 0)))) {
            int64_t __tmp_1601 = ore_list_get(r, 0LL);
            int8_t __tmp_1602 = ore_list_get_kind(r, 0LL);
            last_val = __tmp_1601;
            int64_t __tmp_1603 = ore_list_get(r, 1LL);
            int8_t __tmp_1604 = ore_list_get_kind(r, 1LL);
            last_kind = __tmp_1603;
        } else {
        }
        cont_229: ;
    }
    brk_228: ;
    void* __tmp_1605 = ore_list_new();
    ore_list_push(__tmp_1605, (int64_t)(last_val));
    ore_list_push(__tmp_1605, (int64_t)(last_kind));
    void* __ret_3 = __tmp_1605;
    return __ret_3;
}

void* compile_stmt_by_id(void* st, void* exprs, void* stmts, int64_t stmt_id) {
    struct ore_enum_Stmt s = get_stmt(stmts, stmt_id);
    return compile_stmt_node(st, exprs, stmts, s);
}

void* compile_stmt_node(void* st, void* exprs, void* stmts, struct ore_enum_Stmt s) {
    int64_t __tmp_1606 = 0;
    if (s.tag == 0) {
        int64_t name = s.data[0];
        int64_t mutable = s.data[1];
        int64_t value = s.data[2];
        return compile_let(st, exprs, stmts, name, mutable, value);
    }
    else if (s.tag == 2) {
        int64_t name = s.data[0];
        int64_t value = s.data[1];
        return compile_assign(st, exprs, stmts, name, value);
    }
    else if (s.tag == 5) {
        int64_t expr = s.data[0];
        void* r = compile_expr(st, exprs, stmts, expr);
        int64_t __tmp_1607 = ore_list_get(r, 1LL);
        int8_t __tmp_1608 = ore_list_get_kind(r, 1LL);
        int64_t __tmp_1609 = 0;
        if ((!ore_str_eq((void*)(intptr_t)(96070754465744), ore_str_new("void", 4)))) {
            return r;
        } else {
        }
        void* __tmp_1610 = ore_list_new();
        ore_list_push(__tmp_1610, (int64_t)(intptr_t)(ore_str_new("", 0)));
        ore_list_push(__tmp_1610, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1610;
        __tmp_1606 = (int64_t)(__tmp_1609);
    }
    else if (s.tag == 6) {
        int64_t value = s.data[0];
        int64_t __tmp_1611 = 0;
        if ((96070673572992 >= 96070754484304)) {
            void* r = compile_expr(st, exprs, stmts, value);
            int64_t __tmp_1612 = ore_list_get(r, 0LL);
            int8_t __tmp_1613 = ore_list_get_kind(r, 0LL);
            __tmp_1611 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("return ", 7), ore_dynamic_to_str(__tmp_1612, __tmp_1613)), ore_str_new(";", 1))));
        } else {
            __tmp_1611 = (int64_t)(emit(st, ore_str_new("return;", 7)));
        }
        void* __tmp_1614 = ore_list_new();
        ore_list_push(__tmp_1614, (int64_t)(intptr_t)(ore_str_new("", 0)));
        ore_list_push(__tmp_1614, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1614;
        __tmp_1606 = (int64_t)(__tmp_1611);
    }
    else if (s.tag == 7) {
        int64_t var_name = s.data[0];
        int64_t start = s.data[1];
        int64_t end = s.data[2];
        int64_t step = s.data[3];
        int64_t body = s.data[4];
        return compile_for_in(st, exprs, stmts, var_name, start, end, step, body);
    }
    else if (s.tag == 9) {
        int64_t var_name = s.data[0];
        int64_t iterable = s.data[1];
        int64_t body = s.data[2];
        return compile_for_each(st, exprs, stmts, var_name, iterable, body);
    }
    else if (s.tag == 8) {
        int64_t cond = s.data[0];
        int64_t body = s.data[1];
        return compile_while(st, exprs, stmts, cond, body);
    }
    else if (s.tag == 11) {
        int64_t body = s.data[0];
        return compile_loop(st, exprs, stmts, body);
    }
    else if (s.tag == 12) {
        void* brk = cg_list(st, 18LL);
        int64_t __tmp_1615 = 0;
        if ((96070754611408 > 96070754611648)) {
            int64_t __tmp_1616 = ore_list_get(brk, (96070754616544 - 96070754616784));
            int8_t __tmp_1617 = ore_list_get_kind(brk, (96070754616544 - 96070754616784));
            int64_t lbl = __tmp_1616;
            __tmp_1615 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("goto ", 5), ore_dynamic_to_str(lbl, __tmp_1617)), ore_str_new(";", 1))));
        } else {
        }
        void* __tmp_1618 = ore_list_new();
        ore_list_push(__tmp_1618, (int64_t)(intptr_t)(ore_str_new("", 0)));
        ore_list_push(__tmp_1618, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1618;
        __tmp_1606 = (int64_t)(__tmp_1615);
    }
    else if (s.tag == 13) {
        void* cont = cg_list(st, 19LL);
        int64_t __tmp_1619 = 0;
        if ((96070754649856 > 96070754650096)) {
            int64_t __tmp_1620 = ore_list_get(cont, (96070754655008 - 96070754655248));
            int8_t __tmp_1621 = ore_list_get_kind(cont, (96070754655008 - 96070754655248));
            int64_t lbl = __tmp_1620;
            __tmp_1619 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("goto ", 5), ore_dynamic_to_str(lbl, __tmp_1621)), ore_str_new(";", 1))));
        } else {
        }
        void* __tmp_1622 = ore_list_new();
        ore_list_push(__tmp_1622, (int64_t)(intptr_t)(ore_str_new("", 0)));
        ore_list_push(__tmp_1622, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1622;
        __tmp_1606 = (int64_t)(__tmp_1619);
    }
    else if (s.tag == 3) {
        int64_t object = s.data[0];
        int64_t index = s.data[1];
        int64_t value = s.data[2];
        return compile_index_assign(st, exprs, stmts, object, index, value);
    }
    else if (s.tag == 4) {
        int64_t object = s.data[0];
        int64_t field = s.data[1];
        int64_t value = s.data[2];
        return compile_field_assign(st, exprs, stmts, object, field, value);
    }
    else if (s.tag == 14) {
        int64_t expr = s.data[0];
        void* __tmp_1623 = ore_list_new();
        ore_list_push(__tmp_1623, (int64_t)(intptr_t)(ore_str_new("", 0)));
        ore_list_push(__tmp_1623, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1623;
        __tmp_1606 = (int64_t)(emit(st, ore_str_new("// spawn not fully supported in C codegen", 41)));
    }
    else {
        void* __tmp_1624 = ore_list_new();
        ore_list_push(__tmp_1624, (int64_t)(intptr_t)(ore_str_new("", 0)));
        ore_list_push(__tmp_1624, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1624;
        __tmp_1606 = (int64_t)(cg_error(st, ore_str_new("unsupported statement type", 26)));
    }
    return __tmp_1606;
}

void* compile_let(void* st, void* exprs, void* stmts, void* name, int8_t mutable, int64_t value) {
    void* r = compile_expr(st, exprs, stmts, value);
    int64_t __tmp_1625 = ore_list_get(r, 1LL);
    int8_t __tmp_1626 = ore_list_get_kind(r, 1LL);
    void* c_type = kind_to_c_type(__tmp_1625);
    void* mn = mangle_var(name);
    int64_t __tmp_1627 = ore_list_get(r, 0LL);
    int8_t __tmp_1628 = ore_list_get_kind(r, 0LL);
    int64_t __tmp_1629 = ore_list_get(r, 1LL);
    int8_t __tmp_1630 = ore_list_get_kind(r, 1LL);
    void* __tmp_1631 = ore_list_new();
    ore_list_push(__tmp_1631, (int64_t)(intptr_t)(ore_str_new("", 0)));
    ore_list_push(__tmp_1631, (int64_t)(intptr_t)(ore_str_new("void", 4)));
    return __tmp_1631;
}

void* compile_assign(void* st, void* exprs, void* stmts, void* name, int64_t value) {
    void* r = compile_expr(st, exprs, stmts, value);
    void* mn = mangle_var(name);
    int64_t __tmp_1632 = ore_list_get(r, 0LL);
    int8_t __tmp_1633 = ore_list_get_kind(r, 0LL);
    int64_t __tmp_1634 = ore_list_get(r, 1LL);
    int8_t __tmp_1635 = ore_list_get_kind(r, 1LL);
    void* __tmp_1636 = ore_list_new();
    ore_list_push(__tmp_1636, (int64_t)(intptr_t)(ore_str_new("", 0)));
    ore_list_push(__tmp_1636, (int64_t)(intptr_t)(ore_str_new("void", 4)));
    return __tmp_1636;
}

void* compile_index_assign(void* st, void* exprs, void* stmts, int64_t object, int64_t index, int64_t value) {
    void* obj = compile_expr(st, exprs, stmts, object);
    void* idx = compile_expr(st, exprs, stmts, index);
    void* val = compile_expr(st, exprs, stmts, value);
    int64_t __tmp_1637 = ore_list_get(obj, 1LL);
    int8_t __tmp_1638 = ore_list_get_kind(obj, 1LL);
    int64_t ok = __tmp_1637;
    int64_t __tmp_1639 = 0;
    if (ore_str_eq((void*)(intptr_t)(96070673972880), ore_str_new("list", 4))) {
        int64_t __tmp_1640 = ore_list_get(obj, 0LL);
        int8_t __tmp_1641 = ore_list_get_kind(obj, 0LL);
        int64_t __tmp_1642 = ore_list_get(idx, 0LL);
        int8_t __tmp_1643 = ore_list_get_kind(idx, 0LL);
        int64_t __tmp_1644 = ore_list_get(val, 0LL);
        int8_t __tmp_1645 = ore_list_get_kind(val, 0LL);
        __tmp_1639 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_set(", 13), ore_dynamic_to_str(__tmp_1640, __tmp_1641)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1642, __tmp_1643)), ore_str_new(", (int64_t)(", 12)), ore_dynamic_to_str(__tmp_1644, __tmp_1645)), ore_str_new("));", 3))));
    } else {
        int64_t __tmp_1646 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070673989616), ore_str_new("map", 3))) {
            int64_t __tmp_1647 = ore_list_get(obj, 0LL);
            int8_t __tmp_1648 = ore_list_get_kind(obj, 0LL);
            int64_t __tmp_1649 = ore_list_get(idx, 0LL);
            int8_t __tmp_1650 = ore_list_get_kind(idx, 0LL);
            int64_t __tmp_1651 = ore_list_get(val, 0LL);
            int8_t __tmp_1652 = ore_list_get_kind(val, 0LL);
            __tmp_1646 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_map_set(", 12), ore_dynamic_to_str(__tmp_1647, __tmp_1648)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1649, __tmp_1650)), ore_str_new(", (int64_t)(", 12)), ore_dynamic_to_str(__tmp_1651, __tmp_1652)), ore_str_new("));", 3))));
        } else {
            int64_t __tmp_1653 = ore_list_get(obj, 0LL);
            int8_t __tmp_1654 = ore_list_get_kind(obj, 0LL);
            int64_t __tmp_1655 = ore_list_get(idx, 0LL);
            int8_t __tmp_1656 = ore_list_get_kind(idx, 0LL);
            int64_t __tmp_1657 = ore_list_get(val, 0LL);
            int8_t __tmp_1658 = ore_list_get_kind(val, 0LL);
            __tmp_1646 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_dynamic_to_str(__tmp_1653, __tmp_1654), ore_str_new("[", 1)), ore_dynamic_to_str(__tmp_1655, __tmp_1656)), ore_str_new("] = ", 4)), ore_dynamic_to_str(__tmp_1657, __tmp_1658)), ore_str_new(";", 1))));
        }
        __tmp_1639 = (int64_t)(__tmp_1646);
    }
    void* __tmp_1659 = ore_list_new();
    ore_list_push(__tmp_1659, (int64_t)(intptr_t)(ore_str_new("", 0)));
    ore_list_push(__tmp_1659, (int64_t)(intptr_t)(ore_str_new("void", 4)));
    void* __ret_4 = __tmp_1659;
    return __ret_4;
}

void* compile_field_assign(void* st, void* exprs, void* stmts, int64_t object, void* field, int64_t value) {
    void* obj = compile_expr(st, exprs, stmts, object);
    void* val = compile_expr(st, exprs, stmts, value);
    int64_t __tmp_1660 = ore_list_get(obj, 0LL);
    int8_t __tmp_1661 = ore_list_get_kind(obj, 0LL);
    int64_t __tmp_1662 = ore_list_get(val, 0LL);
    int8_t __tmp_1663 = ore_list_get_kind(val, 0LL);
    void* __tmp_1664 = ore_list_new();
    ore_list_push(__tmp_1664, (int64_t)(intptr_t)(ore_str_new("", 0)));
    ore_list_push(__tmp_1664, (int64_t)(intptr_t)(ore_str_new("void", 4)));
    return __tmp_1664;
}

void* compile_for_in(void* st, void* exprs, void* stmts, void* var_name, int64_t start, int64_t end, int64_t step, struct ore_rec_Block body) {
    void* sv = compile_expr(st, exprs, stmts, start);
    void* ev = compile_expr(st, exprs, stmts, end);
    void* brk_lbl = cg_label(st, ore_str_new("brk", 3));
    void* cont_lbl = cg_label(st, ore_str_new("cont", 4));
    ore_list_push(cg_list(st, 18LL), (int64_t)(intptr_t)(brk_lbl));
    ore_list_push(cg_list(st, 19LL), (int64_t)(intptr_t)(cont_lbl));
    void* mn = mangle_var(var_name);
    int64_t __tmp_1665 = 0;
    if ((96070674166848 >= 96070755192016)) {
        void* step_val = compile_expr(st, exprs, stmts, step);
        int64_t __tmp_1666 = ore_list_get(sv, 0LL);
        int8_t __tmp_1667 = ore_list_get_kind(sv, 0LL);
        int64_t __tmp_1668 = ore_list_get(ev, 0LL);
        int8_t __tmp_1669 = ore_list_get_kind(ev, 0LL);
        int64_t __tmp_1670 = ore_list_get(step_val, 0LL);
        int8_t __tmp_1671 = ore_list_get_kind(step_val, 0LL);
        __tmp_1665 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("for (int64_t ", 13), mn), ore_str_new(" = ", 3)), ore_dynamic_to_str(__tmp_1666, __tmp_1667)), ore_str_new("; ", 2)), mn), ore_str_new(" < ", 3)), ore_dynamic_to_str(__tmp_1668, __tmp_1669)), ore_str_new("; ", 2)), mn), ore_str_new(" += ", 4)), ore_dynamic_to_str(__tmp_1670, __tmp_1671)), ore_str_new(") {", 3))));
    } else {
        int64_t __tmp_1672 = ore_list_get(sv, 0LL);
        int8_t __tmp_1673 = ore_list_get_kind(sv, 0LL);
        int64_t __tmp_1674 = ore_list_get(ev, 0LL);
        int8_t __tmp_1675 = ore_list_get_kind(ev, 0LL);
        __tmp_1665 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("for (int64_t ", 13), mn), ore_str_new(" = ", 3)), ore_dynamic_to_str(__tmp_1672, __tmp_1673)), ore_str_new("; ", 2)), mn), ore_str_new(" < ", 3)), ore_dynamic_to_str(__tmp_1674, __tmp_1675)), ore_str_new("; ", 2)), mn), ore_str_new("++) {", 5))));
    }
    void* __tmp_1676 = ore_list_new();
    ore_list_push(__tmp_1676, (int64_t)(intptr_t)(ore_str_new("", 0)));
    ore_list_push(__tmp_1676, (int64_t)(intptr_t)(ore_str_new("void", 4)));
    return __tmp_1676;
}

void* compile_for_each(void* st, void* exprs, void* stmts, void* var_name, int64_t iterable, struct ore_rec_Block body) {
    void* it = compile_expr(st, exprs, stmts, iterable);
    void* brk_lbl = cg_label(st, ore_str_new("brk", 3));
    void* cont_lbl = cg_label(st, ore_str_new("cont", 4));
    ore_list_push(cg_list(st, 18LL), (int64_t)(intptr_t)(brk_lbl));
    ore_list_push(cg_list(st, 19LL), (int64_t)(intptr_t)(cont_lbl));
    void* mn = mangle_var(var_name);
    void* iter_tmp = cg_tmp(st);
    void* len_tmp = cg_tmp(st);
    void* idx_tmp = cg_tmp(st);
    int64_t __tmp_1677 = ore_list_get(it, 0LL);
    int8_t __tmp_1678 = ore_list_get_kind(it, 0LL);
    void* elem_kind = ore_str_new("int", 3);
    int64_t __tmp_1679 = ore_list_get(it, 1LL);
    int8_t __tmp_1680 = ore_list_get_kind(it, 1LL);
    int64_t __tmp_1681 = 0;
    if (ore_str_eq((void*)(intptr_t)(96070755501184), ore_str_new("list", 4))) {
        elem_kind = ore_str_new("int", 3);
    } else {
    }
    void* c_elem_type = kind_to_c_type(elem_kind);
    void* __tmp_1682 = ore_list_new();
    ore_list_push(__tmp_1682, (int64_t)(intptr_t)(ore_str_new("", 0)));
    ore_list_push(__tmp_1682, (int64_t)(intptr_t)(ore_str_new("void", 4)));
    return __tmp_1682;
}

void* compile_while(void* st, void* exprs, void* stmts, int64_t cond, struct ore_rec_Block body) {
    void* brk_lbl = cg_label(st, ore_str_new("brk", 3));
    void* cont_lbl = cg_label(st, ore_str_new("cont", 4));
    ore_list_push(cg_list(st, 18LL), (int64_t)(intptr_t)(brk_lbl));
    ore_list_push(cg_list(st, 19LL), (int64_t)(intptr_t)(cont_lbl));
    void* cr = compile_expr(st, exprs, stmts, cond);
    int64_t __tmp_1683 = ore_list_get(cr, 0LL);
    int8_t __tmp_1684 = ore_list_get_kind(cr, 0LL);
    void* __tmp_1685 = ore_list_new();
    ore_list_push(__tmp_1685, (int64_t)(intptr_t)(ore_str_new("", 0)));
    ore_list_push(__tmp_1685, (int64_t)(intptr_t)(ore_str_new("void", 4)));
    return __tmp_1685;
}

void* compile_loop(void* st, void* exprs, void* stmts, struct ore_rec_Block body) {
    void* brk_lbl = cg_label(st, ore_str_new("brk", 3));
    void* cont_lbl = cg_label(st, ore_str_new("cont", 4));
    ore_list_push(cg_list(st, 18LL), (int64_t)(intptr_t)(brk_lbl));
    ore_list_push(cg_list(st, 19LL), (int64_t)(intptr_t)(cont_lbl));
    void* __tmp_1686 = ore_list_new();
    ore_list_push(__tmp_1686, (int64_t)(intptr_t)(ore_str_new("", 0)));
    ore_list_push(__tmp_1686, (int64_t)(intptr_t)(ore_str_new("void", 4)));
    return __tmp_1686;
}

void* compile_string_interp(void* st, void* exprs, void* stmts, void* parts) {
    void* result = ore_str_new("", 0);
    int8_t first = ((int8_t)1);
    for (int64_t i = 0LL; i < ore_list_len(parts); i++) {
        struct ore_enum_StringPart p = get_string_part(parts, i);
        int64_t __tmp_1687 = 0;
        if (p.tag == 0) {
            int64_t value = p.data[0];
            void* lit = compile_string_lit(st, value);
            int64_t __tmp_1688 = 0;
            if (first) {
                int64_t __tmp_1689 = ore_list_get(lit, 0LL);
                int8_t __tmp_1690 = ore_list_get_kind(lit, 0LL);
                result = __tmp_1689;
                first = ((int8_t)0);
            } else {
                int64_t __tmp_1691 = ore_list_get(lit, 0LL);
                int8_t __tmp_1692 = ore_list_get_kind(lit, 0LL);
                result = ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_concat(", 15), ore_dynamic_to_str(result, __tmp_1690)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1691, __tmp_1692)), ore_str_new(")", 1));
            }
            __tmp_1687 = (int64_t)(__tmp_1688);
        }
        else if (p.tag == 1) {
            int64_t expr_id = p.data[0];
            void* r = compile_expr(st, exprs, stmts, expr_id);
            int64_t __tmp_1693 = ore_list_get(r, 0LL);
            int8_t __tmp_1694 = ore_list_get_kind(r, 0LL);
            int64_t s_expr = __tmp_1693;
            void* kind = str_at(r, 1LL);
            int64_t __tmp_1695 = 0;
            if (ore_str_starts_with(kind, ore_str_new("dynamic:", 8))) {
                void* kind_var = ore_str_substr(kind, 8LL, (96070755931088 - 96070755931328));
                int64_t __tmp_1696 = ore_list_get(r, 0LL);
                int8_t __tmp_1697 = ore_list_get_kind(r, 0LL);
                s_expr = ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_dynamic_to_str(", 19), ore_dynamic_to_str(__tmp_1696, __tmp_1697)), ore_str_new(", ", 2)), kind_var), ore_str_new(")", 1));
            } else {
                int64_t __tmp_1698 = 0;
                if (ore_str_eq(kind, ore_str_new("int", 3))) {
                    int64_t __tmp_1699 = ore_list_get(r, 0LL);
                    int8_t __tmp_1700 = ore_list_get_kind(r, 0LL);
                    s_expr = ore_str_concat(ore_str_concat(ore_str_new("ore_int_to_str(", 15), ore_dynamic_to_str(__tmp_1699, __tmp_1700)), ore_str_new(")", 1));
                } else {
                    int64_t __tmp_1701 = 0;
                    if (ore_str_eq(kind, ore_str_new("float", 5))) {
                        int64_t __tmp_1702 = ore_list_get(r, 0LL);
                        int8_t __tmp_1703 = ore_list_get_kind(r, 0LL);
                        s_expr = ore_str_concat(ore_str_concat(ore_str_new("ore_float_to_str(", 17), ore_dynamic_to_str(__tmp_1702, __tmp_1703)), ore_str_new(")", 1));
                    } else {
                        int64_t __tmp_1704 = 0;
                        if (ore_str_eq(kind, ore_str_new("bool", 4))) {
                            int64_t __tmp_1705 = ore_list_get(r, 0LL);
                            int8_t __tmp_1706 = ore_list_get_kind(r, 0LL);
                            s_expr = ore_str_concat(ore_str_concat(ore_str_new("ore_bool_to_str(", 16), ore_dynamic_to_str(__tmp_1705, __tmp_1706)), ore_str_new(")", 1));
                        } else {
                            int64_t __tmp_1707 = 0;
                            if ((!ore_str_eq(kind, ore_str_new("str", 3)))) {
                                int64_t __tmp_1708 = ore_list_get(r, 0LL);
                                int8_t __tmp_1709 = ore_list_get_kind(r, 0LL);
                                s_expr = ore_str_concat(ore_str_concat(ore_str_new("ore_int_to_str((int64_t)(", 25), ore_dynamic_to_str(__tmp_1708, __tmp_1709)), ore_str_new("))", 2));
                            } else {
                            }
                            __tmp_1704 = (int64_t)(__tmp_1707);
                        }
                        __tmp_1701 = (int64_t)(__tmp_1704);
                    }
                    __tmp_1698 = (int64_t)(__tmp_1701);
                }
                __tmp_1695 = (int64_t)(__tmp_1698);
            }
            int64_t __tmp_1710 = 0;
            if (first) {
                result = s_expr;
                first = ((int8_t)0);
            } else {
                result = ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_concat(", 15), result), ore_str_new(", ", 2)), s_expr), ore_str_new(")", 1));
            }
            __tmp_1687 = (int64_t)(__tmp_1710);
        }
        else {
            result = result;
        }
        cont_231: ;
    }
    brk_230: ;
    int64_t __tmp_1711 = 0;
    if (first) {
        result = ore_str_new("ore_str_new(\"\", 0)", 18);
    } else {
    }
    void* __tmp_1712 = ore_list_new();
    ore_list_push(__tmp_1712, (int64_t)(intptr_t)(result));
    ore_list_push(__tmp_1712, (int64_t)(intptr_t)(ore_str_new("str", 3)));
    void* __ret_5 = __tmp_1712;
    return __ret_5;
}

void* compile_list_lit(void* st, void* exprs, void* stmts, void* elements) {
    void* t = cg_tmp(st);
    for (int64_t i = 0LL; i < ore_list_len(elements); i++) {
        int64_t __tmp_1713 = ore_list_get(elements, i);
        int8_t __tmp_1714 = ore_list_get_kind(elements, i);
        void* r = compile_expr(st, exprs, stmts, __tmp_1713);
        int64_t __tmp_1715 = ore_list_get(r, 0LL);
        int8_t __tmp_1716 = ore_list_get_kind(r, 0LL);
        int64_t __tmp_1717 = ore_list_get(r, 1LL);
        int8_t __tmp_1718 = ore_list_get_kind(r, 1LL);
        void* i64val = value_to_i64_expr(st, __tmp_1715, __tmp_1717);
        cont_233: ;
    }
    brk_232: ;
    void* __tmp_1719 = ore_list_new();
    ore_list_push(__tmp_1719, (int64_t)(intptr_t)(t));
    ore_list_push(__tmp_1719, (int64_t)(intptr_t)(ore_str_new("list", 4)));
    void* __ret_6 = __tmp_1719;
    return __ret_6;
}

void* compile_map_lit(void* st, void* exprs, void* stmts, void* entries) {
    void* t = cg_tmp(st);
    int64_t i = 0LL;
    while ((96070756202752 < 96070756204688)) {
        int64_t __tmp_1720 = ore_list_get(entries, i);
        int8_t __tmp_1721 = ore_list_get_kind(entries, i);
        void* k = compile_expr(st, exprs, stmts, __tmp_1720);
        int64_t __tmp_1722 = ore_list_get(entries, (96070675028784 + 96070756227600));
        int8_t __tmp_1723 = ore_list_get_kind(entries, (96070675028784 + 96070756227600));
        void* v = compile_expr(st, exprs, stmts, __tmp_1722);
        int64_t __tmp_1724 = ore_list_get(k, 0LL);
        int8_t __tmp_1725 = ore_list_get_kind(k, 0LL);
        int64_t __tmp_1726 = ore_list_get(v, 0LL);
        int8_t __tmp_1727 = ore_list_get_kind(v, 0LL);
        i = (96070675045088 + 96070756264304);
        cont_235: ;
    }
    brk_234: ;
    void* __tmp_1728 = ore_list_new();
    ore_list_push(__tmp_1728, (int64_t)(intptr_t)(t));
    ore_list_push(__tmp_1728, (int64_t)(intptr_t)(ore_str_new("map", 3)));
    void* __ret_7 = __tmp_1728;
    return __ret_7;
}

void* compile_index(void* st, void* exprs, void* stmts, int64_t object, int64_t index) {
    void* obj = compile_expr(st, exprs, stmts, object);
    void* idx = compile_expr(st, exprs, stmts, index);
    int64_t __tmp_1729 = ore_list_get(obj, 1LL);
    int8_t __tmp_1730 = ore_list_get_kind(obj, 1LL);
    int64_t __tmp_1731 = 0;
    if (ore_str_eq((void*)(intptr_t)(96070756311232), ore_str_new("list", 4))) {
        void* vt = cg_tmp(st);
        void* kt = cg_tmp(st);
        int64_t __tmp_1732 = ore_list_get(obj, 0LL);
        int8_t __tmp_1733 = ore_list_get_kind(obj, 0LL);
        int64_t __tmp_1734 = ore_list_get(idx, 0LL);
        int8_t __tmp_1735 = ore_list_get_kind(idx, 0LL);
        int64_t __tmp_1736 = ore_list_get(obj, 0LL);
        int8_t __tmp_1737 = ore_list_get_kind(obj, 0LL);
        int64_t __tmp_1738 = ore_list_get(idx, 0LL);
        int8_t __tmp_1739 = ore_list_get_kind(idx, 0LL);
        void* __tmp_1740 = ore_list_new();
        ore_list_push(__tmp_1740, (int64_t)(intptr_t)(vt));
        ore_list_push(__tmp_1740, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("dynamic:", 8), kt)));
        return __tmp_1740;
        __tmp_1731 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("int8_t ", 7), kt), ore_str_new(" = ore_list_get_kind(", 21)), ore_dynamic_to_str(__tmp_1736, __tmp_1737)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1738, __tmp_1739)), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1741 = ore_list_get(obj, 1LL);
    int8_t __tmp_1742 = ore_list_get_kind(obj, 1LL);
    int64_t __tmp_1743 = 0;
    if (ore_str_eq((void*)(intptr_t)(96070756397120), ore_str_new("map", 3))) {
        void* __tmp_1744 = ore_list_new();
        int64_t __tmp_1745 = ore_list_get(obj, 0LL);
        int8_t __tmp_1746 = ore_list_get_kind(obj, 0LL);
        int64_t __tmp_1747 = ore_list_get(idx, 0LL);
        int8_t __tmp_1748 = ore_list_get_kind(idx, 0LL);
        ore_list_push(__tmp_1744, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_map_get(", 12), ore_dynamic_to_str(__tmp_1745, __tmp_1746)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1747, __tmp_1748)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1744, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1744;
    } else {
    }
    int64_t __tmp_1749 = ore_list_get(obj, 1LL);
    int8_t __tmp_1750 = ore_list_get_kind(obj, 1LL);
    int64_t __tmp_1751 = 0;
    if (ore_str_eq((void*)(intptr_t)(96070756424336), ore_str_new("str", 3))) {
        void* __tmp_1752 = ore_list_new();
        int64_t __tmp_1753 = ore_list_get(obj, 0LL);
        int8_t __tmp_1754 = ore_list_get_kind(obj, 0LL);
        int64_t __tmp_1755 = ore_list_get(idx, 0LL);
        int8_t __tmp_1756 = ore_list_get_kind(idx, 0LL);
        ore_list_push(__tmp_1752, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_char_at(", 16), ore_dynamic_to_str(__tmp_1753, __tmp_1754)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1755, __tmp_1756)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1752, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1752;
    } else {
    }
    void* __tmp_1757 = ore_list_new();
    int64_t __tmp_1758 = ore_list_get(obj, 0LL);
    int8_t __tmp_1759 = ore_list_get_kind(obj, 0LL);
    int64_t __tmp_1760 = ore_list_get(idx, 0LL);
    int8_t __tmp_1761 = ore_list_get_kind(idx, 0LL);
    ore_list_push(__tmp_1757, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("((", 2), ore_dynamic_to_str(__tmp_1758, __tmp_1759)), ore_str_new(")[", 2)), ore_dynamic_to_str(__tmp_1760, __tmp_1761)), ore_str_new("])", 2))));
    ore_list_push(__tmp_1757, (int64_t)(intptr_t)(ore_str_new("int", 3)));
    void* __ret_8 = __tmp_1757;
    return __ret_8;
}

void* compile_field_access(void* st, void* exprs, void* stmts, int64_t object, void* field) {
    void* obj = compile_expr(st, exprs, stmts, object);
    void* kind = str_at(obj, 1LL);
    int64_t __tmp_1762 = 0;
    if ((96070756505280 || 96070756509040)) {
        int64_t __tmp_1763 = 0;
        if (ore_str_eq(field, ore_str_new("tag", 3))) {
            void* __tmp_1764 = ore_list_new();
            int64_t __tmp_1765 = ore_list_get(obj, 0LL);
            int8_t __tmp_1766 = ore_list_get_kind(obj, 0LL);
            ore_list_push(__tmp_1764, (int64_t)(intptr_t)(ore_str_concat(ore_dynamic_to_str(__tmp_1765, __tmp_1766), ore_str_new(".tag", 4))));
            ore_list_push(__tmp_1764, (int64_t)(intptr_t)(ore_str_new("int", 3)));
            return __tmp_1764;
        } else {
        }
        int64_t __tmp_1767 = 0;
        if (ore_str_eq(field, ore_str_new("value", 5))) {
            void* __tmp_1768 = ore_list_new();
            int64_t __tmp_1769 = ore_list_get(obj, 0LL);
            int8_t __tmp_1770 = ore_list_get_kind(obj, 0LL);
            ore_list_push(__tmp_1768, (int64_t)(intptr_t)(ore_str_concat(ore_dynamic_to_str(__tmp_1769, __tmp_1770), ore_str_new(".value", 6))));
            ore_list_push(__tmp_1768, (int64_t)(intptr_t)(ore_str_new("int", 3)));
            return __tmp_1768;
        } else {
        }
        __tmp_1762 = (int64_t)(__tmp_1767);
    } else {
    }
    int64_t __tmp_1771 = 0;
    if (ore_str_starts_with(kind, ore_str_new("rec:", 4))) {
        void* rec_name = ore_str_substr(kind, 4LL, (96070756550224 - 96070756550464));
        void* fk = cg_get_record_field_kind(st, rec_name, field);
        void* __tmp_1772 = ore_list_new();
        int64_t __tmp_1773 = ore_list_get(obj, 0LL);
        int8_t __tmp_1774 = ore_list_get_kind(obj, 0LL);
        ore_list_push(__tmp_1772, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_dynamic_to_str(__tmp_1773, __tmp_1774), ore_str_new(".", 1)), field)));
        ore_list_push(__tmp_1772, (int64_t)(intptr_t)(fk));
        return __tmp_1772;
    } else {
    }
    int64_t __tmp_1775 = 0;
    if (ore_str_starts_with(kind, ore_str_new("enum:", 5))) {
        int64_t __tmp_1776 = 0;
        if (ore_str_eq(field, ore_str_new("tag", 3))) {
            void* __tmp_1777 = ore_list_new();
            int64_t __tmp_1778 = ore_list_get(obj, 0LL);
            int8_t __tmp_1779 = ore_list_get_kind(obj, 0LL);
            ore_list_push(__tmp_1777, (int64_t)(intptr_t)(ore_str_concat(ore_dynamic_to_str(__tmp_1778, __tmp_1779), ore_str_new(".tag", 4))));
            ore_list_push(__tmp_1777, (int64_t)(intptr_t)(ore_str_new("int", 3)));
            return __tmp_1777;
        } else {
        }
        void* __tmp_1780 = ore_list_new();
        int64_t __tmp_1781 = ore_list_get(obj, 0LL);
        int8_t __tmp_1782 = ore_list_get_kind(obj, 0LL);
        ore_list_push(__tmp_1780, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_dynamic_to_str(__tmp_1781, __tmp_1782), ore_str_new(".", 1)), field)));
        ore_list_push(__tmp_1780, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1780;
        __tmp_1775 = (int64_t)(__tmp_1776);
    } else {
    }
    void* __tmp_1783 = ore_list_new();
    int64_t __tmp_1784 = ore_list_get(obj, 0LL);
    int8_t __tmp_1785 = ore_list_get_kind(obj, 0LL);
    ore_list_push(__tmp_1783, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_dynamic_to_str(__tmp_1784, __tmp_1785), ore_str_new(".", 1)), field)));
    ore_list_push(__tmp_1783, (int64_t)(intptr_t)(ore_str_new("int", 3)));
    void* __ret_9 = __tmp_1783;
    return __ret_9;
}

void* compile_method_call(void* st, void* exprs, void* stmts, int64_t object, void* method, void* args) {
    void* obj = compile_expr(st, exprs, stmts, object);
    void* __tmp_1786 = ore_list_new();
    void* arg_strs = __tmp_1786;
    void* __tmp_1787 = ore_list_new();
    void* arg_kinds = __tmp_1787;
    for (int64_t i = 0LL; i < ore_list_len(args); i++) {
        int64_t __tmp_1788 = ore_list_get(args, i);
        int8_t __tmp_1789 = ore_list_get_kind(args, i);
        void* a = compile_expr(st, exprs, stmts, __tmp_1788);
        int64_t __tmp_1790 = ore_list_get(a, 0LL);
        int8_t __tmp_1791 = ore_list_get_kind(a, 0LL);
        ore_list_push(arg_strs, (int64_t)(__tmp_1790));
        int64_t __tmp_1792 = ore_list_get(a, 1LL);
        int8_t __tmp_1793 = ore_list_get_kind(a, 1LL);
        ore_list_push(arg_kinds, (int64_t)(__tmp_1792));
        cont_237: ;
    }
    brk_236: ;
    void* kind = str_at(obj, 1LL);
    int64_t __tmp_1794 = 0;
    if (ore_str_eq(kind, ore_str_new("str", 3))) {
        int64_t __tmp_1795 = ore_list_get(obj, 0LL);
        int8_t __tmp_1796 = ore_list_get_kind(obj, 0LL);
        return compile_str_method(st, __tmp_1795, method, arg_strs, arg_kinds);
    } else {
    }
    int64_t __tmp_1797 = 0;
    if (ore_str_eq(kind, ore_str_new("list", 4))) {
        int64_t __tmp_1798 = ore_list_get(obj, 0LL);
        int8_t __tmp_1799 = ore_list_get_kind(obj, 0LL);
        return compile_list_method(st, __tmp_1798, method, arg_strs, arg_kinds);
    } else {
    }
    int64_t __tmp_1800 = 0;
    if (ore_str_eq(kind, ore_str_new("map", 3))) {
        int64_t __tmp_1801 = ore_list_get(obj, 0LL);
        int8_t __tmp_1802 = ore_list_get_kind(obj, 0LL);
        return compile_map_method(st, __tmp_1801, method, arg_strs, arg_kinds);
    } else {
    }
    int64_t __tmp_1803 = 0;
    if ((96070756749856 || 96070756753616)) {
        int64_t __tmp_1804 = ore_list_get(obj, 0LL);
        int8_t __tmp_1805 = ore_list_get_kind(obj, 0LL);
        return compile_option_method(st, __tmp_1804, kind, method, arg_strs, arg_kinds);
    } else {
    }
    void* all_args = ore_list_join(arg_strs, ore_str_new(", ", 2));
    int64_t __tmp_1806 = 0;
    if ((!ore_str_eq(all_args, ore_str_new("", 0)))) {
        void* __tmp_1807 = ore_list_new();
        int64_t __tmp_1808 = ore_list_get(obj, 0LL);
        int8_t __tmp_1809 = ore_list_get_kind(obj, 0LL);
        ore_list_push(__tmp_1807, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_dynamic_to_str(__tmp_1808, __tmp_1809), ore_str_new(".", 1)), method), ore_str_new("(", 1)), all_args), ore_str_new(")", 1))));
        ore_list_push(__tmp_1807, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        __tmp_1806 = (int64_t)(__tmp_1807);
    } else {
        void* __tmp_1810 = ore_list_new();
        int64_t __tmp_1811 = ore_list_get(obj, 0LL);
        int8_t __tmp_1812 = ore_list_get_kind(obj, 0LL);
        ore_list_push(__tmp_1810, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_dynamic_to_str(__tmp_1811, __tmp_1812), ore_str_new(".", 1)), method), ore_str_new("()", 2))));
        ore_list_push(__tmp_1810, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        __tmp_1806 = (int64_t)(__tmp_1810);
    }
    return __tmp_1806;
}

void* compile_str_method(void* st, void* obj, void* method, void* args, void* kinds) {
    int64_t __tmp_1813 = 0;
    if (ore_str_eq(method, ore_str_new("len", 3))) {
        void* __tmp_1814 = ore_list_new();
        ore_list_push(__tmp_1814, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_str_len(", 12), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1814, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1814;
    } else {
    }
    int64_t __tmp_1815 = 0;
    if (ore_str_eq(method, ore_str_new("contains", 8))) {
        void* __tmp_1816 = ore_list_new();
        int64_t __tmp_1817 = ore_list_get(args, 0LL);
        int8_t __tmp_1818 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1816, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_contains(", 17), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1817, __tmp_1818)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1816, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1816;
    } else {
    }
    int64_t __tmp_1819 = 0;
    if (ore_str_eq(method, ore_str_new("starts_with", 11))) {
        void* __tmp_1820 = ore_list_new();
        int64_t __tmp_1821 = ore_list_get(args, 0LL);
        int8_t __tmp_1822 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1820, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_starts_with(", 20), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1821, __tmp_1822)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1820, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1820;
    } else {
    }
    int64_t __tmp_1823 = 0;
    if (ore_str_eq(method, ore_str_new("ends_with", 9))) {
        void* __tmp_1824 = ore_list_new();
        int64_t __tmp_1825 = ore_list_get(args, 0LL);
        int8_t __tmp_1826 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1824, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_ends_with(", 18), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1825, __tmp_1826)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1824, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1824;
    } else {
    }
    int64_t __tmp_1827 = 0;
    if (ore_str_eq(method, ore_str_new("trim", 4))) {
        void* __tmp_1828 = ore_list_new();
        ore_list_push(__tmp_1828, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_str_trim(", 13), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1828, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1828;
    } else {
    }
    int64_t __tmp_1829 = 0;
    if (ore_str_eq(method, ore_str_new("to_upper", 8))) {
        void* __tmp_1830 = ore_list_new();
        ore_list_push(__tmp_1830, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_str_to_upper(", 17), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1830, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1830;
    } else {
    }
    int64_t __tmp_1831 = 0;
    if (ore_str_eq(method, ore_str_new("to_lower", 8))) {
        void* __tmp_1832 = ore_list_new();
        ore_list_push(__tmp_1832, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_str_to_lower(", 17), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1832, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1832;
    } else {
    }
    int64_t __tmp_1833 = 0;
    if (ore_str_eq(method, ore_str_new("split", 5))) {
        void* __tmp_1834 = ore_list_new();
        int64_t __tmp_1835 = ore_list_get(args, 0LL);
        int8_t __tmp_1836 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1834, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_split(", 14), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1835, __tmp_1836)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1834, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1834;
    } else {
    }
    int64_t __tmp_1837 = 0;
    if (ore_str_eq(method, ore_str_new("replace", 7))) {
        void* __tmp_1838 = ore_list_new();
        int64_t __tmp_1839 = ore_list_get(args, 0LL);
        int8_t __tmp_1840 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1841 = ore_list_get(args, 1LL);
        int8_t __tmp_1842 = ore_list_get_kind(args, 1LL);
        ore_list_push(__tmp_1838, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_replace(", 16), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1839, __tmp_1840)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1841, __tmp_1842)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1838, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1838;
    } else {
    }
    int64_t __tmp_1843 = 0;
    if (ore_str_eq(method, ore_str_new("substr", 6))) {
        void* __tmp_1844 = ore_list_new();
        int64_t __tmp_1845 = ore_list_get(args, 0LL);
        int8_t __tmp_1846 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1847 = ore_list_get(args, 1LL);
        int8_t __tmp_1848 = ore_list_get_kind(args, 1LL);
        ore_list_push(__tmp_1844, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_substr(", 15), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1845, __tmp_1846)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1847, __tmp_1848)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1844, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1844;
    } else {
    }
    int64_t __tmp_1849 = 0;
    if (ore_str_eq(method, ore_str_new("chars", 5))) {
        void* __tmp_1850 = ore_list_new();
        ore_list_push(__tmp_1850, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_str_chars(", 14), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1850, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1850;
    } else {
    }
    int64_t __tmp_1851 = 0;
    if (ore_str_eq(method, ore_str_new("repeat", 6))) {
        void* __tmp_1852 = ore_list_new();
        int64_t __tmp_1853 = ore_list_get(args, 0LL);
        int8_t __tmp_1854 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1852, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_repeat(", 15), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1853, __tmp_1854)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1852, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1852;
    } else {
    }
    int64_t __tmp_1855 = 0;
    if (ore_str_eq(method, ore_str_new("reverse", 7))) {
        void* __tmp_1856 = ore_list_new();
        ore_list_push(__tmp_1856, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_str_reverse(", 16), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1856, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1856;
    } else {
    }
    int64_t __tmp_1857 = 0;
    if (ore_str_eq(method, ore_str_new("char_at", 7))) {
        void* __tmp_1858 = ore_list_new();
        int64_t __tmp_1859 = ore_list_get(args, 0LL);
        int8_t __tmp_1860 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1858, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_char_at(", 16), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1859, __tmp_1860)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1858, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1858;
    } else {
    }
    int64_t __tmp_1861 = 0;
    if (ore_str_eq(method, ore_str_new("to_int", 6))) {
        void* __tmp_1862 = ore_list_new();
        ore_list_push(__tmp_1862, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_str_to_int(", 15), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1862, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1862;
    } else {
    }
    int64_t __tmp_1863 = 0;
    if (ore_str_eq(method, ore_str_new("to_float", 8))) {
        void* __tmp_1864 = ore_list_new();
        ore_list_push(__tmp_1864, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_str_to_float(", 17), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1864, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1864;
    } else {
    }
    int64_t __tmp_1865 = 0;
    if (ore_str_eq(method, ore_str_new("index_of", 8))) {
        void* __tmp_1866 = ore_list_new();
        int64_t __tmp_1867 = ore_list_get(args, 0LL);
        int8_t __tmp_1868 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1866, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_index_of(", 17), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1867, __tmp_1868)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1866, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1866;
    } else {
    }
    void* __tmp_1869 = ore_list_new();
    ore_list_push(__tmp_1869, (int64_t)(intptr_t)(ore_str_new("0", 1)));
    ore_list_push(__tmp_1869, (int64_t)(intptr_t)(ore_str_new("int", 3)));
    return __tmp_1869;
}

void* compile_list_method(void* st, void* obj, void* method, void* args, void* kinds) {
    int64_t __tmp_1870 = 0;
    if (ore_str_eq(method, ore_str_new("len", 3))) {
        void* __tmp_1871 = ore_list_new();
        ore_list_push(__tmp_1871, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_len(", 13), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1871, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1871;
    } else {
    }
    int64_t __tmp_1872 = 0;
    if (ore_str_eq(method, ore_str_new("push", 4))) {
        void* ak = str_at(kinds, 0LL);
        int64_t __tmp_1873 = ore_list_get(args, 0LL);
        int8_t __tmp_1874 = ore_list_get_kind(args, 0LL);
        void* i64val = value_to_i64_expr(st, __tmp_1873, ak);
        void* __tmp_1875 = ore_list_new();
        ore_list_push(__tmp_1875, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1875, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1875;
        __tmp_1872 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_push(", 14), obj), ore_str_new(", ", 2)), i64val), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1876 = 0;
    if (ore_str_eq(method, ore_str_new("pop", 3))) {
        void* __tmp_1877 = ore_list_new();
        ore_list_push(__tmp_1877, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_pop(", 13), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1877, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1877;
    } else {
    }
    int64_t __tmp_1878 = 0;
    if (ore_str_eq(method, ore_str_new("clear", 5))) {
        void* __tmp_1879 = ore_list_new();
        ore_list_push(__tmp_1879, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1879, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1879;
        __tmp_1878 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_list_clear(", 15), obj), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1880 = 0;
    if (ore_str_eq(method, ore_str_new("get", 3))) {
        void* __tmp_1881 = ore_list_new();
        int64_t __tmp_1882 = ore_list_get(args, 0LL);
        int8_t __tmp_1883 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1881, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_get(", 13), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1882, __tmp_1883)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1881, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1881;
    } else {
    }
    int64_t __tmp_1884 = 0;
    if (ore_str_eq(method, ore_str_new("set", 3))) {
        void* ak = str_at(kinds, 1LL);
        int64_t __tmp_1885 = ore_list_get(args, 1LL);
        int8_t __tmp_1886 = ore_list_get_kind(args, 1LL);
        void* i64val = value_to_i64_expr(st, __tmp_1885, ak);
        int64_t __tmp_1887 = ore_list_get(args, 0LL);
        int8_t __tmp_1888 = ore_list_get_kind(args, 0LL);
        void* __tmp_1889 = ore_list_new();
        ore_list_push(__tmp_1889, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1889, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1889;
        __tmp_1884 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_set(", 13), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1887, __tmp_1888)), ore_str_new(", ", 2)), i64val), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1890 = 0;
    if (ore_str_eq(method, ore_str_new("contains", 8))) {
        void* __tmp_1891 = ore_list_new();
        int64_t __tmp_1892 = ore_list_get(args, 0LL);
        int8_t __tmp_1893 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1891, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_contains(", 18), obj), ore_str_new(", (int64_t)(", 12)), ore_dynamic_to_str(__tmp_1892, __tmp_1893)), ore_str_new("))", 2))));
        ore_list_push(__tmp_1891, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1891;
    } else {
    }
    int64_t __tmp_1894 = 0;
    if (ore_str_eq(method, ore_str_new("sort", 4))) {
        void* __tmp_1895 = ore_list_new();
        ore_list_push(__tmp_1895, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_sort(", 14), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1895, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1895;
    } else {
    }
    int64_t __tmp_1896 = 0;
    if (ore_str_eq(method, ore_str_new("reverse", 7))) {
        void* __tmp_1897 = ore_list_new();
        ore_list_push(__tmp_1897, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1897, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1897;
        __tmp_1896 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_list_reverse(", 17), obj), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1898 = 0;
    if (ore_str_eq(method, ore_str_new("concat", 6))) {
        void* __tmp_1899 = ore_list_new();
        int64_t __tmp_1900 = ore_list_get(args, 0LL);
        int8_t __tmp_1901 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1899, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_concat(", 16), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1900, __tmp_1901)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1899, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1899;
    } else {
    }
    int64_t __tmp_1902 = 0;
    if (ore_str_eq(method, ore_str_new("join", 4))) {
        void* __tmp_1903 = ore_list_new();
        int64_t __tmp_1904 = ore_list_get(args, 0LL);
        int8_t __tmp_1905 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1903, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_join(", 14), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1904, __tmp_1905)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1903, (int64_t)(intptr_t)(ore_str_new("str", 3)));
        return __tmp_1903;
    } else {
    }
    int64_t __tmp_1906 = 0;
    if (ore_str_eq(method, ore_str_new("sum", 3))) {
        void* __tmp_1907 = ore_list_new();
        ore_list_push(__tmp_1907, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_sum(", 13), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1907, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1907;
    } else {
    }
    int64_t __tmp_1908 = 0;
    if (ore_str_eq(method, ore_str_new("min", 3))) {
        void* __tmp_1909 = ore_list_new();
        ore_list_push(__tmp_1909, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_min(", 13), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1909, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1909;
    } else {
    }
    int64_t __tmp_1910 = 0;
    if (ore_str_eq(method, ore_str_new("max", 3))) {
        void* __tmp_1911 = ore_list_new();
        ore_list_push(__tmp_1911, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_max(", 13), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1911, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1911;
    } else {
    }
    int64_t __tmp_1912 = 0;
    if (ore_str_eq(method, ore_str_new("average", 7))) {
        void* __tmp_1913 = ore_list_new();
        ore_list_push(__tmp_1913, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_average(", 17), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1913, (int64_t)(intptr_t)(ore_str_new("float", 5)));
        return __tmp_1913;
    } else {
    }
    int64_t __tmp_1914 = 0;
    if (ore_str_eq(method, ore_str_new("take", 4))) {
        void* __tmp_1915 = ore_list_new();
        int64_t __tmp_1916 = ore_list_get(args, 0LL);
        int8_t __tmp_1917 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1915, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_take(", 14), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1916, __tmp_1917)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1915, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1915;
    } else {
    }
    int64_t __tmp_1918 = 0;
    if (ore_str_eq(method, ore_str_new("skip", 4))) {
        void* __tmp_1919 = ore_list_new();
        int64_t __tmp_1920 = ore_list_get(args, 0LL);
        int8_t __tmp_1921 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_1919, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_skip(", 14), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1920, __tmp_1921)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1919, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1919;
    } else {
    }
    int64_t __tmp_1922 = 0;
    if (ore_str_eq(method, ore_str_new("slice", 5))) {
        void* __tmp_1923 = ore_list_new();
        int64_t __tmp_1924 = ore_list_get(args, 0LL);
        int8_t __tmp_1925 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1926 = ore_list_get(args, 1LL);
        int8_t __tmp_1927 = ore_list_get_kind(args, 1LL);
        ore_list_push(__tmp_1923, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_slice(", 15), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1924, __tmp_1925)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1926, __tmp_1927)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1923, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1923;
    } else {
    }
    int64_t __tmp_1928 = 0;
    if (ore_str_eq(method, ore_str_new("flatten", 7))) {
        void* __tmp_1929 = ore_list_new();
        ore_list_push(__tmp_1929, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_flatten(", 17), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1929, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1929;
    } else {
    }
    int64_t __tmp_1930 = 0;
    if (ore_str_eq(method, ore_str_new("unique", 6))) {
        void* __tmp_1931 = ore_list_new();
        ore_list_push(__tmp_1931, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_unique(", 16), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1931, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1931;
    } else {
    }
    int64_t __tmp_1932 = 0;
    if (ore_str_eq(method, ore_str_new("enumerate", 9))) {
        void* __tmp_1933 = ore_list_new();
        ore_list_push(__tmp_1933, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_list_enumerate(", 19), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1933, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1933;
    } else {
    }
    int64_t __tmp_1934 = 0;
    if (ore_str_eq(method, ore_str_new("map", 3))) {
        int64_t __tmp_1935 = ore_list_get(args, 0LL);
        int8_t __tmp_1936 = ore_list_get_kind(args, 0LL);
        void* cp = parse_closure_expr(__tmp_1935);
        void* __tmp_1937 = ore_list_new();
        int64_t __tmp_1938 = ore_list_get(cp, 0LL);
        int8_t __tmp_1939 = ore_list_get_kind(cp, 0LL);
        int64_t __tmp_1940 = ore_list_get(cp, 1LL);
        int8_t __tmp_1941 = ore_list_get_kind(cp, 1LL);
        ore_list_push(__tmp_1937, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_map(", 13), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1938, __tmp_1939)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1940, __tmp_1941)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1937, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1937;
    } else {
    }
    int64_t __tmp_1942 = 0;
    if (ore_str_eq(method, ore_str_new("filter", 6))) {
        int64_t __tmp_1943 = ore_list_get(args, 0LL);
        int8_t __tmp_1944 = ore_list_get_kind(args, 0LL);
        void* cp = parse_closure_expr(__tmp_1943);
        void* __tmp_1945 = ore_list_new();
        int64_t __tmp_1946 = ore_list_get(cp, 0LL);
        int8_t __tmp_1947 = ore_list_get_kind(cp, 0LL);
        int64_t __tmp_1948 = ore_list_get(cp, 1LL);
        int8_t __tmp_1949 = ore_list_get_kind(cp, 1LL);
        ore_list_push(__tmp_1945, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_filter(", 16), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1946, __tmp_1947)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1948, __tmp_1949)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1945, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_1945;
    } else {
    }
    int64_t __tmp_1950 = 0;
    if (ore_str_eq(method, ore_str_new("each", 4))) {
        int64_t __tmp_1951 = ore_list_get(args, 0LL);
        int8_t __tmp_1952 = ore_list_get_kind(args, 0LL);
        void* cp = parse_closure_expr(__tmp_1951);
        int64_t __tmp_1953 = ore_list_get(cp, 0LL);
        int8_t __tmp_1954 = ore_list_get_kind(cp, 0LL);
        int64_t __tmp_1955 = ore_list_get(cp, 1LL);
        int8_t __tmp_1956 = ore_list_get_kind(cp, 1LL);
        void* __tmp_1957 = ore_list_new();
        ore_list_push(__tmp_1957, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_1957, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_1957;
        __tmp_1950 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_each(", 14), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1953, __tmp_1954)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1955, __tmp_1956)), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_1958 = 0;
    if (ore_str_eq(method, ore_str_new("fold", 4))) {
        int64_t __tmp_1959 = ore_list_get(args, 1LL);
        int8_t __tmp_1960 = ore_list_get_kind(args, 1LL);
        void* cp = parse_closure_expr(__tmp_1959);
        void* __tmp_1961 = ore_list_new();
        int64_t __tmp_1962 = ore_list_get(args, 0LL);
        int8_t __tmp_1963 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1964 = ore_list_get(cp, 0LL);
        int8_t __tmp_1965 = ore_list_get_kind(cp, 0LL);
        int64_t __tmp_1966 = ore_list_get(cp, 1LL);
        int8_t __tmp_1967 = ore_list_get_kind(cp, 1LL);
        ore_list_push(__tmp_1961, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_fold(", 14), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1962, __tmp_1963)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1964, __tmp_1965)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1966, __tmp_1967)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1961, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1961;
    } else {
    }
    int64_t __tmp_1968 = 0;
    if (ore_str_eq(method, ore_str_new("reduce", 6))) {
        int64_t __tmp_1969 = ore_list_get(args, 0LL);
        int8_t __tmp_1970 = ore_list_get_kind(args, 0LL);
        void* cp = parse_closure_expr(__tmp_1969);
        void* __tmp_1971 = ore_list_new();
        int64_t __tmp_1972 = ore_list_get(cp, 0LL);
        int8_t __tmp_1973 = ore_list_get_kind(cp, 0LL);
        int64_t __tmp_1974 = ore_list_get(cp, 1LL);
        int8_t __tmp_1975 = ore_list_get_kind(cp, 1LL);
        ore_list_push(__tmp_1971, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_reduce1(", 17), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1972, __tmp_1973)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1974, __tmp_1975)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1971, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1971;
    } else {
    }
    int64_t __tmp_1976 = 0;
    if (ore_str_eq(method, ore_str_new("any", 3))) {
        int64_t __tmp_1977 = ore_list_get(args, 0LL);
        int8_t __tmp_1978 = ore_list_get_kind(args, 0LL);
        void* cp = parse_closure_expr(__tmp_1977);
        void* __tmp_1979 = ore_list_new();
        int64_t __tmp_1980 = ore_list_get(cp, 0LL);
        int8_t __tmp_1981 = ore_list_get_kind(cp, 0LL);
        int64_t __tmp_1982 = ore_list_get(cp, 1LL);
        int8_t __tmp_1983 = ore_list_get_kind(cp, 1LL);
        ore_list_push(__tmp_1979, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_any(", 13), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1980, __tmp_1981)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1982, __tmp_1983)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1979, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1979;
    } else {
    }
    int64_t __tmp_1984 = 0;
    if (ore_str_eq(method, ore_str_new("all", 3))) {
        int64_t __tmp_1985 = ore_list_get(args, 0LL);
        int8_t __tmp_1986 = ore_list_get_kind(args, 0LL);
        void* cp = parse_closure_expr(__tmp_1985);
        void* __tmp_1987 = ore_list_new();
        int64_t __tmp_1988 = ore_list_get(cp, 0LL);
        int8_t __tmp_1989 = ore_list_get_kind(cp, 0LL);
        int64_t __tmp_1990 = ore_list_get(cp, 1LL);
        int8_t __tmp_1991 = ore_list_get_kind(cp, 1LL);
        ore_list_push(__tmp_1987, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_list_all(", 13), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1988, __tmp_1989)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1990, __tmp_1991)), ore_str_new(")", 1))));
        ore_list_push(__tmp_1987, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_1987;
    } else {
    }
    void* __tmp_1992 = ore_list_new();
    ore_list_push(__tmp_1992, (int64_t)(intptr_t)(ore_str_new("0", 1)));
    ore_list_push(__tmp_1992, (int64_t)(intptr_t)(ore_str_new("int", 3)));
    return __tmp_1992;
}

void* compile_map_method(void* st, void* obj, void* method, void* args, void* kinds) {
    int64_t __tmp_1993 = 0;
    if (ore_str_eq(method, ore_str_new("len", 3))) {
        void* __tmp_1994 = ore_list_new();
        ore_list_push(__tmp_1994, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_map_len(", 12), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_1994, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_1994;
    } else {
    }
    int64_t __tmp_1995 = 0;
    if (ore_str_eq(method, ore_str_new("set", 3))) {
        int64_t __tmp_1996 = ore_list_get(args, 0LL);
        int8_t __tmp_1997 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_1998 = ore_list_get(args, 1LL);
        int8_t __tmp_1999 = ore_list_get_kind(args, 1LL);
        void* __tmp_2000 = ore_list_new();
        ore_list_push(__tmp_2000, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_2000, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_2000;
        __tmp_1995 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_map_set(", 12), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_1996, __tmp_1997)), ore_str_new(", (int64_t)(", 12)), ore_dynamic_to_str(__tmp_1998, __tmp_1999)), ore_str_new("));", 3))));
    } else {
    }
    int64_t __tmp_2001 = 0;
    if (ore_str_eq(method, ore_str_new("get", 3))) {
        void* __tmp_2002 = ore_list_new();
        int64_t __tmp_2003 = ore_list_get(args, 0LL);
        int8_t __tmp_2004 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_2002, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_map_get(", 12), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_2003, __tmp_2004)), ore_str_new(")", 1))));
        ore_list_push(__tmp_2002, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_2002;
    } else {
    }
    int64_t __tmp_2005 = 0;
    if (ore_str_eq(method, ore_str_new("get_or", 6))) {
        void* __tmp_2006 = ore_list_new();
        int64_t __tmp_2007 = ore_list_get(args, 0LL);
        int8_t __tmp_2008 = ore_list_get_kind(args, 0LL);
        int64_t __tmp_2009 = ore_list_get(args, 1LL);
        int8_t __tmp_2010 = ore_list_get_kind(args, 1LL);
        ore_list_push(__tmp_2006, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_map_get_or(", 15), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_2007, __tmp_2008)), ore_str_new(", (int64_t)(", 12)), ore_dynamic_to_str(__tmp_2009, __tmp_2010)), ore_str_new("))", 2))));
        ore_list_push(__tmp_2006, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_2006;
    } else {
    }
    int64_t __tmp_2011 = 0;
    if (ore_str_eq(method, ore_str_new("contains", 8))) {
        void* __tmp_2012 = ore_list_new();
        int64_t __tmp_2013 = ore_list_get(args, 0LL);
        int8_t __tmp_2014 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_2012, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_map_contains(", 17), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_2013, __tmp_2014)), ore_str_new(")", 1))));
        ore_list_push(__tmp_2012, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_2012;
    } else {
    }
    int64_t __tmp_2015 = 0;
    if (ore_str_eq(method, ore_str_new("remove", 6))) {
        void* __tmp_2016 = ore_list_new();
        int64_t __tmp_2017 = ore_list_get(args, 0LL);
        int8_t __tmp_2018 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_2016, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_map_remove(", 15), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_2017, __tmp_2018)), ore_str_new(")", 1))));
        ore_list_push(__tmp_2016, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_2016;
    } else {
    }
    int64_t __tmp_2019 = 0;
    if (ore_str_eq(method, ore_str_new("keys", 4))) {
        void* __tmp_2020 = ore_list_new();
        ore_list_push(__tmp_2020, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_map_keys(", 13), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_2020, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_2020;
    } else {
    }
    int64_t __tmp_2021 = 0;
    if (ore_str_eq(method, ore_str_new("values", 6))) {
        void* __tmp_2022 = ore_list_new();
        ore_list_push(__tmp_2022, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_map_values(", 15), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_2022, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_2022;
    } else {
    }
    int64_t __tmp_2023 = 0;
    if (ore_str_eq(method, ore_str_new("entries", 7))) {
        void* __tmp_2024 = ore_list_new();
        ore_list_push(__tmp_2024, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("ore_map_entries(", 16), obj), ore_str_new(")", 1))));
        ore_list_push(__tmp_2024, (int64_t)(intptr_t)(ore_str_new("list", 4)));
        return __tmp_2024;
    } else {
    }
    int64_t __tmp_2025 = 0;
    if (ore_str_eq(method, ore_str_new("merge", 5))) {
        void* __tmp_2026 = ore_list_new();
        int64_t __tmp_2027 = ore_list_get(args, 0LL);
        int8_t __tmp_2028 = ore_list_get_kind(args, 0LL);
        ore_list_push(__tmp_2026, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_map_merge(", 14), obj), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_2027, __tmp_2028)), ore_str_new(")", 1))));
        ore_list_push(__tmp_2026, (int64_t)(intptr_t)(ore_str_new("map", 3)));
        return __tmp_2026;
    } else {
    }
    int64_t __tmp_2029 = 0;
    if (ore_str_eq(method, ore_str_new("clear", 5))) {
        void* __tmp_2030 = ore_list_new();
        ore_list_push(__tmp_2030, (int64_t)(intptr_t)(ore_str_new("0", 1)));
        ore_list_push(__tmp_2030, (int64_t)(intptr_t)(ore_str_new("void", 4)));
        return __tmp_2030;
        __tmp_2029 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("ore_map_clear(", 14), obj), ore_str_new(");", 2))));
    } else {
    }
    void* __tmp_2031 = ore_list_new();
    ore_list_push(__tmp_2031, (int64_t)(intptr_t)(ore_str_new("0", 1)));
    ore_list_push(__tmp_2031, (int64_t)(intptr_t)(ore_str_new("int", 3)));
    return __tmp_2031;
}

void* compile_option_method(void* st, void* obj, void* kind, void* method, void* args, void* kinds) {
    int64_t __tmp_2032 = 0;
    if (ore_str_eq(method, ore_str_new("unwrap", 6))) {
        void* __tmp_2033 = ore_list_new();
        ore_list_push(__tmp_2033, (int64_t)(intptr_t)(ore_str_concat(obj, ore_str_new(".value", 6))));
        ore_list_push(__tmp_2033, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_2033;
    } else {
    }
    int64_t __tmp_2034 = 0;
    if (ore_str_eq(method, ore_str_new("unwrap_or", 9))) {
        void* t = cg_tmp(st);
        int64_t __tmp_2035 = ore_list_get(args, 0LL);
        int8_t __tmp_2036 = ore_list_get_kind(args, 0LL);
        void* __tmp_2037 = ore_list_new();
        ore_list_push(__tmp_2037, (int64_t)(intptr_t)(t));
        ore_list_push(__tmp_2037, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        return __tmp_2037;
        __tmp_2034 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("int64_t ", 8), t), ore_str_new(" = (", 4)), obj), ore_str_new(".tag != 0) ? ", 13)), obj), ore_str_new(".value : (int64_t)(", 19)), ore_dynamic_to_str(__tmp_2035, __tmp_2036)), ore_str_new(");", 2))));
    } else {
    }
    int64_t __tmp_2038 = 0;
    if ((96070758374224 || 96070758377808)) {
        void* __tmp_2039 = ore_list_new();
        ore_list_push(__tmp_2039, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("(int8_t)(", 9), obj), ore_str_new(".tag != 0)", 10))));
        ore_list_push(__tmp_2039, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_2039;
    } else {
    }
    int64_t __tmp_2040 = 0;
    if ((96070758396080 || 96070758399856)) {
        void* __tmp_2041 = ore_list_new();
        ore_list_push(__tmp_2041, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_new("(int8_t)(", 9), obj), ore_str_new(".tag == 0)", 10))));
        ore_list_push(__tmp_2041, (int64_t)(intptr_t)(ore_str_new("bool", 4)));
        return __tmp_2041;
    } else {
    }
    void* __tmp_2042 = ore_list_new();
    ore_list_push(__tmp_2042, (int64_t)(intptr_t)(ore_str_new("0", 1)));
    ore_list_push(__tmp_2042, (int64_t)(intptr_t)(ore_str_new("int", 3)));
    return __tmp_2042;
}

void* compile_block_expr(void* st, void* exprs, void* stmts, struct ore_rec_Block block) {
    return compile_block(st, exprs, stmts, block);
}

void* match_assign_expr(void* t, void* kind, void* val) {
    int64_t __tmp_2043 = 0;
    if ((96070758464416 || 96070758468128)) {
        return ore_str_concat(ore_str_concat(ore_str_concat(t, ore_str_new(" = ", 3)), val), ore_str_new(";", 1));
    } else {
    }
    void* c = kind_to_c_type(kind);
    return ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(t, ore_str_new(" = (", 4)), c), ore_str_new(")(", 2)), val), ore_str_new(");", 2));
}

void* match_default_init(void* kind) {
    int64_t __tmp_2044 = 0;
    if (ore_str_eq(kind, ore_str_new("void", 4))) {
        return ore_str_new(" = 0", 4);
    } else {
    }
    int64_t __tmp_2045 = 0;
    if ((96070758510976 || 96070758514688)) {
        return ore_str_new(" = {0}", 6);
    } else {
    }
    int64_t __tmp_2046 = 0;
    if (ore_str_eq(kind, ore_str_new("float", 5))) {
        return ore_str_new(" = 0.0", 6);
    } else {
    }
    return ore_str_new(" = 0", 4);
}

void* match_temp_c_type(void* kind) {
    int64_t __tmp_2047 = 0;
    if (ore_str_eq(kind, ore_str_new("void", 4))) {
        return ore_str_new("int64_t", 7);
    } else {
    }
    return kind_to_c_type(kind);
}

void* compile_match(void* st, void* exprs, void* stmts, int64_t subject, void* arms) {
    void* subj = compile_expr(st, exprs, stmts, subject);
    void* t = cg_tmp(st);
    int64_t decl_pos = ore_list_len(cg_lines(st));
    void* decl_indent = indent_str(cg_indent(st));
    void* result_kind = ore_str_new("int", 3);
    int8_t first_arm_compiled = ((int8_t)0);
    int8_t first = ((int8_t)1);
    for (int64_t i = 0LL; i < ore_list_len(arms); i++) {
        struct ore_rec_MatchArm arm = get_match_arm(arms, i);
        struct ore_enum_Pattern pat = arm.pattern;
        int64_t body_id = arm.body;
        int64_t __tmp_2048 = 0;
        if (pat.tag == 1) {
            int64_t __tmp_2049 = 0;
            if (first) {
                __tmp_2049 = (int64_t)(emit(st, ore_str_new("{", 1)));
            } else {
                __tmp_2049 = (int64_t)(emit(st, ore_str_new("else {", 6)));
            }
            void* br = compile_expr(st, exprs, stmts, body_id);
            int64_t __tmp_2050 = ore_list_get(br, 1LL);
            int8_t __tmp_2051 = ore_list_get_kind(br, 1LL);
            int64_t __tmp_2052 = 0;
            if ((96070758666944 && 96070758672496)) {
                int64_t __tmp_2053 = ore_list_get(br, 1LL);
                int8_t __tmp_2054 = ore_list_get_kind(br, 1LL);
                result_kind = __tmp_2053;
                int64_t __tmp_2055 = 0;
                if ((!ore_str_eq((void*)(intptr_t)(96070676630064), ore_str_new("void", 4)))) {
                    void* ct = kind_to_c_type(result_kind);
                    void* init = match_default_init(result_kind);
                    ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                } else {
                }
                first_arm_compiled = ((int8_t)1);
                __tmp_2052 = (int64_t)(__tmp_2055);
            } else {
            }
            int64_t __tmp_2056 = ore_list_get(br, 0LL);
            int8_t __tmp_2057 = ore_list_get_kind(br, 0LL);
            int64_t __tmp_2058 = 0;
            if ((!ore_str_eq((void*)(intptr_t)(96070758721648), ore_str_new("", 0)))) {
                int64_t __tmp_2059 = ore_list_get(br, 0LL);
                int8_t __tmp_2060 = ore_list_get_kind(br, 0LL);
                __tmp_2058 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2059)));
            } else {
            }
            first = ((int8_t)0);
            __tmp_2048 = (int64_t)(emit(st, ore_str_new("}", 1)));
        }
        else if (pat.tag == 2) {
            int64_t value = pat.data[0];
            void* kw = ore_str_new("if", 2);
            int64_t __tmp_2061 = 0;
            if ((!(first))) {
                kw = ore_str_new("else if", 7);
            } else {
            }
            int64_t __tmp_2062 = ore_list_get(subj, 0LL);
            int8_t __tmp_2063 = ore_list_get_kind(subj, 0LL);
            void* br = compile_expr(st, exprs, stmts, body_id);
            int64_t __tmp_2064 = ore_list_get(br, 1LL);
            int8_t __tmp_2065 = ore_list_get_kind(br, 1LL);
            int64_t __tmp_2066 = 0;
            if ((96070758807120 && 96070758812672)) {
                int64_t __tmp_2067 = ore_list_get(br, 1LL);
                int8_t __tmp_2068 = ore_list_get_kind(br, 1LL);
                result_kind = __tmp_2067;
                int64_t __tmp_2069 = 0;
                if ((!ore_str_eq((void*)(intptr_t)(96070676735808), ore_str_new("void", 4)))) {
                    void* ct = kind_to_c_type(result_kind);
                    void* init = match_default_init(result_kind);
                    ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                } else {
                }
                first_arm_compiled = ((int8_t)1);
                __tmp_2066 = (int64_t)(__tmp_2069);
            } else {
            }
            int64_t __tmp_2070 = ore_list_get(br, 0LL);
            int8_t __tmp_2071 = ore_list_get_kind(br, 0LL);
            int64_t __tmp_2072 = 0;
            if ((!ore_str_eq((void*)(intptr_t)(96070758861584), ore_str_new("", 0)))) {
                int64_t __tmp_2073 = ore_list_get(br, 0LL);
                int8_t __tmp_2074 = ore_list_get_kind(br, 0LL);
                __tmp_2072 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2073)));
            } else {
            }
            first = ((int8_t)0);
            __tmp_2048 = (int64_t)(emit(st, ore_str_new("}", 1)));
        }
        else if (pat.tag == 4) {
            int64_t value = pat.data[0];
            void* kw = ore_str_new("if", 2);
            int64_t __tmp_2075 = 0;
            if ((!(first))) {
                kw = ore_str_new("else if", 7);
            } else {
            }
            int64_t bv = 0LL;
            int64_t __tmp_2076 = 0;
            if (value) {
                bv = 1LL;
            } else {
            }
            int64_t __tmp_2077 = ore_list_get(subj, 0LL);
            int8_t __tmp_2078 = ore_list_get_kind(subj, 0LL);
            void* br = compile_expr(st, exprs, stmts, body_id);
            int64_t __tmp_2079 = ore_list_get(br, 1LL);
            int8_t __tmp_2080 = ore_list_get_kind(br, 1LL);
            int64_t __tmp_2081 = 0;
            if ((96070758952624 && 96070758958176)) {
                int64_t __tmp_2082 = ore_list_get(br, 1LL);
                int8_t __tmp_2083 = ore_list_get_kind(br, 1LL);
                result_kind = __tmp_2082;
                int64_t __tmp_2084 = 0;
                if ((!ore_str_eq((void*)(intptr_t)(96070676848752), ore_str_new("void", 4)))) {
                    void* ct = kind_to_c_type(result_kind);
                    void* init = match_default_init(result_kind);
                    ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                } else {
                }
                first_arm_compiled = ((int8_t)1);
                __tmp_2081 = (int64_t)(__tmp_2084);
            } else {
            }
            int64_t __tmp_2085 = ore_list_get(br, 0LL);
            int8_t __tmp_2086 = ore_list_get_kind(br, 0LL);
            int64_t __tmp_2087 = 0;
            if ((!ore_str_eq((void*)(intptr_t)(96070759007088), ore_str_new("", 0)))) {
                int64_t __tmp_2088 = ore_list_get(br, 0LL);
                int8_t __tmp_2089 = ore_list_get_kind(br, 0LL);
                __tmp_2087 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2088)));
            } else {
            }
            first = ((int8_t)0);
            __tmp_2048 = (int64_t)(emit(st, ore_str_new("}", 1)));
        }
        else if (pat.tag == 5) {
            int64_t value = pat.data[0];
            void* kw = ore_str_new("if", 2);
            int64_t __tmp_2090 = 0;
            if ((!(first))) {
                kw = ore_str_new("else if", 7);
            } else {
            }
            void* slit = compile_string_lit(st, value);
            int64_t __tmp_2091 = ore_list_get(subj, 0LL);
            int8_t __tmp_2092 = ore_list_get_kind(subj, 0LL);
            int64_t __tmp_2093 = ore_list_get(slit, 0LL);
            int8_t __tmp_2094 = ore_list_get_kind(slit, 0LL);
            void* br = compile_expr(st, exprs, stmts, body_id);
            int64_t __tmp_2095 = ore_list_get(br, 1LL);
            int8_t __tmp_2096 = ore_list_get_kind(br, 1LL);
            int64_t __tmp_2097 = 0;
            if ((96070759101616 && 96070759107168)) {
                int64_t __tmp_2098 = ore_list_get(br, 1LL);
                int8_t __tmp_2099 = ore_list_get_kind(br, 1LL);
                result_kind = __tmp_2098;
                int64_t __tmp_2100 = 0;
                if ((!ore_str_eq((void*)(intptr_t)(96070676962160), ore_str_new("void", 4)))) {
                    void* ct = kind_to_c_type(result_kind);
                    void* init = match_default_init(result_kind);
                    ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                } else {
                }
                first_arm_compiled = ((int8_t)1);
                __tmp_2097 = (int64_t)(__tmp_2100);
            } else {
            }
            int64_t __tmp_2101 = ore_list_get(br, 0LL);
            int8_t __tmp_2102 = ore_list_get_kind(br, 0LL);
            int64_t __tmp_2103 = 0;
            if ((!ore_str_eq((void*)(intptr_t)(96070759156320), ore_str_new("", 0)))) {
                int64_t __tmp_2104 = ore_list_get(br, 0LL);
                int8_t __tmp_2105 = ore_list_get_kind(br, 0LL);
                __tmp_2103 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2104)));
            } else {
            }
            first = ((int8_t)0);
            __tmp_2048 = (int64_t)(emit(st, ore_str_new("}", 1)));
        }
        else if (pat.tag == 0) {
            int64_t name = pat.data[0];
            int64_t bindings = pat.data[1];
            int64_t __tmp_2106 = ore_list_get(subj, 1LL);
            int8_t __tmp_2107 = ore_list_get_kind(subj, 1LL);
            int64_t __tmp_2108 = ore_list_get(subj, 1LL);
            int8_t __tmp_2109 = ore_list_get_kind(subj, 1LL);
            int64_t __tmp_2110 = 0;
            if ((96070759198928 || 96070759204960)) {
                int64_t vtag = 0LL;
                int64_t __tmp_2111 = 0;
                if ((96070759211904 || 96070759215584)) {
                    vtag = 1LL;
                } else {
                }
                void* kw = ore_str_new("if", 2);
                int64_t __tmp_2112 = 0;
                if ((!(first))) {
                    kw = ore_str_new("else if", 7);
                } else {
                }
                int64_t __tmp_2113 = ore_list_get(subj, 0LL);
                int8_t __tmp_2114 = ore_list_get_kind(subj, 0LL);
                int64_t __tmp_2115 = 0;
                if ((96070759258096 > 96070759258320)) {
                    int64_t bname = ((bindings)[0LL]);
                    void* mn = mangle_var(bname);
                    int64_t __tmp_2116 = ore_list_get(subj, 0LL);
                    int8_t __tmp_2117 = ore_list_get_kind(subj, 0LL);
                    __tmp_2115 = (int64_t)(cg_set_var(st, bname, ore_str_new("int", 3)));
                } else {
                }
                void* br = compile_expr(st, exprs, stmts, body_id);
                int64_t __tmp_2118 = ore_list_get(br, 1LL);
                int8_t __tmp_2119 = ore_list_get_kind(br, 1LL);
                int64_t __tmp_2120 = 0;
                if ((96070759312240 && 96070759317792)) {
                    int64_t __tmp_2121 = ore_list_get(br, 1LL);
                    int8_t __tmp_2122 = ore_list_get_kind(br, 1LL);
                    result_kind = __tmp_2121;
                    int64_t __tmp_2123 = 0;
                    if ((!ore_str_eq((void*)(intptr_t)(96070677124496), ore_str_new("void", 4)))) {
                        void* ct = kind_to_c_type(result_kind);
                        void* init = match_default_init(result_kind);
                        ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                    } else {
                    }
                    first_arm_compiled = ((int8_t)1);
                    __tmp_2120 = (int64_t)(__tmp_2123);
                } else {
                }
                int64_t __tmp_2124 = ore_list_get(br, 0LL);
                int8_t __tmp_2125 = ore_list_get_kind(br, 0LL);
                int64_t __tmp_2126 = 0;
                if ((!ore_str_eq((void*)(intptr_t)(96070759370016), ore_str_new("", 0)))) {
                    int64_t __tmp_2127 = ore_list_get(br, 0LL);
                    int8_t __tmp_2128 = ore_list_get_kind(br, 0LL);
                    __tmp_2126 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2127)));
                } else {
                }
                first = ((int8_t)0);
                __tmp_2110 = (int64_t)(emit(st, ore_str_new("}", 1)));
            } else {
                int64_t __tmp_2129 = ore_list_get(subj, 1LL);
                int8_t __tmp_2130 = ore_list_get_kind(subj, 1LL);
                int64_t __tmp_2131 = 0;
                if ((96070759410336 && 96070759416320)) {
                    int64_t vtag = 1LL;
                    int64_t __tmp_2132 = 0;
                    if (ore_str_eq((void*)(intptr_t)(96070677198192), ore_str_new("Err", 3))) {
                        vtag = 0LL;
                    } else {
                    }
                    void* kw = ore_str_new("if", 2);
                    int64_t __tmp_2133 = 0;
                    if ((!(first))) {
                        kw = ore_str_new("else if", 7);
                    } else {
                    }
                    int64_t __tmp_2134 = ore_list_get(subj, 0LL);
                    int8_t __tmp_2135 = ore_list_get_kind(subj, 0LL);
                    int64_t __tmp_2136 = 0;
                    if ((96070759468704 > 96070759468928)) {
                        int64_t bname = ((bindings)[0LL]);
                        void* mn = mangle_var(bname);
                        int64_t __tmp_2137 = ore_list_get(subj, 0LL);
                        int8_t __tmp_2138 = ore_list_get_kind(subj, 0LL);
                        __tmp_2136 = (int64_t)(cg_set_var(st, bname, ore_str_new("int", 3)));
                    } else {
                    }
                    void* br = compile_expr(st, exprs, stmts, body_id);
                    int64_t __tmp_2139 = 0;
                    if ((!(first_arm_compiled))) {
                        int64_t __tmp_2140 = ore_list_get(br, 1LL);
                        int8_t __tmp_2141 = ore_list_get_kind(br, 1LL);
                        result_kind = __tmp_2140;
                        int64_t __tmp_2142 = 0;
                        if ((!ore_str_eq((void*)(intptr_t)(96070677278592), ore_str_new("void", 4)))) {
                            void* ct = kind_to_c_type(result_kind);
                            void* init = match_default_init(result_kind);
                            ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                        } else {
                        }
                        first_arm_compiled = ((int8_t)1);
                        __tmp_2139 = (int64_t)(__tmp_2142);
                    } else {
                    }
                    int64_t __tmp_2143 = ore_list_get(br, 0LL);
                    int8_t __tmp_2144 = ore_list_get_kind(br, 0LL);
                    int64_t __tmp_2145 = 0;
                    if ((!ore_str_eq((void*)(intptr_t)(96070759578592), ore_str_new("", 0)))) {
                        int64_t __tmp_2146 = ore_list_get(br, 0LL);
                        int8_t __tmp_2147 = ore_list_get_kind(br, 0LL);
                        __tmp_2145 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2146)));
                    } else {
                    }
                    first = ((int8_t)0);
                    __tmp_2131 = (int64_t)(emit(st, ore_str_new("}", 1)));
                } else {
                    int64_t __tmp_2148 = ore_list_get(subj, 1LL);
                    int8_t __tmp_2149 = ore_list_get_kind(subj, 1LL);
                    int64_t __tmp_2150 = 0;
                    if ((96070759622480 && 96070759629152)) {
                        int64_t vtag = 1LL;
                        int64_t __tmp_2151 = 0;
                        if (ore_str_eq((void*)(intptr_t)(96070677352576), ore_str_new("None", 4))) {
                            vtag = 0LL;
                        } else {
                        }
                        void* kw = ore_str_new("if", 2);
                        int64_t __tmp_2152 = 0;
                        if ((!(first))) {
                            kw = ore_str_new("else if", 7);
                        } else {
                        }
                        int64_t __tmp_2153 = ore_list_get(subj, 0LL);
                        int8_t __tmp_2154 = ore_list_get_kind(subj, 0LL);
                        int64_t __tmp_2155 = 0;
                        if ((96070759685056 && 96070759688304)) {
                            int64_t bname = ((bindings)[0LL]);
                            void* mn = mangle_var(bname);
                            int64_t __tmp_2156 = ore_list_get(subj, 0LL);
                            int8_t __tmp_2157 = ore_list_get_kind(subj, 0LL);
                            __tmp_2155 = (int64_t)(cg_set_var(st, bname, ore_str_new("int", 3)));
                        } else {
                        }
                        void* br = compile_expr(st, exprs, stmts, body_id);
                        int64_t __tmp_2158 = 0;
                        if ((!(first_arm_compiled))) {
                            int64_t __tmp_2159 = ore_list_get(br, 1LL);
                            int8_t __tmp_2160 = ore_list_get_kind(br, 1LL);
                            result_kind = __tmp_2159;
                            int64_t __tmp_2161 = 0;
                            if ((!ore_str_eq((void*)(intptr_t)(96070677436880), ore_str_new("void", 4)))) {
                                void* ct = kind_to_c_type(result_kind);
                                void* init = match_default_init(result_kind);
                                ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                            } else {
                            }
                            first_arm_compiled = ((int8_t)1);
                            __tmp_2158 = (int64_t)(__tmp_2161);
                        } else {
                        }
                        int64_t __tmp_2162 = ore_list_get(br, 0LL);
                        int8_t __tmp_2163 = ore_list_get_kind(br, 0LL);
                        int64_t __tmp_2164 = 0;
                        if ((!ore_str_eq((void*)(intptr_t)(96070759800352), ore_str_new("", 0)))) {
                            int64_t __tmp_2165 = ore_list_get(br, 0LL);
                            int8_t __tmp_2166 = ore_list_get_kind(br, 0LL);
                            __tmp_2164 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2165)));
                        } else {
                        }
                        first = ((int8_t)0);
                        __tmp_2150 = (int64_t)(emit(st, ore_str_new("}", 1)));
                    } else {
                        void* en = cg_variant_enum(st, name);
                        int64_t __tmp_2167 = 0;
                        if ((!ore_str_eq(en, ore_str_new("", 0)))) {
                            int64_t vtag = find_variant_tag(st, en, name);
                            void* kw = ore_str_new("if", 2);
                            int64_t __tmp_2168 = 0;
                            if ((!(first))) {
                                kw = ore_str_new("else if", 7);
                            } else {
                            }
                            int64_t __tmp_2169 = ore_list_get(subj, 0LL);
                            int8_t __tmp_2170 = ore_list_get_kind(subj, 0LL);
                            for (int64_t j = 0LL; j < bindings.len(); j++) {
                                int64_t bname = ((bindings)[j]);
                                void* mn = mangle_var(bname);
                                int64_t __tmp_2171 = ore_list_get(subj, 0LL);
                                int8_t __tmp_2172 = ore_list_get_kind(subj, 0LL);
                                cont_241: ;
                            }
                            brk_240: ;
                            void* br = compile_expr(st, exprs, stmts, body_id);
                            int64_t __tmp_2173 = ore_list_get(br, 1LL);
                            int8_t __tmp_2174 = ore_list_get_kind(br, 1LL);
                            int64_t __tmp_2175 = 0;
                            if ((96070759962784 && 96070759969360)) {
                                int64_t __tmp_2176 = ore_list_get(br, 1LL);
                                int8_t __tmp_2177 = ore_list_get_kind(br, 1LL);
                                result_kind = __tmp_2176;
                                void* ct = kind_to_c_type(result_kind);
                                void* init = match_default_init(result_kind);
                                ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                                first_arm_compiled = ((int8_t)1);
                            } else {
                            }
                            int64_t __tmp_2178 = ore_list_get(br, 0LL);
                            int8_t __tmp_2179 = ore_list_get_kind(br, 0LL);
                            int64_t __tmp_2180 = 0;
                            if ((!ore_str_eq((void*)(intptr_t)(96070760017904), ore_str_new("", 0)))) {
                                int64_t __tmp_2181 = ore_list_get(br, 0LL);
                                int8_t __tmp_2182 = ore_list_get_kind(br, 0LL);
                                __tmp_2180 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2181)));
                            } else {
                            }
                            first = ((int8_t)0);
                            __tmp_2167 = (int64_t)(emit(st, ore_str_new("}", 1)));
                        } else {
                            void* kw = ore_str_new("if", 2);
                            int64_t __tmp_2183 = 0;
                            if ((!(first))) {
                                kw = ore_str_new("else if", 7);
                            } else {
                            }
                            int64_t __tmp_2184 = 0;
                            if ((!(first))) {
                                __tmp_2184 = (int64_t)(emit(st, ore_str_new("else {", 6)));
                            } else {
                                __tmp_2184 = (int64_t)(emit(st, ore_str_new("{", 1)));
                            }
                            void* mn = mangle_var(name);
                            int64_t __tmp_2185 = ore_list_get(subj, 0LL);
                            int8_t __tmp_2186 = ore_list_get_kind(subj, 0LL);
                            int64_t __tmp_2187 = ore_list_get(subj, 1LL);
                            int8_t __tmp_2188 = ore_list_get_kind(subj, 1LL);
                            void* br = compile_expr(st, exprs, stmts, body_id);
                            int64_t __tmp_2189 = ore_list_get(br, 1LL);
                            int8_t __tmp_2190 = ore_list_get_kind(br, 1LL);
                            int64_t __tmp_2191 = 0;
                            if ((96070760144000 && 96070760150576)) {
                                int64_t __tmp_2192 = ore_list_get(br, 1LL);
                                int8_t __tmp_2193 = ore_list_get_kind(br, 1LL);
                                result_kind = __tmp_2192;
                                void* ct = kind_to_c_type(result_kind);
                                void* init = match_default_init(result_kind);
                                ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                                first_arm_compiled = ((int8_t)1);
                            } else {
                            }
                            int64_t __tmp_2194 = ore_list_get(br, 0LL);
                            int8_t __tmp_2195 = ore_list_get_kind(br, 0LL);
                            int64_t __tmp_2196 = 0;
                            if ((!ore_str_eq((void*)(intptr_t)(96070760199120), ore_str_new("", 0)))) {
                                int64_t __tmp_2197 = ore_list_get(br, 0LL);
                                int8_t __tmp_2198 = ore_list_get_kind(br, 0LL);
                                __tmp_2196 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2197)));
                            } else {
                            }
                            first = ((int8_t)0);
                            __tmp_2167 = (int64_t)(emit(st, ore_str_new("}", 1)));
                        }
                        __tmp_2150 = (int64_t)(__tmp_2167);
                    }
                    __tmp_2131 = (int64_t)(__tmp_2150);
                }
                __tmp_2110 = (int64_t)(__tmp_2131);
            }
            __tmp_2048 = (int64_t)(__tmp_2110);
        }
        else if (pat.tag == 3) {
            int64_t value = pat.data[0];
            void* kw = ore_str_new("if", 2);
            int64_t __tmp_2199 = 0;
            if ((!(first))) {
                kw = ore_str_new("else if", 7);
            } else {
            }
            int64_t __tmp_2200 = ore_list_get(subj, 0LL);
            int8_t __tmp_2201 = ore_list_get_kind(subj, 0LL);
            void* br = compile_expr(st, exprs, stmts, body_id);
            int64_t __tmp_2202 = ore_list_get(br, 1LL);
            int8_t __tmp_2203 = ore_list_get_kind(br, 1LL);
            int64_t __tmp_2204 = 0;
            if ((96070760294496 && 96070760300048)) {
                int64_t __tmp_2205 = ore_list_get(br, 1LL);
                int8_t __tmp_2206 = ore_list_get_kind(br, 1LL);
                result_kind = __tmp_2205;
                int64_t __tmp_2207 = 0;
                if ((!ore_str_eq((void*)(intptr_t)(96070677836016), ore_str_new("void", 4)))) {
                    void* ct = kind_to_c_type(result_kind);
                    void* init = match_default_init(result_kind);
                    ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                } else {
                }
                first_arm_compiled = ((int8_t)1);
                __tmp_2204 = (int64_t)(__tmp_2207);
            } else {
            }
            int64_t __tmp_2208 = ore_list_get(br, 0LL);
            int8_t __tmp_2209 = ore_list_get_kind(br, 0LL);
            int64_t __tmp_2210 = 0;
            if ((!ore_str_eq((void*)(intptr_t)(96070760348960), ore_str_new("", 0)))) {
                int64_t __tmp_2211 = ore_list_get(br, 0LL);
                int8_t __tmp_2212 = ore_list_get_kind(br, 0LL);
                __tmp_2210 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2211)));
            } else {
            }
            first = ((int8_t)0);
            __tmp_2048 = (int64_t)(emit(st, ore_str_new("}", 1)));
        }
        else if (pat.tag == 6) {
            int64_t start = pat.data[0];
            int64_t end = pat.data[1];
            void* kw = ore_str_new("if", 2);
            int64_t __tmp_2213 = 0;
            if ((!(first))) {
                kw = ore_str_new("else if", 7);
            } else {
            }
            int64_t __tmp_2214 = ore_list_get(subj, 0LL);
            int8_t __tmp_2215 = ore_list_get_kind(subj, 0LL);
            int64_t __tmp_2216 = ore_list_get(subj, 0LL);
            int8_t __tmp_2217 = ore_list_get_kind(subj, 0LL);
            void* br = compile_expr(st, exprs, stmts, body_id);
            int64_t __tmp_2218 = ore_list_get(br, 1LL);
            int8_t __tmp_2219 = ore_list_get_kind(br, 1LL);
            int64_t __tmp_2220 = 0;
            if ((96070760452896 && 96070760458448)) {
                int64_t __tmp_2221 = ore_list_get(br, 1LL);
                int8_t __tmp_2222 = ore_list_get_kind(br, 1LL);
                result_kind = __tmp_2221;
                int64_t __tmp_2223 = 0;
                if ((!ore_str_eq((void*)(intptr_t)(96070677948832), ore_str_new("void", 4)))) {
                    void* ct = kind_to_c_type(result_kind);
                    void* init = match_default_init(result_kind);
                    ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                } else {
                }
                first_arm_compiled = ((int8_t)1);
                __tmp_2220 = (int64_t)(__tmp_2223);
            } else {
            }
            int64_t __tmp_2224 = ore_list_get(br, 0LL);
            int8_t __tmp_2225 = ore_list_get_kind(br, 0LL);
            int64_t __tmp_2226 = 0;
            if ((!ore_str_eq((void*)(intptr_t)(96070760507600), ore_str_new("", 0)))) {
                int64_t __tmp_2227 = ore_list_get(br, 0LL);
                int8_t __tmp_2228 = ore_list_get_kind(br, 0LL);
                __tmp_2226 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2227)));
            } else {
            }
            first = ((int8_t)0);
            __tmp_2048 = (int64_t)(emit(st, ore_str_new("}", 1)));
        }
        else if (pat.tag == 7) {
            int64_t alternatives = pat.data[0];
            void* kw = ore_str_new("if", 2);
            int64_t __tmp_2229 = 0;
            if ((!(first))) {
                kw = ore_str_new("else if", 7);
            } else {
            }
            void* __tmp_2230 = ore_list_new();
            void* conds = __tmp_2230;
            for (int64_t j = 0LL; j < alternatives.len(); j++) {
                int64_t ap = ((alternatives)[j]);
                int64_t __tmp_2231 = 0;
                if (ap.tag == 2) {
                    int64_t av = ap.data[0];
                    int64_t __tmp_2232 = ore_list_get(subj, 0LL);
                    int8_t __tmp_2233 = ore_list_get_kind(subj, 0LL);
                    ore_list_push(conds, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(int64_t)(", 10), ore_dynamic_to_str(__tmp_2232, __tmp_2233)), ore_str_new(") == ", 5)), ore_int_to_str(av)), ore_str_new("LL", 2))));
                }
                else if (ap.tag == 3) {
                    int64_t fv = ap.data[0];
                    int64_t __tmp_2234 = ore_list_get(subj, 0LL);
                    int8_t __tmp_2235 = ore_list_get_kind(subj, 0LL);
                    ore_list_push(conds, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("*(double*)&(", 12), ore_dynamic_to_str(__tmp_2234, __tmp_2235)), ore_str_new(") == ", 5)), ore_int_to_str(fv))));
                }
                else if (ap.tag == 4) {
                    int64_t bv = ap.data[0];
                    int64_t bval = 0LL;
                    int64_t __tmp_2236 = 0;
                    if (bv) {
                        bval = 1LL;
                    } else {
                    }
                    int64_t __tmp_2237 = ore_list_get(subj, 0LL);
                    int8_t __tmp_2238 = ore_list_get_kind(subj, 0LL);
                    ore_list_push(conds, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("(int8_t)(", 9), ore_dynamic_to_str(__tmp_2237, __tmp_2238)), ore_str_new(") == ", 5)), ore_int_to_str(bval))));
                    __tmp_2231 = (int64_t)(__tmp_2236);
                }
                else if (ap.tag == 5) {
                    int64_t sv = ap.data[0];
                    void* slit = compile_string_lit(st, sv);
                    int64_t __tmp_2239 = ore_list_get(subj, 0LL);
                    int8_t __tmp_2240 = ore_list_get_kind(subj, 0LL);
                    int64_t __tmp_2241 = ore_list_get(slit, 0LL);
                    int8_t __tmp_2242 = ore_list_get_kind(slit, 0LL);
                    ore_list_push(conds, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("ore_str_eq(", 11), ore_dynamic_to_str(__tmp_2239, __tmp_2240)), ore_str_new(", ", 2)), ore_dynamic_to_str(__tmp_2241, __tmp_2242)), ore_str_new(")", 1))));
                }
                else if (ap.tag == 6) {
                    int64_t rs = ap.data[0];
                    int64_t re = ap.data[1];
                    int64_t __tmp_2243 = ore_list_get(subj, 0LL);
                    int8_t __tmp_2244 = ore_list_get_kind(subj, 0LL);
                    int64_t __tmp_2245 = ore_list_get(subj, 0LL);
                    int8_t __tmp_2246 = ore_list_get_kind(subj, 0LL);
                    ore_list_push(conds, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("((int64_t)(", 11), ore_dynamic_to_str(__tmp_2243, __tmp_2244)), ore_str_new(") >= ", 5)), ore_int_to_str(rs)), ore_str_new("LL && (int64_t)(", 16)), ore_dynamic_to_str(__tmp_2245, __tmp_2246)), ore_str_new(") <= ", 5)), ore_int_to_str(re)), ore_str_new("LL)", 3))));
                }
                else {
                    ore_list_push(conds, (int64_t)(intptr_t)(ore_str_new("1", 1)));
                }
                cont_243: ;
            }
            brk_242: ;
            void* cond = ore_list_join(conds, ore_str_new(" || ", 4));
            void* br = compile_expr(st, exprs, stmts, body_id);
            int64_t __tmp_2247 = ore_list_get(br, 1LL);
            int8_t __tmp_2248 = ore_list_get_kind(br, 1LL);
            int64_t __tmp_2249 = 0;
            if ((96070760758496 && 96070760764048)) {
                int64_t __tmp_2250 = ore_list_get(br, 1LL);
                int8_t __tmp_2251 = ore_list_get_kind(br, 1LL);
                result_kind = __tmp_2250;
                int64_t __tmp_2252 = 0;
                if ((!ore_str_eq((void*)(intptr_t)(96070678171760), ore_str_new("void", 4)))) {
                    void* ct = kind_to_c_type(result_kind);
                    void* init = match_default_init(result_kind);
                    ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                } else {
                }
                first_arm_compiled = ((int8_t)1);
                __tmp_2249 = (int64_t)(__tmp_2252);
            } else {
            }
            int64_t __tmp_2253 = ore_list_get(br, 0LL);
            int8_t __tmp_2254 = ore_list_get_kind(br, 0LL);
            int64_t __tmp_2255 = 0;
            if ((!ore_str_eq((void*)(intptr_t)(96070760813088), ore_str_new("", 0)))) {
                int64_t __tmp_2256 = ore_list_get(br, 0LL);
                int8_t __tmp_2257 = ore_list_get_kind(br, 0LL);
                __tmp_2255 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2256)));
            } else {
            }
            first = ((int8_t)0);
            __tmp_2048 = (int64_t)(emit(st, ore_str_new("}", 1)));
        }
        else {
            int64_t __tmp_2258 = 0;
            if ((!(first))) {
                __tmp_2258 = (int64_t)(emit(st, ore_str_new("else {", 6)));
            } else {
                __tmp_2258 = (int64_t)(emit(st, ore_str_new("{", 1)));
            }
            void* br = compile_expr(st, exprs, stmts, body_id);
            int64_t __tmp_2259 = ore_list_get(br, 1LL);
            int8_t __tmp_2260 = ore_list_get_kind(br, 1LL);
            int64_t __tmp_2261 = 0;
            if ((96070760878144 && 96070760883696)) {
                int64_t __tmp_2262 = ore_list_get(br, 1LL);
                int8_t __tmp_2263 = ore_list_get_kind(br, 1LL);
                result_kind = __tmp_2262;
                int64_t __tmp_2264 = 0;
                if ((!ore_str_eq((void*)(intptr_t)(96070678271424), ore_str_new("void", 4)))) {
                    void* ct = kind_to_c_type(result_kind);
                    void* init = match_default_init(result_kind);
                    ore_list_set(cg_lines(st), decl_pos, (int64_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(decl_indent, ct), ore_str_new(" ", 1)), t), init), ore_str_new(";", 1))));
                } else {
                }
                first_arm_compiled = ((int8_t)1);
                __tmp_2261 = (int64_t)(__tmp_2264);
            } else {
            }
            int64_t __tmp_2265 = ore_list_get(br, 0LL);
            int8_t __tmp_2266 = ore_list_get_kind(br, 0LL);
            int64_t __tmp_2267 = 0;
            if ((!ore_str_eq((void*)(intptr_t)(96070760932848), ore_str_new("", 0)))) {
                int64_t __tmp_2268 = ore_list_get(br, 0LL);
                int8_t __tmp_2269 = ore_list_get_kind(br, 0LL);
                __tmp_2267 = (int64_t)(emit(st, match_assign_expr(t, result_kind, __tmp_2268)));
            } else {
            }
            first = ((int8_t)0);
            __tmp_2048 = (int64_t)(emit(st, ore_str_new("}", 1)));
        }
        cont_239: ;
    }
    brk_238: ;
    void* __tmp_2270 = ore_list_new();
    ore_list_push(__tmp_2270, (int64_t)(intptr_t)(t));
    ore_list_push(__tmp_2270, (int64_t)(result_kind));
    void* __ret_10 = __tmp_2270;
    return __ret_10;
}

int64_t collect_free_vars_expr(void* exprs, void* stmts, int64_t expr_id, void* bound, void* seen, void* ore_v_free) {
    struct ore_enum_Expr e = get_expr(exprs, expr_id);
    int64_t __tmp_2271 = 0;
    if (e.tag == 4) {
        int64_t name = e.data[0];
        int64_t __tmp_2272 = 0;
        if ((!(list_contains_str(bound, name)))) {
            int64_t __tmp_2273 = 0;
            if ((!(list_contains_str(seen, name)))) {
                ore_list_push(seen, (int64_t)(name));
                ore_list_push(ore_v_free, (int64_t)(name));
            } else {
            }
            __tmp_2272 = (int64_t)(__tmp_2273);
        } else {
        }
        __tmp_2271 = (int64_t)(__tmp_2272);
    }
    else if (e.tag == 0) {
        int64_t _ = e.data[0];
        int64_t st = 0LL;
    }
    else if (e.tag == 1) {
        int64_t _ = e.data[0];
        int64_t st = 0LL;
    }
    else if (e.tag == 2) {
        int64_t _ = e.data[0];
        int64_t st = 0LL;
    }
    else if (e.tag == 3) {
        int64_t _ = e.data[0];
        int64_t st = 0LL;
    }
    else if (e.tag == 23) {
        int64_t st = 0LL;
    }
    else if (e.tag == 24) {
        int64_t st = 0LL;
    }
    else if (e.tag == 5) {
        int64_t op = e.data[0];
        int64_t left = e.data[1];
        int64_t right = e.data[2];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, right, bound, seen, ore_v_free));
    }
    else if (e.tag == 6) {
        int64_t inner = e.data[0];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, inner, bound, seen, ore_v_free));
    }
    else if (e.tag == 7) {
        int64_t inner = e.data[0];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, inner, bound, seen, ore_v_free));
    }
    else if (e.tag == 9) {
        int64_t inner = e.data[0];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, inner, bound, seen, ore_v_free));
    }
    else if (e.tag == 25) {
        int64_t inner = e.data[0];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, inner, bound, seen, ore_v_free));
    }
    else if (e.tag == 26) {
        int64_t inner = e.data[0];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, inner, bound, seen, ore_v_free));
    }
    else if (e.tag == 27) {
        int64_t inner = e.data[0];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, inner, bound, seen, ore_v_free));
    }
    else if (e.tag == 28) {
        int64_t inner = e.data[0];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, inner, bound, seen, ore_v_free));
    }
    else if (e.tag == 29) {
        int64_t inner = e.data[0];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, inner, bound, seen, ore_v_free));
    }
    else if (e.tag == 8) {
        int64_t func = e.data[0];
        int64_t args = e.data[1];
        for (int64_t i = 0LL; i < args.len(); i++) {
            cont_245: ;
        }
        brk_244: ;
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, func, bound, seen, ore_v_free));
    }
    else if (e.tag == 15) {
        int64_t lparams = e.data[0];
        int64_t lbody = e.data[1];
        void* inner_bound = copy_str_list(bound);
        for (int64_t i = 0LL; i < lparams.len(); i++) {
            ore_list_push(inner_bound, (int64_t)(((lparams)[i])));
            cont_247: ;
        }
        brk_246: ;
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, lbody, inner_bound, seen, ore_v_free));
    }
    else if (e.tag == 10) {
        int64_t cond = e.data[0];
        int64_t then_block = e.data[1];
        int64_t else_block = e.data[2];
        __tmp_2271 = (int64_t)(collect_free_vars_block(exprs, stmts, else_block, bound, seen, ore_v_free));
    }
    else if (e.tag == 11) {
        int64_t cond = e.data[0];
        int64_t then_expr = e.data[1];
        int64_t else_expr = e.data[2];
        int64_t __tmp_2274 = 0;
        if ((96070678738432 >= 96070761415600)) {
            __tmp_2274 = (int64_t)(collect_free_vars_expr(exprs, stmts, else_expr, bound, seen, ore_v_free));
        } else {
        }
        __tmp_2271 = (int64_t)(__tmp_2274);
    }
    else if (e.tag == 12) {
        int64_t subject = e.data[0];
        int64_t arms = e.data[1];
        for (int64_t i = 0LL; i < arms.len(); i++) {
            struct ore_rec_MatchArm arm = get_match_arm(arms, i);
            int64_t __tmp_2275 = 0;
            if ((96070761465632 >= 96070761465824)) {
                __tmp_2275 = (int64_t)(collect_free_vars_expr(exprs, stmts, arm.guard, bound, seen, ore_v_free));
            } else {
            }
            cont_249: ;
        }
        brk_248: ;
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, subject, bound, seen, ore_v_free));
    }
    else if (e.tag == 13) {
        int64_t parts = e.data[0];
        for (int64_t i = 0LL; i < parts.len(); i++) {
            struct ore_enum_StringPart p = get_string_part(parts, i);
            int64_t __tmp_2276 = 0;
            if (p.tag == 1) {
                int64_t expr_id = p.data[0];
                __tmp_2276 = (int64_t)(collect_free_vars_expr(exprs, stmts, expr_id, bound, seen, ore_v_free));
            }
            else {
                int64_t st = 0LL;
            }
            cont_251: ;
        }
        brk_250: ;
    }
    else if (e.tag == 14) {
        int64_t block = e.data[0];
        __tmp_2271 = (int64_t)(collect_free_vars_block(exprs, stmts, block, bound, seen, ore_v_free));
    }
    else if (e.tag == 16) {
        int64_t type_name = e.data[0];
        int64_t fields = e.data[1];
        for (int64_t i = 0LL; i < fields.len(); i++) {
            cont_253: ;
        }
        brk_252: ;
    }
    else if (e.tag == 17) {
        int64_t object = e.data[0];
        int64_t field = e.data[1];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, object, bound, seen, ore_v_free));
    }
    else if (e.tag == 30) {
        int64_t object = e.data[0];
        int64_t field = e.data[1];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, object, bound, seen, ore_v_free));
    }
    else if (e.tag == 18) {
        int64_t object = e.data[0];
        int64_t method = e.data[1];
        int64_t args = e.data[2];
        for (int64_t i = 0LL; i < args.len(); i++) {
            cont_255: ;
        }
        brk_254: ;
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, object, bound, seen, ore_v_free));
    }
    else if (e.tag == 31) {
        int64_t object = e.data[0];
        int64_t method = e.data[1];
        int64_t args = e.data[2];
        for (int64_t i = 0LL; i < args.len(); i++) {
            cont_257: ;
        }
        brk_256: ;
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, object, bound, seen, ore_v_free));
    }
    else if (e.tag == 19) {
        int64_t elements = e.data[0];
        for (int64_t i = 0LL; i < elements.len(); i++) {
            cont_259: ;
        }
        brk_258: ;
    }
    else if (e.tag == 20) {
        int64_t expr = e.data[0];
        int64_t var_name = e.data[1];
        int64_t iterable = e.data[2];
        int64_t cond = e.data[3];
        void* inner_bound = copy_str_list(bound);
        ore_list_push(inner_bound, (int64_t)(var_name));
        int64_t __tmp_2277 = 0;
        if ((96070679086704 >= 96070761778720)) {
            __tmp_2277 = (int64_t)(collect_free_vars_expr(exprs, stmts, cond, inner_bound, seen, ore_v_free));
        } else {
        }
        __tmp_2271 = (int64_t)(__tmp_2277);
    }
    else if (e.tag == 21) {
        int64_t entries = e.data[0];
        for (int64_t i = 0LL; i < entries.len(); i++) {
            cont_261: ;
        }
        brk_260: ;
    }
    else if (e.tag == 22) {
        int64_t object = e.data[0];
        int64_t index = e.data[1];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, index, bound, seen, ore_v_free));
    }
    else if (e.tag == 32) {
        int64_t cond = e.data[0];
        int64_t message = e.data[1];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, cond, bound, seen, ore_v_free));
    }
    else if (e.tag == 33) {
        int64_t left = e.data[0];
        int64_t right = e.data[1];
        int64_t message = e.data[2];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, right, bound, seen, ore_v_free));
    }
    else if (e.tag == 34) {
        int64_t left = e.data[0];
        int64_t right = e.data[1];
        int64_t message = e.data[2];
        __tmp_2271 = (int64_t)(collect_free_vars_expr(exprs, stmts, right, bound, seen, ore_v_free));
    }
    else {
        int64_t st = 0LL;
    }
    return __tmp_2271;
}

int64_t collect_free_vars_stmt(void* exprs, void* stmts, int64_t stmt_id, void* bound, void* seen, void* ore_v_free) {
    struct ore_enum_Stmt s = get_stmt(stmts, stmt_id);
    int64_t __tmp_2278 = 0;
    if (s.tag == 0) {
        int64_t name = s.data[0];
        int64_t mutable = s.data[1];
        int64_t value = s.data[2];
        __tmp_2278 = (int64_t)(collect_free_vars_expr(exprs, stmts, value, bound, seen, ore_v_free));
    }
    else if (s.tag == 1) {
        int64_t names = s.data[0];
        int64_t value = s.data[1];
        __tmp_2278 = (int64_t)(collect_free_vars_expr(exprs, stmts, value, bound, seen, ore_v_free));
    }
    else if (s.tag == 2) {
        int64_t name = s.data[0];
        int64_t value = s.data[1];
        __tmp_2278 = (int64_t)(collect_free_vars_expr(exprs, stmts, value, bound, seen, ore_v_free));
    }
    else if (s.tag == 5) {
        int64_t expr = s.data[0];
        __tmp_2278 = (int64_t)(collect_free_vars_expr(exprs, stmts, expr, bound, seen, ore_v_free));
    }
    else if (s.tag == 6) {
        int64_t value = s.data[0];
        int64_t __tmp_2279 = 0;
        if ((96070679343456 >= 96070762047312)) {
            __tmp_2279 = (int64_t)(collect_free_vars_expr(exprs, stmts, value, bound, seen, ore_v_free));
        } else {
        }
        __tmp_2278 = (int64_t)(__tmp_2279);
    }
    else if (s.tag == 14) {
        int64_t expr = s.data[0];
        __tmp_2278 = (int64_t)(collect_free_vars_expr(exprs, stmts, expr, bound, seen, ore_v_free));
    }
    else if (s.tag == 7) {
        int64_t var_name = s.data[0];
        int64_t start = s.data[1];
        int64_t end = s.data[2];
        int64_t step = s.data[3];
        int64_t body = s.data[4];
        int64_t __tmp_2280 = 0;
        if ((96070679408080 >= 96070762116800)) {
            __tmp_2280 = (int64_t)(collect_free_vars_expr(exprs, stmts, step, bound, seen, ore_v_free));
        } else {
        }
        __tmp_2278 = (int64_t)(collect_free_vars_block(exprs, stmts, body, bound, seen, ore_v_free));
    }
    else if (s.tag == 8) {
        int64_t cond = s.data[0];
        int64_t body = s.data[1];
        __tmp_2278 = (int64_t)(collect_free_vars_block(exprs, stmts, body, bound, seen, ore_v_free));
    }
    else if (s.tag == 9) {
        int64_t var_name = s.data[0];
        int64_t iterable = s.data[1];
        int64_t body = s.data[2];
        __tmp_2278 = (int64_t)(collect_free_vars_block(exprs, stmts, body, bound, seen, ore_v_free));
    }
    else if (s.tag == 10) {
        int64_t key_var = s.data[0];
        int64_t val_var = s.data[1];
        int64_t iterable = s.data[2];
        int64_t body = s.data[3];
        __tmp_2278 = (int64_t)(collect_free_vars_block(exprs, stmts, body, bound, seen, ore_v_free));
    }
    else if (s.tag == 11) {
        int64_t body = s.data[0];
        __tmp_2278 = (int64_t)(collect_free_vars_block(exprs, stmts, body, bound, seen, ore_v_free));
    }
    else if (s.tag == 3) {
        int64_t object = s.data[0];
        int64_t index = s.data[1];
        int64_t value = s.data[2];
        __tmp_2278 = (int64_t)(collect_free_vars_expr(exprs, stmts, value, bound, seen, ore_v_free));
    }
    else if (s.tag == 4) {
        int64_t object = s.data[0];
        int64_t field = s.data[1];
        int64_t value = s.data[2];
        __tmp_2278 = (int64_t)(collect_free_vars_expr(exprs, stmts, value, bound, seen, ore_v_free));
    }
    else {
        int64_t st = 0LL;
    }
    return __tmp_2278;
}

int64_t collect_free_vars_block(void* exprs, void* stmts, struct ore_rec_Block block, void* bound, void* seen, void* ore_v_free) {
    void* ss = block.stmts;
    for (int64_t i = 0LL; i < ore_list_len(ss); i++) {
        struct ore_rec_SpannedStmt sp = get_sstmt(ss, i);
        cont_263: ;
    }
    brk_262: ;
}

int8_t list_contains_str(void* lst, void* needle) {
    for (int64_t i = 0LL; i < ore_list_len(lst); i++) {
        int64_t __tmp_2281 = 0;
        if (ore_str_eq(str_at(lst, i), needle)) {
            return ((int8_t)1);
        } else {
        }
        cont_265: ;
    }
    brk_264: ;
    return ((int8_t)0);
}

void* copy_str_list(void* src) {
    void* __tmp_2282 = ore_list_new();
    void* dst = __tmp_2282;
    for (int64_t i = 0LL; i < ore_list_len(src); i++) {
        int64_t __tmp_2283 = ore_list_get(src, i);
        int8_t __tmp_2284 = ore_list_get_kind(src, i);
        ore_list_push(dst, (int64_t)(__tmp_2283));
        cont_267: ;
    }
    brk_266: ;
    return dst;
}

void* find_free_vars(void* exprs, void* stmts, int64_t expr_id, void* params) {
    void* __tmp_2285 = ore_list_new();
    void* bound = __tmp_2285;
    for (int64_t i = 0LL; i < ore_list_len(params); i++) {
        int64_t __tmp_2286 = ore_list_get(params, i);
        int8_t __tmp_2287 = ore_list_get_kind(params, i);
        ore_list_push(bound, (int64_t)(__tmp_2286));
        cont_269: ;
    }
    brk_268: ;
    void* __tmp_2288 = ore_list_new();
    void* seen = __tmp_2288;
    void* __tmp_2289 = ore_list_new();
    void* ore_v_free = __tmp_2289;
    return ore_v_free;
}

void* compile_lambda(void* st, void* exprs, void* stmts, void* params, int64_t body) {
    void* lname = cg_label(st, ore_str_new("lambda", 6));
    void* free_vars = find_free_vars(exprs, stmts, body, params);
    void* __tmp_2290 = ore_list_new();
    void* capture_names = __tmp_2290;
    void* __tmp_2291 = ore_list_new();
    void* capture_kinds = __tmp_2291;
    for (int64_t i = 0LL; i < ore_list_len(free_vars); i++) {
        void* fv = str_at(free_vars, i);
        int64_t __tmp_2292 = 0;
        if (cg_has_var(st, fv)) {
            ore_list_push(capture_names, (int64_t)(intptr_t)(fv));
            ore_list_push(capture_kinds, (int64_t)(intptr_t)(cg_get_var_kind(st, fv)));
        } else {
        }
        cont_271: ;
    }
    brk_270: ;
    int8_t has_captures = (96070762541808 > 96070762542048);
    void* param_sig = ore_str_new("", 0);
    int64_t __tmp_2293 = 0;
    if (has_captures) {
        param_sig = ore_str_new("void* __env", 11);
    } else {
    }
    for (int64_t i = 0LL; i < ore_list_len(params); i++) {
        int64_t __tmp_2294 = 0;
        if ((!ore_str_eq(param_sig, ore_str_new("", 0)))) {
            param_sig = ore_str_concat(param_sig, ore_str_new(", ", 2));
        } else {
        }
        int64_t __tmp_2295 = ore_list_get(params, i);
        int8_t __tmp_2296 = ore_list_get_kind(params, i);
        param_sig = ore_str_concat(param_sig, ore_str_concat(ore_str_new("int64_t ", 8), ore_dynamic_to_str(__tmp_2295, __tmp_2296)));
        cont_273: ;
    }
    brk_272: ;
    void* struct_name = ore_str_new("", 0);
    int64_t __tmp_2297 = 0;
    if (has_captures) {
        struct_name = ore_str_concat(ore_str_new("__captures_", 11), lname);
        void* tl = cg_list(st, 9LL);
        void* struct_def = ore_str_concat(ore_str_concat(ore_str_new("struct ", 7), struct_name), ore_str_new(" {", 2));
        for (int64_t i = 0LL; i < ore_list_len(capture_names); i++) {
            void* cn = str_at(capture_names, i);
            void* ck = str_at(capture_kinds, i);
            void* ct = kind_to_c_type(ck);
            struct_def = ore_str_concat(struct_def, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new(" ", 1), ct), ore_str_new(" ", 1)), cn), ore_str_new(";", 1)));
            cont_275: ;
        }
        brk_274: ;
        struct_def = ore_str_concat(struct_def, ore_str_new(" };", 3));
        ore_list_push(tl, (int64_t)(intptr_t)(struct_def));
        ore_list_push(tl, (int64_t)(intptr_t)(ore_str_new("", 0)));
    } else {
    }
    void* saved_lines = cg_lines(st);
    int64_t saved_indent = cg_indent(st);
    int64_t saved_var_count = ore_list_len(cg_list(st, 4LL));
    void* __tmp_2298 = ore_list_new();
    ore_list_set(st, 0LL, (int64_t)(__tmp_2298));
    int64_t __tmp_2299 = 0;
    if (has_captures) {
        for (int64_t i = 0LL; i < ore_list_len(capture_names); i++) {
            void* cn = str_at(capture_names, i);
            void* ck = str_at(capture_kinds, i);
            void* ct = kind_to_c_type(ck);
            cont_277: ;
        }
        brk_276: ;
        __tmp_2299 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("struct ", 7), struct_name), ore_str_new("* __cap = (struct ", 18)), struct_name), ore_str_new("*)__env;", 8))));
    } else {
    }
    for (int64_t i = 0LL; i < ore_list_len(params); i++) {
        void* pn = str_at(params, i);
        cont_279: ;
    }
    brk_278: ;
    void* r = compile_expr(st, exprs, stmts, body);
    int64_t __tmp_2300 = ore_list_get(r, 0LL);
    int8_t __tmp_2301 = ore_list_get_kind(r, 0LL);
    void* body_lines = cg_lines(st);
    ore_list_set(st, 0LL, (int64_t)(saved_lines));
    void* vnames = cg_list(st, 4LL);
    void* vkinds = cg_list(st, 5LL);
    while ((96070762842016 > 96070680149920)) {
        cont_281: ;
    }
    brk_280: ;
    void* tl2 = cg_list(st, 9LL);
    ore_list_push(tl2, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("static int64_t ", 15), lname), ore_str_new("(", 1)), param_sig), ore_str_new(") {", 3))));
    for (int64_t i = 0LL; i < ore_list_len(body_lines); i++) {
        int64_t __tmp_2302 = ore_list_get(body_lines, i);
        int8_t __tmp_2303 = ore_list_get_kind(body_lines, i);
        ore_list_push(tl2, (int64_t)(__tmp_2302));
        cont_283: ;
    }
    brk_282: ;
    ore_list_push(tl2, (int64_t)(intptr_t)(ore_str_new("}", 1)));
    ore_list_push(tl2, (int64_t)(intptr_t)(ore_str_new("", 0)));
    int64_t __tmp_2304 = 0;
    if (has_captures) {
        void* cap_var = cg_tmp(st);
        for (int64_t i = 0LL; i < ore_list_len(capture_names); i++) {
            void* cn = str_at(capture_names, i);
            void* mn = mangle_var(cn);
            cont_285: ;
        }
        brk_284: ;
        void* __tmp_2305 = ore_list_new();
        ore_list_push(__tmp_2305, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("__closure_", 10), lname), ore_str_new("|", 1)), cap_var)));
        ore_list_push(__tmp_2305, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        __tmp_2304 = (int64_t)(__tmp_2305);
    } else {
        void* __tmp_2306 = ore_list_new();
        ore_list_push(__tmp_2306, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("(void*)&", 8), lname)));
        ore_list_push(__tmp_2306, (int64_t)(intptr_t)(ore_str_new("int", 3)));
        __tmp_2304 = (int64_t)(__tmp_2306);
    }
    return __tmp_2304;
}

void* parse_closure_expr(void* cexpr) {
    int64_t __tmp_2307 = 0;
    if (ore_str_starts_with(cexpr, ore_str_new("__closure_", 10))) {
        void* rest = ore_str_substr(cexpr, 10LL, (96070762985184 - 96070762985424));
        void* chars = ore_str_chars(rest);
        int64_t pipe_idx = (-(1LL));
        for (int64_t i = 0LL; i < ore_list_len(chars); i++) {
            int64_t __tmp_2308 = ore_list_get(chars, i);
            int8_t __tmp_2309 = ore_list_get_kind(chars, i);
            int64_t __tmp_2310 = 0;
            if (ore_str_eq((void*)(intptr_t)(96070763003744), ore_str_new("|", 1))) {
                pipe_idx = i;
            } else {
            }
            cont_287: ;
        }
        brk_286: ;
        int64_t __tmp_2311 = 0;
        if ((96070680320560 >= 96070763014032)) {
            void* lname = ore_str_substr(rest, 0LL, pipe_idx);
            void* cap_var = ore_str_substr(rest, (96070680335472 + 96070763025184), (96070763030288 - 96070763030512));
            void* __tmp_2312 = ore_list_new();
            ore_list_push(__tmp_2312, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("(void*)&", 8), lname)));
            ore_list_push(__tmp_2312, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("(void*)&", 8), cap_var)));
            return __tmp_2312;
        } else {
        }
        __tmp_2307 = (int64_t)(__tmp_2311);
    } else {
    }
    void* __tmp_2313 = ore_list_new();
    ore_list_push(__tmp_2313, (int64_t)(intptr_t)(cexpr));
    ore_list_push(__tmp_2313, (int64_t)(intptr_t)(ore_str_new("NULL", 4)));
    void* result = __tmp_2313;
    return result;
}

void* compile_record_construct(void* st, void* exprs, void* stmts, void* type_name, void* fields) {
    void* t = cg_tmp(st);
    void* c_type = ore_str_concat(ore_str_new("struct ore_rec_", 15), type_name);
    for (int64_t i = 0LL; i < ore_list_len(fields); i++) {
        void* pair = cg_list(fields, i);
        void* field_name = str_at(pair, 0LL);
        int64_t __tmp_2314 = ore_list_get(pair, 1LL);
        int8_t __tmp_2315 = ore_list_get_kind(pair, 1LL);
        int64_t value_id = __tmp_2314;
        void* r = compile_expr(st, exprs, stmts, value_id);
        int64_t __tmp_2316 = ore_list_get(r, 0LL);
        int8_t __tmp_2317 = ore_list_get_kind(r, 0LL);
        cont_289: ;
    }
    brk_288: ;
    void* __tmp_2318 = ore_list_new();
    ore_list_push(__tmp_2318, (int64_t)(intptr_t)(t));
    ore_list_push(__tmp_2318, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("rec:", 4), type_name)));
    return __tmp_2318;
}

int64_t compile_type_def(void* st, struct ore_rec_TypeDefNode td) {
    void* name = td.name;
    void* fields = td.fields;
    void* tl = cg_list(st, 9LL);
    ore_list_push(tl, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("// Record: ", 11), name)));
    void* sig = ore_str_concat(ore_str_concat(ore_str_new("struct ore_rec_", 15), name), ore_str_new(" {", 2));
    void* field_names_str = ore_str_new("", 0);
    void* field_kinds_str = ore_str_new("", 0);
    for (int64_t i = 0LL; i < ore_list_len(fields); i++) {
        struct ore_rec_FieldDef f = get_field_def(fields, i);
        void* fn_str = f.name;
        void* ft = type_expr_to_kind_str(st, f.ty);
        void* c_type = kind_to_c_type(ft);
        sig = ore_str_concat(sig, ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new(" ", 1), c_type), ore_str_new(" ", 1)), fn_str), ore_str_new(";", 1)));
        int64_t __tmp_2319 = 0;
        if ((96070680533568 > 96070763247728)) {
            field_names_str = ore_str_concat(field_names_str, ore_str_new(",", 1));
            field_kinds_str = ore_str_concat(field_kinds_str, ore_str_new(",", 1));
        } else {
        }
        field_names_str = ore_str_concat(field_names_str, fn_str);
        field_kinds_str = ore_str_concat(field_kinds_str, ft);
        cont_291: ;
    }
    brk_290: ;
    sig = ore_str_concat(sig, ore_str_new(" };", 3));
    ore_list_push(tl, (int64_t)(intptr_t)(sig));
    ore_list_push(tl, (int64_t)(intptr_t)(ore_str_new("", 0)));
    return cg_set_fn(st, name, ore_str_concat(ore_str_new("rec:", 4), name));
}

int64_t compile_enum_def(void* st, struct ore_rec_EnumDefNode ed) {
    void* name = ed.name;
    void* variants = ed.variants;
    void* tl = cg_list(st, 9LL);
    int64_t max_fields = 0LL;
    void* variant_names_str = ore_str_new("", 0);
    for (int64_t i = 0LL; i < ore_list_len(variants); i++) {
        struct ore_rec_VariantDef v = get_variant_def(variants, i);
        int64_t __tmp_2320 = 0;
        if ((96070763347040 > 96070680641024)) {
            max_fields = ore_list_len(v.fields);
        } else {
        }
        int64_t __tmp_2321 = 0;
        if ((96070680650064 > 96070763356976)) {
            variant_names_str = ore_str_concat(variant_names_str, ore_str_new(",", 1));
        } else {
        }
        variant_names_str = ore_str_concat(variant_names_str, v.name);
        cont_293: ;
    }
    brk_292: ;
    void* data_decl = ore_str_new("", 0);
    int64_t __tmp_2322 = 0;
    if ((96070680682176 > 96070763383248)) {
        data_decl = ore_str_concat(ore_str_concat(ore_str_new(" int64_t data[", 14), ore_int_to_str(max_fields)), ore_str_new("];", 2));
    } else {
    }
    ore_list_push(tl, (int64_t)(intptr_t)(ore_str_concat(ore_str_new("// Enum: ", 9), name)));
    ore_list_push(tl, (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("struct ore_enum_", 16), name), ore_str_new(" { int8_t tag;", 14)), data_decl), ore_str_new(" };", 3))));
    ore_list_push(tl, (int64_t)(intptr_t)(ore_str_new("", 0)));
    return cg_add_enum(st, name, ore_str_concat(ore_str_concat(name, ore_str_new(":", 1)), variant_names_str), ore_list_len(variants));
}

int64_t compile_fn_def(void* st, void* exprs, void* stmts, struct ore_rec_FnDef fd) {
    void* name = fd.name;
    void* params = fd.params;
    struct ore_rec_Block body = fd.body;
    struct ore_enum_TypeExpr ret_te = fd.ret_type;
    void* ret_kind = type_expr_to_kind_str(st, ret_te);
    void* ret_c = kind_to_c_type(ret_kind);
    int64_t __tmp_2323 = 0;
    if (ore_str_eq(name, ore_str_new("main", 4))) {
        ret_c = ore_str_new("int", 3);
        ret_kind = ore_str_new("int", 3);
    } else {
    }
    int64_t saved_var_count = ore_list_len(cg_list(st, 4LL));
    void* mn = mangle_fn(name);
    void* param_sig = ore_str_new("", 0);
    for (int64_t i = 0LL; i < ore_list_len(params); i++) {
        struct ore_rec_ParamDef p = get_param_def(params, i);
        void* pk = type_expr_to_kind_str(st, p.ty);
        void* pc = kind_to_c_type(pk);
        void* pn = mangle_var(p.name);
        int64_t __tmp_2324 = 0;
        if ((96070680848128 > 96070763552368)) {
            param_sig = ore_str_concat(param_sig, ore_str_new(", ", 2));
        } else {
        }
        param_sig = ore_str_concat(param_sig, ore_str_concat(ore_str_concat(pc, ore_str_new(" ", 1)), pn));
        cont_295: ;
    }
    brk_294: ;
    ore_list_push(cg_list(st, 8LL), (int64_t)(intptr_t)(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ret_c, ore_str_new(" ", 1)), mn), ore_str_new("(", 1)), param_sig), ore_str_new(");", 2))));
    void* block_result = compile_block(st, exprs, stmts, body);
    int64_t __tmp_2325 = 0;
    if (ore_str_eq(name, ore_str_new("main", 4))) {
        __tmp_2325 = (int64_t)(emit(st, ore_str_new("return 0;", 9)));
    } else {
        int64_t __tmp_2326 = ore_list_get(block_result, 0LL);
        int8_t __tmp_2327 = ore_list_get_kind(block_result, 0LL);
        int64_t __tmp_2328 = 0;
        if ((96070763646000 && 96070763650832)) {
            int64_t __tmp_2329 = ore_list_get(block_result, 1LL);
            int8_t __tmp_2330 = ore_list_get_kind(block_result, 1LL);
            int64_t bk = __tmp_2329;
            int64_t __tmp_2331 = 0;
            if ((96070763661760 && 96070763670320)) {
                int64_t __tmp_2332 = ore_list_get(block_result, 0LL);
                int8_t __tmp_2333 = ore_list_get_kind(block_result, 0LL);
                void* coerced = coerce_from_i64_expr(__tmp_2332, ret_kind);
                __tmp_2331 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("return ", 7), coerced), ore_str_new(";", 1))));
            } else {
                int64_t __tmp_2334 = ore_list_get(block_result, 0LL);
                int8_t __tmp_2335 = ore_list_get_kind(block_result, 0LL);
                __tmp_2331 = (int64_t)(emit(st, ore_str_concat(ore_str_concat(ore_str_new("return ", 7), ore_dynamic_to_str(__tmp_2334, __tmp_2335)), ore_str_new(";", 1))));
            }
            __tmp_2328 = (int64_t)(__tmp_2331);
        } else {
        }
        __tmp_2325 = (int64_t)(__tmp_2328);
    }
    return emit_raw(st, ore_str_new("}", 1));
}

struct ore_enum_TypeExpr resolve_self_type(struct ore_enum_TypeExpr te, void* type_name) {
    struct ore_enum_TypeExpr __tmp_2336 = {0};
    if (te.tag == 0) {
        int64_t name = te.data[0];
        int64_t __tmp_2337 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070681017200), ore_str_new("Self", 4))) {
            struct ore_rec_NamedType __tmp_2338;
            __tmp_2338.name = type_name;
            return __tmp_2338;
        } else {
        }
        __tmp_2336 = te;
    }
    else if (te.tag == 1) {
        int64_t name = te.data[0];
        int64_t args = te.data[1];
        int64_t new_name = name;
        int64_t __tmp_2339 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070681037344), ore_str_new("Self", 4))) {
            new_name = type_name;
        } else {
        }
        void* __tmp_2340 = ore_list_new();
        void* new_args = __tmp_2340;
        for (int64_t i = 0LL; i < args.len(); i++) {
            ore_list_push(new_args, ({ struct ore_enum_TypeExpr __v2i = resolve_self_type(((args)[i]), type_name); (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_enum_TypeExpr)), &__v2i, sizeof(struct ore_enum_TypeExpr)); }));
            cont_297: ;
        }
        brk_296: ;
        struct ore_rec_GenericType __tmp_2341;
        __tmp_2341.name = new_name;
        __tmp_2341.args = new_args;
        __tmp_2336 = __tmp_2341;
    }
    else {
        __tmp_2336 = te;
    }
    return __tmp_2336;
}

struct ore_rec_FnDef mangle_impl_method(void* type_name, struct ore_rec_FnDef method) {
    void* mangled_name = ore_str_concat(ore_str_concat(type_name, ore_str_new("_", 1)), method.name);
    void* __tmp_2342 = ore_list_new();
    void* resolved_params = __tmp_2342;
    for (int64_t i = 0LL; i < ore_list_len(method.params); i++) {
        struct ore_rec_ParamDef p = get_param_def(method.params, i);
        struct ore_enum_TypeExpr resolved_ty = resolve_self_type(p.ty, type_name);
        struct ore_rec_ParamDef __tmp_2343;
        __tmp_2343.name = p.name;
        __tmp_2343.ty = resolved_ty;
        __tmp_2343.default_expr = p.default_expr;
        ore_list_push(resolved_params, ({ struct ore_rec_ParamDef __v2i = __tmp_2343; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_ParamDef)), &__v2i, sizeof(struct ore_rec_ParamDef)); }));
        cont_299: ;
    }
    brk_298: ;
    struct ore_enum_TypeExpr resolved_ret = resolve_self_type(method.ret_type, type_name);
    int8_t has_self = ((int8_t)0);
    int64_t __tmp_2344 = 0;
    if ((96070763857216 > 96070763857456)) {
        struct ore_rec_ParamDef first_p = get_param_def(resolved_params, 0LL);
        int64_t __tmp_2345 = 0;
        if (ore_str_eq(first_p.name, ore_str_new("self", 4))) {
            has_self = ((int8_t)1);
        } else {
        }
        __tmp_2344 = (int64_t)(__tmp_2345);
    } else {
    }
    void* __tmp_2346 = ore_list_new();
    void* final_params = __tmp_2346;
    int64_t __tmp_2347 = 0;
    if (has_self) {
        final_params = resolved_params;
    } else {
        struct ore_rec_ParamDef __tmp_2348;
        __tmp_2348.name = ore_str_new("self", 4);
        struct ore_rec_NamedType __tmp_2349;
        __tmp_2349.name = type_name;
        __tmp_2348.ty = __tmp_2349;
        __tmp_2348.default_expr = (-(1LL));
        ore_list_push(final_params, ({ struct ore_rec_ParamDef __v2i = __tmp_2348; (int64_t)(intptr_t)memcpy(malloc(sizeof(struct ore_rec_ParamDef)), &__v2i, sizeof(struct ore_rec_ParamDef)); }));
        for (int64_t i = 0LL; i < ore_list_len(resolved_params); i++) {
            int64_t __tmp_2350 = ore_list_get(resolved_params, i);
            int8_t __tmp_2351 = ore_list_get_kind(resolved_params, i);
            ore_list_push(final_params, (int64_t)(__tmp_2350));
            cont_301: ;
        }
        brk_300: ;
    }
    struct ore_rec_FnDef __tmp_2352;
    __tmp_2352.name = mangled_name;
    __tmp_2352.type_params = method.type_params;
    __tmp_2352.params = final_params;
    __tmp_2352.ret_type = resolved_ret;
    __tmp_2352.body = method.body;
    return __tmp_2352;
}

int64_t compile_impl_methods(void* st, void* exprs, void* stmts, void* type_name, void* methods) {
    for (int64_t i = 0LL; i < ore_list_len(methods); i++) {
        struct ore_rec_FnDef method = get_fn_def(methods, i);
        struct ore_rec_FnDef mangled = mangle_impl_method(type_name, method);
        cont_303: ;
    }
    brk_302: ;
}

int64_t compile_item(void* st, void* exprs, void* stmts, struct ore_enum_Item item) {
    int64_t __tmp_2353 = 0;
    if (item.tag == 0) {
        int64_t fn_def = item.data[0];
        __tmp_2353 = (int64_t)(compile_fn_def(st, exprs, stmts, fn_def));
    }
    else if (item.tag == 1) {
        int64_t type_def = item.data[0];
        __tmp_2353 = (int64_t)(compile_type_def(st, type_def));
    }
    else if (item.tag == 2) {
        int64_t enum_def = item.data[0];
        __tmp_2353 = (int64_t)(compile_enum_def(st, enum_def));
    }
    else if (item.tag == 7) {
        int64_t name = item.data[0];
        int64_t body = item.data[1];
        void* test_name = ore_str_concat(ore_str_new("test_", 5), ore_int_to_str(name));
        struct ore_rec_FnDef __tmp_2354;
        __tmp_2354.name = test_name;
        void* __tmp_2355 = ore_list_new();
        __tmp_2354.type_params = __tmp_2355;
        void* __tmp_2356 = ore_list_new();
        __tmp_2354.params = __tmp_2356;
        struct ore_rec_NamedType __tmp_2357;
        __tmp_2357.name = ore_str_new("Void", 4);
        __tmp_2354.ret_type = __tmp_2357;
        __tmp_2354.body = body;
        struct ore_rec_FnDef test_fn = __tmp_2354;
        __tmp_2353 = (int64_t)(compile_fn_def(st, exprs, stmts, test_fn));
    }
    else if (item.tag == 6) {
        int64_t path = item.data[0];
        __tmp_2353 = (int64_t)(0LL);
    }
    else if (item.tag == 3) {
        int64_t type_name = item.data[0];
        int64_t methods = item.data[1];
        __tmp_2353 = (int64_t)(compile_impl_methods(st, exprs, stmts, type_name, methods));
    }
    else if (item.tag == 5) {
        int64_t trait_name = item.data[0];
        int64_t type_name = item.data[1];
        int64_t methods = item.data[2];
        __tmp_2353 = (int64_t)(compile_impl_methods(st, exprs, stmts, type_name, methods));
    }
    else {
        __tmp_2353 = (int64_t)(emit_raw(st, ore_str_new("// (skipped item)", 17)));
    }
    return __tmp_2353;
}

void* generate(void* items, void* exprs, void* stmts) {
    void* st = cg_new();
    ore_list_set(st, 23LL, (int64_t)(exprs));
    ore_list_set(st, 24LL, (int64_t)(stmts));
    for (int64_t i = 0LL; i < ore_list_len(items); i++) {
        struct ore_enum_Item item = get_item(items, i);
        int64_t __tmp_2358 = 0;
        if (item.tag == 1) {
            int64_t type_def = item.data[0];
            __tmp_2358 = (int64_t)(compile_type_def(st, type_def));
        }
        else if (item.tag == 2) {
            int64_t enum_def = item.data[0];
            __tmp_2358 = (int64_t)(compile_enum_def(st, enum_def));
        }
        else if (item.tag == 0) {
            int64_t fn_def = item.data[0];
            int64_t __tmp_2359 = 0;
            if ((96070764154880 > 96070764155120)) {
                __tmp_2359 = (int64_t)(cg_add_generic_fn(st, fn_def.name, fn_def));
            } else {
                void* ret_kind = type_expr_to_kind_str(st, fn_def.ret_type);
                __tmp_2359 = (int64_t)(cg_set_fn(st, fn_def.name, ret_kind));
            }
            __tmp_2358 = (int64_t)(__tmp_2359);
        }
        else if (item.tag == 3) {
            int64_t type_name = item.data[0];
            int64_t methods = item.data[1];
            for (int64_t j = 0LL; j < methods.len(); j++) {
                struct ore_rec_FnDef m = get_fn_def(methods, j);
                struct ore_rec_FnDef mangled = mangle_impl_method(type_name, m);
                void* ret_kind = type_expr_to_kind_str(st, mangled.ret_type);
                cont_307: ;
            }
            brk_306: ;
        }
        else if (item.tag == 5) {
            int64_t trait_name = item.data[0];
            int64_t type_name = item.data[1];
            int64_t methods = item.data[2];
            for (int64_t j = 0LL; j < methods.len(); j++) {
                struct ore_rec_FnDef m = get_fn_def(methods, j);
                struct ore_rec_FnDef mangled = mangle_impl_method(type_name, m);
                void* ret_kind = type_expr_to_kind_str(st, mangled.ret_type);
                cont_309: ;
            }
            brk_308: ;
        }
        else {
            __tmp_2358 = (int64_t)(0LL);
        }
        cont_305: ;
    }
    brk_304: ;
    void* top = cg_list(st, 9LL);
    for (int64_t i = 0LL; i < ore_list_len(top); i++) {
        int64_t __tmp_2360 = ore_list_get(top, i);
        int8_t __tmp_2361 = ore_list_get_kind(top, i);
        cont_311: ;
    }
    brk_310: ;
    void* header_lines = cg_list(st, 0LL);
    void* __tmp_2362 = ore_list_new();
    ore_list_set(st, 0LL, (int64_t)(__tmp_2362));
    for (int64_t i = 0LL; i < ore_list_len(items); i++) {
        struct ore_enum_Item item = get_item(items, i);
        int64_t __tmp_2363 = 0;
        if (item.tag == 0) {
            int64_t fn_def = item.data[0];
            int64_t __tmp_2364 = 0;
            if ((96070764333632 == 96070764333872)) {
                __tmp_2364 = (int64_t)(compile_fn_def(st, exprs, stmts, fn_def));
            } else {
            }
            __tmp_2363 = (int64_t)(__tmp_2364);
        }
        else if (item.tag == 7) {
            int64_t name = item.data[0];
            int64_t body = item.data[1];
            int64_t __tmp_2365 = ore_list_get(items, i);
            int8_t __tmp_2366 = ore_list_get_kind(items, i);
            __tmp_2363 = (int64_t)(compile_item(st, exprs, stmts, __tmp_2365));
        }
        else if (item.tag == 3) {
            int64_t type_name = item.data[0];
            int64_t methods = item.data[1];
            __tmp_2363 = (int64_t)(compile_impl_methods(st, exprs, stmts, type_name, methods));
        }
        else if (item.tag == 5) {
            int64_t trait_name = item.data[0];
            int64_t type_name = item.data[1];
            int64_t methods = item.data[2];
            __tmp_2363 = (int64_t)(compile_impl_methods(st, exprs, stmts, type_name, methods));
        }
        else {
            __tmp_2363 = (int64_t)(0LL);
        }
        cont_313: ;
    }
    brk_312: ;
    void* fn_lines = cg_list(st, 0LL);
    ore_list_set(st, 0LL, (int64_t)(header_lines));
    void* fwd = cg_list(st, 8LL);
    int64_t __tmp_2367 = 0;
    if ((96070764422224 > 96070764422464)) {
        for (int64_t i = 0LL; i < ore_list_len(fwd); i++) {
            int64_t __tmp_2368 = ore_list_get(fwd, i);
            int8_t __tmp_2369 = ore_list_get_kind(fwd, i);
            cont_315: ;
        }
        brk_314: ;
        __tmp_2367 = (int64_t)(emit_raw(st, ore_str_new("", 0)));
    } else {
    }
    void* deferred = cg_list(st, 25LL);
    for (int64_t i = 0LL; i < ore_list_len(deferred); i++) {
        int64_t __tmp_2370 = ore_list_get(deferred, i);
        int8_t __tmp_2371 = ore_list_get_kind(deferred, i);
        cont_317: ;
    }
    brk_316: ;
    for (int64_t i = 0LL; i < ore_list_len(fn_lines); i++) {
        int64_t __tmp_2372 = ore_list_get(fn_lines, i);
        int8_t __tmp_2373 = ore_list_get_kind(fn_lines, i);
        cont_319: ;
    }
    brk_318: ;
    return ore_list_join(cg_lines(st), ore_str_new("\n", 1));
}

void* dir_of_file(void* path) {
    int64_t last_slash = (-(1LL));
    for (int64_t i = 0LL; i < ore_str_len(path); i++) {
        int64_t __tmp_2374 = 0;
        if (ore_str_eq(ore_str_char_at(path, i), ore_str_new("/", 1))) {
            last_slash = i;
        } else {
        }
        cont_321: ;
    }
    brk_320: ;
    int64_t __tmp_2375 = 0;
    if ((96070681958224 == 96070764521904)) {
        return ore_str_new(".", 1);
    } else {
    }
    int64_t __tmp_2376 = 0;
    if ((96070681962752 == 96070764527216)) {
        return ore_str_new("/", 1);
    } else {
    }
    return ore_str_substr(path, 0LL, last_slash);
}

void* resolve_use_path(void* base_dir, void* use_path) {
    int64_t __tmp_2377 = 0;
    if (ore_str_starts_with(use_path, ore_str_new("/", 1))) {
        return use_path;
    } else {
    }
    int64_t __tmp_2378 = 0;
    if (ore_str_eq(base_dir, ore_str_new(".", 1))) {
        return use_path;
    } else {
    }
    return ore_str_concat(ore_str_concat(base_dir, ore_str_new("/", 1)), use_path);
}

void* parse_use_line(void* line) {
    void* trimmed = ore_str_trim(line);
    int64_t __tmp_2379 = 0;
    if ((!(ore_str_starts_with(trimmed, ore_str_new("use ", 4))))) {
        return ore_str_new("", 0);
    } else {
    }
    void* rest = ore_str_trim(ore_str_substr(trimmed, 4LL, (96070764578800 - 96070764579040)));
    int64_t __tmp_2380 = 0;
    if ((96070764585568 == 96070764585808)) {
        return ore_str_new("", 0);
    } else {
    }
    int64_t __tmp_2381 = 0;
    if ((96070764592304 && 96070764595408)) {
        return ore_str_substr(rest, 1LL, (96070764600800 - 96070764601040));
    } else {
    }
    return ore_str_concat(rest, ore_str_new(".ore", 4));
}

void* resolve_imports(void* source, void* base_dir, void* loaded) {
    void* lines = ore_str_split(source, ore_str_new("\n", 1));
    void* imported_source = ore_str_new("", 0);
    void* __tmp_2382 = ore_list_new();
    void* main_lines = __tmp_2382;
    for (int64_t i = 0LL; i < ore_list_len(lines); i++) {
        int64_t __tmp_2383 = ore_list_get(lines, i);
        int8_t __tmp_2384 = ore_list_get_kind(lines, i);
        void* use_path = parse_use_line(__tmp_2383);
        int64_t __tmp_2385 = 0;
        if (ore_str_eq(use_path, ore_str_new("", 0))) {
            int64_t __tmp_2386 = ore_list_get(lines, i);
            int8_t __tmp_2387 = ore_list_get_kind(lines, i);
            ore_list_push(main_lines, (int64_t)(__tmp_2386));
        } else {
            void* resolved = resolve_use_path(base_dir, use_path);
            int8_t already = ((int8_t)0);
            for (int64_t j = 0LL; j < ore_list_len(loaded); j++) {
                int64_t __tmp_2388 = ore_list_get(loaded, j);
                int8_t __tmp_2389 = ore_list_get_kind(loaded, j);
                int64_t __tmp_2390 = 0;
                if (ore_str_eq((void*)(intptr_t)(96070764670144), resolved)) {
                    already = ((int8_t)1);
                    goto brk_324;
                } else {
                }
                cont_325: ;
            }
            brk_324: ;
            int64_t __tmp_2391 = 0;
            if ((!(already))) {
                ore_list_push(loaded, (int64_t)(intptr_t)(resolved));
                void* dep_source = ore_file_read(resolved);
                int64_t __tmp_2392 = 0;
                if ((!ore_str_eq(dep_source, ore_str_new("", 0)))) {
                    void* dep_dir = dir_of_file(resolved);
                    void* dep_merged = resolve_imports(dep_source, dep_dir, loaded);
                    imported_source = ore_str_concat(ore_str_concat(imported_source, dep_merged), ore_str_new("\n", 1));
                } else {
                }
                __tmp_2391 = (int64_t)(__tmp_2392);
            } else {
            }
            __tmp_2385 = (int64_t)(__tmp_2391);
        }
        cont_323: ;
    }
    brk_322: ;
    return ore_str_concat(imported_source, ore_list_join(main_lines, ore_str_new("\n", 1)));
}

void* parse_source(void* source) {
    void* split = lex_split(source);
    void* tokens = s_get_list(split, 0LL);
    void* __tmp_2393 = ore_list_new();
    void* holder = __tmp_2393;
    int8_t ok = parse_to_lists(split, holder);
    int64_t __tmp_2394 = 0;
    if ((!(ok))) {
        void* __tmp_2395 = ore_list_new();
        void* err_result = __tmp_2395;
        void* msg = ore_str_new("parse error", 11);
        int64_t __tmp_2396 = 0;
        if ((96070764767040 > 96070764767280)) {
            int64_t __tmp_2397 = ore_list_get(holder, 0LL);
            int8_t __tmp_2398 = ore_list_get_kind(holder, 0LL);
            msg = ore_dynamic_to_str(__tmp_2397, __tmp_2398);
        } else {
        }
        ore_list_push(err_result, (int64_t)(intptr_t)(msg));
        return err_result;
        __tmp_2394 = (int64_t)(__tmp_2396);
    } else {
    }
    void* __tmp_2399 = ore_list_new();
    void* result = __tmp_2399;
    ore_list_push(result, (int64_t)(intptr_t)(tokens));
    ore_list_push(result, (int64_t)(intptr_t)(s_get_list(holder, 0LL)));
    ore_list_push(result, (int64_t)(intptr_t)(s_get_list(holder, 1LL)));
    ore_list_push(result, (int64_t)(intptr_t)(s_get_list(holder, 2LL)));
    return result;
}

void* run_typecheck(void* parsed) {
    void* tokens = s_get_list(parsed, 0LL);
    void* exprs = s_get_list(parsed, 2LL);
    void* stmts = s_get_list(parsed, 3LL);
    return typecheck(tokens, exprs, stmts);
}

int64_t report_errors(void* errors, void* file) {
    for (int64_t i = 0LL; i < ore_list_len(errors); i++) {
        int64_t __tmp_2400 = ore_list_get(errors, i);
        int8_t __tmp_2401 = ore_list_get_kind(errors, i);
        ore_str_print(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_new("typecheck error in ", 19), file), ore_str_new(": ", 2)), ore_dynamic_to_str(__tmp_2400, __tmp_2401)));
        cont_327: ;
    }
    brk_326: ;
    return ore_list_len(errors);
}

void* find_runtime_lib() {
    void* env_path = ore_env_get(ore_str_new("ORE_RUNTIME_LIB", 15));
    int64_t __tmp_2402 = 0;
    if ((!ore_str_eq(env_path, ore_str_new("", 0)))) {
        int64_t __tmp_2403 = 0;
        if (ore_file_exists(env_path)) {
            return env_path;
        } else {
        }
        __tmp_2402 = (int64_t)(__tmp_2403);
    } else {
    }
    int64_t __tmp_2404 = ore_list_get(ore_args(), 0LL);
    int8_t __tmp_2405 = ore_list_get_kind(ore_args(), 0LL);
    int64_t exe = __tmp_2404;
    void* exe_dir = dir_of_file(exe);
    void* candidate = ore_str_concat(exe_dir, ore_str_new("/libore_runtime.a", 17));
    int64_t __tmp_2406 = 0;
    if (ore_file_exists(candidate)) {
        return candidate;
    } else {
    }
    int64_t __tmp_2407 = 0;
    if (ore_file_exists(ore_str_new("target/debug/libore_runtime.a", 29))) {
        return ore_str_new("target/debug/libore_runtime.a", 29);
    } else {
    }
    int64_t __tmp_2408 = 0;
    if (ore_file_exists(ore_str_new("target/release/libore_runtime.a", 31))) {
        return ore_str_new("target/release/libore_runtime.a", 31);
    } else {
    }
    return ore_str_new("libore_runtime.a", 16);
}

void* c_output_path(void* file) {
    int64_t __tmp_2409 = 0;
    if (ore_str_ends_with(file, ore_str_new(".ore", 4))) {
        __tmp_2409 = (int64_t)(ore_str_concat(ore_str_substr(file, 0LL, (96070764974800 - 96070764975040)), ore_str_new(".c", 2)));
    } else {
        __tmp_2409 = (int64_t)(ore_str_concat(file, ore_str_new(".c", 2)));
    }
    return __tmp_2409;
}

void* bin_output_path(void* file) {
    int64_t __tmp_2410 = 0;
    if (ore_str_ends_with(file, ore_str_new(".ore", 4))) {
        __tmp_2410 = (int64_t)(ore_str_substr(file, 0LL, (96070764996112 - 96070764996352)));
    } else {
        __tmp_2410 = (int64_t)(ore_str_concat(file, ore_str_new(".out", 4)));
    }
    return __tmp_2410;
}

int64_t cmd_check(void* file) {
    void* source = ore_file_read(file);
    int64_t __tmp_2411 = 0;
    if (ore_str_eq(source, ore_str_new("", 0))) {
        ore_str_print(ore_str_concat(ore_str_concat(ore_str_new("error: could not read file '", 28), file), ore_str_new("'", 1)));
        return 1LL;
    } else {
    }
    void* base_dir = dir_of_file(file);
    void* __tmp_2412 = ore_list_new();
    ore_list_push(__tmp_2412, (int64_t)(intptr_t)(file));
    void* loaded = __tmp_2412;
    void* merged = resolve_imports(source, base_dir, loaded);
    void* parsed = parse_source(merged);
    int64_t __tmp_2413 = 0;
    if ((96070765057344 == 96070765057584)) {
        int64_t __tmp_2414 = ore_list_get(parsed, 0LL);
        int8_t __tmp_2415 = ore_list_get_kind(parsed, 0LL);
        ore_str_print(ore_str_concat(ore_str_new("error: ", 7), ore_dynamic_to_str(__tmp_2414, __tmp_2415)));
        return 1LL;
    } else {
    }
    void* errors = run_typecheck(parsed);
    int64_t __tmp_2416 = 0;
    if ((96070765076672 > 96070765076912)) {
        return 1LL;
        __tmp_2416 = (int64_t)(report_errors(errors, file));
    } else {
    }
    ore_str_print(ore_str_concat(ore_str_concat(ore_str_new("ok: ", 4), file), ore_str_new(" passed type checking", 21)));
    return 0LL;
}

int64_t cmd_run(void* file) {
    void* source = ore_file_read(file);
    int64_t __tmp_2417 = 0;
    if (ore_str_eq(source, ore_str_new("", 0))) {
        ore_str_print(ore_str_concat(ore_str_concat(ore_str_new("error: could not read file '", 28), file), ore_str_new("'", 1)));
        return 1LL;
    } else {
    }
    void* base_dir = dir_of_file(file);
    void* __tmp_2418 = ore_list_new();
    ore_list_push(__tmp_2418, (int64_t)(intptr_t)(file));
    void* loaded = __tmp_2418;
    void* merged = resolve_imports(source, base_dir, loaded);
    void* parsed = parse_source(merged);
    int64_t __tmp_2419 = 0;
    if ((96070765150288 == 96070765150528)) {
        int64_t __tmp_2420 = ore_list_get(parsed, 0LL);
        int8_t __tmp_2421 = ore_list_get_kind(parsed, 0LL);
        ore_str_print(ore_str_concat(ore_str_new("error: ", 7), ore_dynamic_to_str(__tmp_2420, __tmp_2421)));
        return 1LL;
    } else {
    }
    void* errors = run_typecheck(parsed);
    int64_t __tmp_2422 = 0;
    if ((96070765169616 > 96070765169856)) {
        return 1LL;
        __tmp_2422 = (int64_t)(report_errors(errors, file));
    } else {
    }
    void* result = ore_exec(ore_str_concat(ore_str_new("ore run ", 8), file));
    ore_str_print(result);
    return 0LL;
}

int64_t cmd_build(void* file, void* output) {
    void* source = ore_file_read(file);
    int64_t __tmp_2423 = 0;
    if (ore_str_eq(source, ore_str_new("", 0))) {
        ore_str_print(ore_str_concat(ore_str_concat(ore_str_new("error: could not read file '", 28), file), ore_str_new("'", 1)));
        return 1LL;
    } else {
    }
    void* base_dir = dir_of_file(file);
    void* __tmp_2424 = ore_list_new();
    ore_list_push(__tmp_2424, (int64_t)(intptr_t)(file));
    void* loaded = __tmp_2424;
    void* merged = resolve_imports(source, base_dir, loaded);
    void* parsed = parse_source(merged);
    int64_t __tmp_2425 = 0;
    if ((96070765245264 == 96070765245504)) {
        int64_t __tmp_2426 = ore_list_get(parsed, 0LL);
        int8_t __tmp_2427 = ore_list_get_kind(parsed, 0LL);
        ore_str_print(ore_str_concat(ore_str_new("error: ", 7), ore_dynamic_to_str(__tmp_2426, __tmp_2427)));
        return 1LL;
    } else {
    }
    void* errors = run_typecheck(parsed);
    int64_t __tmp_2428 = 0;
    if ((96070765264592 > 96070765264832)) {
        return 1LL;
        __tmp_2428 = (int64_t)(report_errors(errors, file));
    } else {
    }
    void* items = s_get_list(parsed, 1LL);
    void* exprs = s_get_list(parsed, 2LL);
    void* stmts = s_get_list(parsed, 3LL);
    void* c_code = generate(items, exprs, stmts);
    void* c_path = c_output_path(file);
    void* compiler = ore_env_get(ore_str_new("CC", 2));
    int64_t __tmp_2429 = 0;
    if (ore_str_eq(compiler, ore_str_new("", 0))) {
        compiler = ore_str_new("cc", 2);
    } else {
    }
    void* runtime_lib = find_runtime_lib();
    void* cmd = ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(ore_str_concat(compiler, ore_str_new(" ", 1)), c_path), ore_str_new(" ", 1)), runtime_lib), ore_str_new(" -o ", 4)), output), ore_str_new(" -lm -lpthread -ldl 2>&1", 24));
    void* result = ore_exec(cmd);
    int64_t __tmp_2430 = 0;
    if ((!ore_str_eq(result, ore_str_new("", 0)))) {
        ore_str_print(result);
    } else {
    }
    int64_t __tmp_2431 = 0;
    if ((!(ore_file_exists(output)))) {
        ore_str_print(ore_str_new("error: C compilation failed", 27));
        return 1LL;
    } else {
    }
    ore_str_print(ore_str_concat(ore_str_concat(ore_str_new("compiled to ", 12), output), ore_str_new(" (via C backend)", 16)));
    return 0LL;
}

int64_t print_usage() {
    ore_str_print(ore_str_new("ore-native: Ore compiler (native bootstrap)", 43));
    ore_str_print(ore_str_new("", 0));
    ore_str_print(ore_str_new("Usage: ore-native <command> <file> [options]", 44));
    ore_str_print(ore_str_new("", 0));
    ore_str_print(ore_str_new("Commands:", 9));
    ore_str_print(ore_str_new("  check <file>              Type-check the file", 47));
    ore_str_print(ore_str_new("  run <file>                Type-check and run the file", 55));
    ore_str_print(ore_str_new("  build <file> -o <output>  Compile to binary via C backend", 59));
    ore_str_print(ore_str_new("  help                      Show this help", 42));
}

int main() {
    void* a = ore_args();
    int64_t __tmp_2432 = 0;
    if ((96070765478672 < 96070765478896)) {
        ore_exit(1LL);
        __tmp_2432 = (int64_t)(print_usage());
    } else {
    }
    int64_t __tmp_2433 = ore_list_get(a, 1LL);
    int8_t __tmp_2434 = ore_list_get_kind(a, 1LL);
    int64_t cmd = __tmp_2433;
    int64_t __tmp_2435 = 0;
    if (ore_str_eq((void*)(intptr_t)(96070682953248), ore_str_new("help", 4))) {
        ore_exit(0LL);
        __tmp_2435 = (int64_t)(print_usage());
    } else {
    }
    int64_t __tmp_2436 = 0;
    if ((96070765504000 < 96070765504224)) {
        ore_str_print(ore_str_new("error: missing file argument", 28));
        ore_exit(1LL);
        __tmp_2436 = (int64_t)(print_usage());
    } else {
    }
    int64_t __tmp_2437 = ore_list_get(a, 2LL);
    int8_t __tmp_2438 = ore_list_get_kind(a, 2LL);
    int64_t file = __tmp_2437;
    int64_t __tmp_2439 = 0;
    if (ore_str_eq((void*)(intptr_t)(96070682978656), ore_str_new("check", 5))) {
        int64_t code = cmd_check(file);
        ore_exit(code);
    } else {
        int64_t __tmp_2440 = 0;
        if (ore_str_eq((void*)(intptr_t)(96070682990976), ore_str_new("run", 3))) {
            int64_t code = cmd_run(file);
            ore_exit(code);
        } else {
            int64_t __tmp_2441 = 0;
            if (ore_str_eq((void*)(intptr_t)(96070683003040), ore_str_new("build", 5))) {
                void* output = bin_output_path(file);
                int64_t i = 3LL;
                while ((96070683017552 < 96070765569728)) {
                    int64_t __tmp_2442 = ore_list_get(a, i);
                    int8_t __tmp_2443 = ore_list_get_kind(a, i);
                    int64_t __tmp_2444 = 0;
                    if ((96070765577904 && 96070765583184)) {
                        int64_t __tmp_2445 = ore_list_get(a, (96070683036400 + 96070765588512));
                        int8_t __tmp_2446 = ore_list_get_kind(a, (96070683036400 + 96070765588512));
                        output = __tmp_2445;
                        i = (96070683040448 + 96070765595696);
                    } else {
                        int64_t __tmp_2447 = ore_list_get(a, i);
                        int8_t __tmp_2448 = ore_list_get_kind(a, i);
                        ore_str_print(ore_str_concat(ore_str_concat(ore_str_new("error: unknown option '", 23), ore_dynamic_to_str(__tmp_2447, __tmp_2448)), ore_str_new("'", 1)));
                        ore_exit(1LL);
                        __tmp_2444 = (int64_t)(print_usage());
                    }
                    cont_329: ;
                }
                brk_328: ;
                int64_t code = cmd_build(file, output);
                ore_exit(code);
            } else {
                ore_str_print(ore_str_concat(ore_str_concat(ore_str_new("error: unknown command '", 24), ore_dynamic_to_str(cmd, __tmp_2434)), ore_str_new("'", 1)));
                ore_exit(1LL);
                __tmp_2441 = (int64_t)(print_usage());
            }
            __tmp_2440 = (int64_t)(__tmp_2441);
        }
        __tmp_2439 = (int64_t)(__tmp_2440);
    }
    return 0;
}