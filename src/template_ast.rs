#[deriving(Show, Clone)]
pub struct TemplateAST {
    pub children: Vec<TemplateExpr>
}


#[deriving(Show, Clone)]
pub enum TemplateExpr {
    Text(String),
    Show(RustExpr),
}


#[deriving(Show, Clone)]
pub enum RustExpr {
    Value(RustExprValue),
    GetAttribute(Box<RustExpr>, String),
    GetItem(Box<RustExpr>, RustExprValue),
}


#[deriving(Show, Clone)]
pub enum RustExprValue {
    Ident(String),
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
}
