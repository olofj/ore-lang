/// Runtime value kind tags, shared between codegen and runtime.
/// These must stay in sync with `ValKind::valkind_to_tag()` in ore_codegen.
pub const KIND_INT: i8 = 0;
pub const KIND_FLOAT: i8 = 1;
pub const KIND_BOOL: i8 = 2;
pub const KIND_STR: i8 = 3;
pub const KIND_LIST: i8 = 9;
pub const KIND_MAP: i8 = 10;
