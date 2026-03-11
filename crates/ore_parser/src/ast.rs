#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    FnDef(FnDef),
    TypeDef(TypeDef),
    EnumDef(EnumDef),
    ImplBlock {
        type_name: String,
        methods: Vec<FnDef>,
    },
    TraitDef(TraitDef),
    ImplTrait {
        trait_name: String,
        type_name: String,
        methods: Vec<FnDef>,
    },
    Use {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: String,
    pub methods: Vec<TraitMethod>,
}

/// A trait method signature (no body).
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: Option<TypeExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub ret_type: Option<TypeExpr>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Named(String),
    /// Generic type application: List[Int], Map[Str, Int]
    Generic(String, Vec<TypeExpr>),
    /// Function type: (Int, Int -> Bool) means fn(Int, Int) -> Bool
    Fn { params: Vec<TypeExpr>, ret: Box<TypeExpr> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<SpannedStmt>,
}

impl Block {
    /// Iterate over statements without span info (convenience for codegen)
    pub fn iter_stmts(&self) -> impl Iterator<Item = &Stmt> {
        self.stmts.iter().map(|s| &s.stmt)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedStmt {
    pub stmt: Stmt,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        mutable: bool,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    IndexAssign {
        object: Expr,
        index: Expr,
        value: Expr,
    },
    FieldAssign {
        object: Expr,
        field: String,
        value: Expr,
    },
    Expr(Expr),
    Return(Option<Expr>),
    ForIn {
        var: String,
        start: Expr,
        end: Expr,
        body: Block,
    },
    While {
        cond: Expr,
        body: Block,
    },
    ForEach {
        var: String,
        iterable: Expr,
        body: Block,
    },
    Loop {
        body: Block,
    },
    Break,
    Continue,
    Spawn(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    StringLit(String),
    Ident(String),
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryMinus(Box<Expr>),
    UnaryNot(Box<Expr>),
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Print(Box<Expr>),
    IfElse {
        cond: Box<Expr>,
        then_block: Block,
        else_block: Option<Block>,
    },
    ColonMatch {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Option<Box<Expr>>,
    },
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    StringInterp(Vec<StringPart>),
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    RecordConstruct {
        type_name: String,
        fields: Vec<(String, Expr)>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    ListLit(Vec<Expr>),
    MapLit(Vec<(Expr, Expr)>),
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Break,
    OptionNone,
    OptionSome(Box<Expr>),
    ResultOk(Box<Expr>),
    ResultErr(Box<Expr>),
    Try(Box<Expr>),
    Sleep(Box<Expr>),
    OptionalChain {
        object: Box<Expr>,
        field: String,
    },
    OptionalMethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Variant {
        name: String,
        bindings: Vec<String>,
    },
    Wildcard,
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    StringLit(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Lit(String),
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Pipe,
}
