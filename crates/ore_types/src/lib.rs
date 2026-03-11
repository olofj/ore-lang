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
            Type::Record(name) => write!(f, "{}", name),
            Type::Enum(name) => write!(f, "{}", name),
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
            (Type::Option(a), Type::Option(b)) => a.compatible_with(b),
            (Type::Result(a1, a2), Type::Result(b1, b2)) => a1.compatible_with(b1) && a2.compatible_with(b2),
            (Type::List(a), Type::List(b)) => a.compatible_with(b),
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
