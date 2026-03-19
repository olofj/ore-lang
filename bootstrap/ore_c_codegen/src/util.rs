use super::*;

impl CCodeGen {
    /// Map a ValKind to its name for type_of/mangling.
    pub(crate) fn valkind_to_name(kind: &ValKind) -> String {
        match kind {
            ValKind::Int => "Int".to_string(),
            ValKind::Float => "Float".to_string(),
            ValKind::Bool => "Bool".to_string(),
            ValKind::Str => "Str".to_string(),
            ValKind::Void => "Void".to_string(),
            ValKind::Record(name) | ValKind::Enum(name) => name.clone(),
            ValKind::Option => "Option".to_string(),
            ValKind::Result => "Result".to_string(),
            ValKind::List(_) => "List".to_string(),
            ValKind::Map(_) => "Map".to_string(),
            ValKind::Channel => "Channel".to_string(),
        }
    }
}
