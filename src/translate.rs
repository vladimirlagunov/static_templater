#[deriving(Show, Clone)]
pub enum Expr {
    ExprText(String),
    ExprRustCode(String),
}
