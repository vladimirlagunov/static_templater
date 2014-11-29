#[deriving(Show, Clone)]
pub struct TemplateAST {
    pub children: Vec<TemplateExpr>
}


#[deriving(Show, Clone)]
pub enum TemplateExpr {
    Text(String),
    ShowVariable(String, Option<String>),
}
