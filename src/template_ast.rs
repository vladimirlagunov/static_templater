use std::error;
use std::fmt;


pub struct TemplateAST {
    pub children: Vec<TemplateExpr>
}


pub enum TemplateExpr {
    Text(String),
    Show(Box<RustExpr>),
}


#[derive(Debug, Clone)]
pub enum RustExpr {
    Value(RustExprValue),
    GetAttribute(Box<RustExpr>, String),
    GetItem(Box<RustExpr>, RustExprValue),
}


#[allow(dead_code, unused_attributes)]
#[derive(Debug, Clone)]
pub enum RustExprValue {
    Ident(String),
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
}


pub struct RustExprError {
    msg: String,
}


impl RustExprError {
    pub fn new(msg: String) -> Self {
        RustExprError {msg: msg}
    }
}


impl error::Error for RustExprError {
    fn description(&self) -> &str {
        self.msg.as_slice()
    }
}


impl fmt::Display for RustExprError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.msg.fmt(fmt)
    }
}


impl RustExpr {
    pub fn concat_as_get_attribute(&self, expr: Box<RustExpr>) -> Result<Box<Self>, RustExprError> {
        Ok(box match *expr {
            RustExpr::Value(RustExprValue::Ident(new_attr)) => {
                RustExpr::GetAttribute(box self.clone(), new_attr)
            },
            RustExpr::GetAttribute(ref expr_box, ref attr) => {
                RustExpr::GetAttribute(
                    try!(self.concat_as_get_attribute(expr_box.clone())), 
                    attr.clone())
            },
            _ => { return Err(RustExprError::new("lolwut".to_string())) },
        })
    }
}
