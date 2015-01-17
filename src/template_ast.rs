pub struct TemplateAST {
    pub children: Vec<TemplateExpr>
}


pub enum TemplateExpr {
    Text(String),
    Show(RustExpr),
}


#[derive(Show)]
pub enum RustExpr {
    Value(RustExprValue),
    GetAttribute(Box<RustExpr>, String),
}


#[allow(dead_code, unused_attributes)]
#[derive(Show)]
pub enum RustExprValue {
    Ident(String),
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
}
