#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Unit,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Record(String),
    Enum(String),
    Fn {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    Channel,
    /// A type variable for inference (not yet resolved)
    Any,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
            Type::Str => write!(f, "Str"),
            Type::Unit => write!(f, "Unit"),
            Type::List(elem) => write!(f, "List[{}]", elem),
            Type::Map(k, v) => write!(f, "Map[{}, {}]", k, v),
            Type::Option(inner) => write!(f, "Option[{}]", inner),
            Type::Result(ok, err) => write!(f, "Result[{}, {}]", ok, err),
            Type::Record(name) | Type::Enum(name) => write!(f, "{}", name),
            Type::Channel => write!(f, "Channel"),
            Type::Fn { params, ret } => {
                let ps: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "({}) -> {}", ps.join(", "), ret)
            }
            Type::Any => write!(f, "Any"),
        }
    }
}

impl Type {
    /// Check if this type is compatible with another (considering Any as wildcard)
    pub fn compatible_with(&self, other: &Type) -> bool {
        if *self == Type::Any || *other == Type::Any {
            return true;
        }
        match (self, other) {
            (Type::Option(a), Type::Option(b)) | (Type::List(a), Type::List(b)) => a.compatible_with(b),
            (Type::Result(a1, a2), Type::Result(b1, b2)) => a1.compatible_with(b1) && a2.compatible_with(b2),
            (Type::Map(k1, v1), Type::Map(k2, v2)) => k1.compatible_with(k2) && v1.compatible_with(v2),
            _ => self == other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn dummy() -> Self {
        Self { start: 0, end: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    // --- Any wildcard ---

    #[test]
    fn any_is_compatible_with_everything() {
        let types = [
            Type::Int,
            Type::Float,
            Type::Bool,
            Type::Str,
            Type::Unit,
            Type::Channel,
            Type::Any,
            Type::List(Box::new(Type::Int)),
            Type::Option(Box::new(Type::Str)),
            Type::Result(Box::new(Type::Int), Box::new(Type::Str)),
            Type::Map(Box::new(Type::Str), Box::new(Type::Int)),
            Type::Record("Foo".into()),
            Type::Enum("Bar".into()),
            Type::Fn { params: vec![Type::Int], ret: Box::new(Type::Bool) },
        ];
        for t in &types {
            assert!(Type::Any.compatible_with(t), "Any should be compatible with {t}");
            assert!(t.compatible_with(&Type::Any), "{t} should be compatible with Any");
        }
    }

    // --- Identical primitive types ---

    #[test]
    fn identical_primitives_are_compatible() {
        for t in [Type::Int, Type::Float, Type::Bool, Type::Str, Type::Unit, Type::Channel] {
            assert!(t.compatible_with(&t.clone()));
        }
    }

    #[test]
    fn different_primitives_are_not_compatible() {
        assert!(!Type::Int.compatible_with(&Type::Float));
        assert!(!Type::Bool.compatible_with(&Type::Str));
        assert!(!Type::Unit.compatible_with(&Type::Int));
        assert!(!Type::Channel.compatible_with(&Type::Bool));
    }

    // --- Option ---

    #[test]
    fn option_compatible_same_inner() {
        let a = Type::Option(Box::new(Type::Int));
        let b = Type::Option(Box::new(Type::Int));
        assert!(a.compatible_with(&b));
    }

    #[test]
    fn option_incompatible_different_inner() {
        let a = Type::Option(Box::new(Type::Int));
        let b = Type::Option(Box::new(Type::Str));
        assert!(!a.compatible_with(&b));
    }

    #[test]
    fn option_compatible_with_any_inner() {
        let a = Type::Option(Box::new(Type::Any));
        let b = Type::Option(Box::new(Type::Int));
        assert!(a.compatible_with(&b));
        assert!(b.compatible_with(&a));
    }

    // --- List ---

    #[test]
    fn list_compatible_same_element() {
        let a = Type::List(Box::new(Type::Str));
        let b = Type::List(Box::new(Type::Str));
        assert!(a.compatible_with(&b));
    }

    #[test]
    fn list_incompatible_different_element() {
        let a = Type::List(Box::new(Type::Int));
        let b = Type::List(Box::new(Type::Str));
        assert!(!a.compatible_with(&b));
    }

    #[test]
    fn list_compatible_with_any_element() {
        let a = Type::List(Box::new(Type::Any));
        let b = Type::List(Box::new(Type::Float));
        assert!(a.compatible_with(&b));
        assert!(b.compatible_with(&a));
    }

    // --- Result ---

    #[test]
    fn result_compatible_same_types() {
        let a = Type::Result(Box::new(Type::Int), Box::new(Type::Str));
        let b = Type::Result(Box::new(Type::Int), Box::new(Type::Str));
        assert!(a.compatible_with(&b));
    }

    #[test]
    fn result_incompatible_different_ok() {
        let a = Type::Result(Box::new(Type::Int), Box::new(Type::Str));
        let b = Type::Result(Box::new(Type::Float), Box::new(Type::Str));
        assert!(!a.compatible_with(&b));
    }

    #[test]
    fn result_incompatible_different_err() {
        let a = Type::Result(Box::new(Type::Int), Box::new(Type::Str));
        let b = Type::Result(Box::new(Type::Int), Box::new(Type::Bool));
        assert!(!a.compatible_with(&b));
    }

    #[test]
    fn result_compatible_with_any_ok() {
        let a = Type::Result(Box::new(Type::Any), Box::new(Type::Str));
        let b = Type::Result(Box::new(Type::Int), Box::new(Type::Str));
        assert!(a.compatible_with(&b));
    }

    #[test]
    fn result_compatible_with_any_err() {
        let a = Type::Result(Box::new(Type::Int), Box::new(Type::Any));
        let b = Type::Result(Box::new(Type::Int), Box::new(Type::Str));
        assert!(a.compatible_with(&b));
    }

    // --- Map ---

    #[test]
    fn map_compatible_same_types() {
        let a = Type::Map(Box::new(Type::Str), Box::new(Type::Int));
        let b = Type::Map(Box::new(Type::Str), Box::new(Type::Int));
        assert!(a.compatible_with(&b));
    }

    #[test]
    fn map_incompatible_different_key() {
        let a = Type::Map(Box::new(Type::Str), Box::new(Type::Int));
        let b = Type::Map(Box::new(Type::Int), Box::new(Type::Int));
        assert!(!a.compatible_with(&b));
    }

    #[test]
    fn map_incompatible_different_value() {
        let a = Type::Map(Box::new(Type::Str), Box::new(Type::Int));
        let b = Type::Map(Box::new(Type::Str), Box::new(Type::Bool));
        assert!(!a.compatible_with(&b));
    }

    #[test]
    fn map_compatible_with_any_key() {
        let a = Type::Map(Box::new(Type::Any), Box::new(Type::Int));
        let b = Type::Map(Box::new(Type::Str), Box::new(Type::Int));
        assert!(a.compatible_with(&b));
    }

    #[test]
    fn map_compatible_with_any_value() {
        let a = Type::Map(Box::new(Type::Str), Box::new(Type::Any));
        let b = Type::Map(Box::new(Type::Str), Box::new(Type::Int));
        assert!(a.compatible_with(&b));
    }

    // --- Record / Enum ---

    #[test]
    fn record_compatible_same_name() {
        assert!(Type::Record("Foo".into()).compatible_with(&Type::Record("Foo".into())));
    }

    #[test]
    fn record_incompatible_different_name() {
        assert!(!Type::Record("Foo".into()).compatible_with(&Type::Record("Bar".into())));
    }

    #[test]
    fn enum_compatible_same_name() {
        assert!(Type::Enum("Color".into()).compatible_with(&Type::Enum("Color".into())));
    }

    #[test]
    fn enum_incompatible_different_name() {
        assert!(!Type::Enum("Color".into()).compatible_with(&Type::Enum("Shape".into())));
    }

    #[test]
    fn record_not_compatible_with_enum() {
        assert!(!Type::Record("Foo".into()).compatible_with(&Type::Enum("Foo".into())));
    }

    // --- Fn ---

    #[test]
    fn fn_compatible_identical() {
        let a = Type::Fn { params: vec![Type::Int, Type::Str], ret: Box::new(Type::Bool) };
        let b = Type::Fn { params: vec![Type::Int, Type::Str], ret: Box::new(Type::Bool) };
        assert!(a.compatible_with(&b));
    }

    #[test]
    fn fn_incompatible_different_params() {
        let a = Type::Fn { params: vec![Type::Int], ret: Box::new(Type::Bool) };
        let b = Type::Fn { params: vec![Type::Str], ret: Box::new(Type::Bool) };
        assert!(!a.compatible_with(&b));
    }

    #[test]
    fn fn_incompatible_different_ret() {
        let a = Type::Fn { params: vec![Type::Int], ret: Box::new(Type::Bool) };
        let b = Type::Fn { params: vec![Type::Int], ret: Box::new(Type::Str) };
        assert!(!a.compatible_with(&b));
    }

    // --- Cross-kind incompatibility ---

    #[test]
    fn list_not_compatible_with_option() {
        let a = Type::List(Box::new(Type::Int));
        let b = Type::Option(Box::new(Type::Int));
        assert!(!a.compatible_with(&b));
    }

    #[test]
    fn option_not_compatible_with_primitive() {
        assert!(!Type::Option(Box::new(Type::Int)).compatible_with(&Type::Int));
    }

    // --- Nested Any recursion ---

    #[test]
    fn nested_any_in_list_of_option() {
        let a = Type::List(Box::new(Type::Option(Box::new(Type::Any))));
        let b = Type::List(Box::new(Type::Option(Box::new(Type::Int))));
        assert!(a.compatible_with(&b));
    }

    #[test]
    fn nested_any_in_map_of_result() {
        let a = Type::Map(
            Box::new(Type::Str),
            Box::new(Type::Result(Box::new(Type::Any), Box::new(Type::Any))),
        );
        let b = Type::Map(
            Box::new(Type::Str),
            Box::new(Type::Result(Box::new(Type::Int), Box::new(Type::Str))),
        );
        assert!(a.compatible_with(&b));
    }

    // --- Symmetry ---

    #[test]
    fn compatible_with_is_symmetric() {
        let pairs = [
            (Type::Int, Type::Int),
            (Type::Int, Type::Any),
            (Type::List(Box::new(Type::Any)), Type::List(Box::new(Type::Int))),
            (
                Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
                Type::Result(Box::new(Type::Int), Box::new(Type::Str)),
            ),
        ];
        for (a, b) in &pairs {
            assert_eq!(
                a.compatible_with(b),
                b.compatible_with(a),
                "Symmetry violated for {a} vs {b}"
            );
        }
    }
}
