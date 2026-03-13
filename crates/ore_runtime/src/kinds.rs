/// Runtime value kind tags, shared between codegen and runtime.
/// These must stay in sync with `ValKind::valkind_to_tag()` in ore_codegen.
pub const KIND_INT: i8 = 0;
pub const KIND_FLOAT: i8 = 1;
pub const KIND_BOOL: i8 = 2;
pub const KIND_STR: i8 = 3;
pub const KIND_VOID: i8 = 4;
pub const KIND_RECORD: i8 = 5;
pub const KIND_ENUM: i8 = 6;
pub const KIND_OPTION: i8 = 7;
pub const KIND_RESULT: i8 = 8;
pub const KIND_LIST: i8 = 9;
pub const KIND_MAP: i8 = 10;
pub const KIND_CHANNEL: i8 = 11;
