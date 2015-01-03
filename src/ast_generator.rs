use std::collections::HashSet;
use std::path::Path;
use std::io::fs::File;
use std::borrow::ToOwned;

use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base;
use syntax::ext::build::AstBuilder;
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token;
use syntax::ptr::P;

use super::template_ast::{TemplateAST, TemplateExpr};
use super::utils::to_camel_case;

use self::template_parser::template_parser;
peg_file! template_parser("template_parser.rustpeg");


pub fn templater_from_file<'cx>(ecx: &'cx mut base::ExtCtxt, sp: Span, tts: &[ast::TokenTree]) -> Box<base::MacResult + 'cx> {
    // Аргументы макроса. Первый аргумент - идентификатор функции,
    // второй аргумент - файл с шаблоном.
    let mut token_parser = ecx.new_parser_from_tts(tts);

    let result_fn_ident = token_parser.parse_ident(); // can fatally exit

    if !token_parser.eat(&token::Comma) {
        ecx.span_err(sp, "expected token: `,`");
        return base::DummyResult::any(sp);
    }

    let file_relative_path: String = token_parser.parse_str().0.get().to_owned();

    if !token_parser.eat(&token::Eof) {
        ecx.span_err(sp, "expected 2 arguments");
        return base::DummyResult::any(sp);
    }

    let source = File::open(&Path::new(file_relative_path.clone())).and_then(
        |mut f| f.read_to_string());

    match source {
        Ok(source) => make_templater_ast(
            ecx,
            sp,
            token_parser.id_to_interned_str(result_fn_ident).get().to_string(),
            source,
            file_relative_path),
        Err(e) => {
            ecx.span_err(sp, format!("unexpected error: {}", e).as_slice());
            base::DummyResult::any(sp)
        },
    }
}


fn make_templater_ast<'cx>(
    ecx: &'cx mut base::ExtCtxt,
    sp: Span,
    result_fn_name: String,
    source: String,
    source_file: String)
    -> Box<base::MacResult + 'cx>
{
    use syntax::print::pprust;

    let template_tree = match template_parser(source.as_slice()) {
        Ok(x) => x,
        Err(e) => {
            ecx.span_err(sp, format!("Syntax error in \"{}\": {}",
                                     source_file, e).as_slice());
            return base::DummyResult::any(sp);
        }
    };

    let template_variables = _get_template_variables(&template_tree);
    let template_variable_types: Vec<String> = template_variables.iter().map(
        |ref varname| {
            let mut s = to_camel_case(varname);
            s.push_str("Type");
            s
        }).collect();
    let args_struct_name = {
        let mut s = to_camel_case(result_fn_name);
        s.push_str("Args");
        s
    };

    let mut items = Vec::<P<ast::Item>>::new();

    items.push(
        ecx.item_struct_poly(
            sp, 
            ecx.ident_of(args_struct_name.as_slice()),  // struct name
            ast::StructDef {
                fields: template_variables.iter().zip(
                    template_variable_types.iter()).map(
                    |(varname, vartype)| ast::StructField {
                        span: sp,
                        node: ast::StructField_ {
                            id: ast::DUMMY_NODE_ID,
                            kind: ast::NamedField(ecx.ident_of(varname.as_slice()), ast::Public),
                            ty: ecx.ty(sp, ast::TyPath(
                                ecx.path(sp, vec![ecx.ident_of(vartype.as_slice())]),
                                ast::DUMMY_NODE_ID)),
                            attrs: vec![],
                        },
                    }).collect(),
                ctor_id: None,
            },
            ast::Generics {
                lifetimes: vec![],
                ty_params: OwnedSlice::from_vec(template_variable_types.iter().map(
                    |name| ast::TyParam {
                        ident: ecx.ident_of(name.as_slice()),
                        id: ast::DUMMY_NODE_ID,
                        bounds: OwnedSlice::empty(),
                        unbound: None,
                        default: None,
                        span: sp,
                    }).collect()),
                where_clause: ast::WhereClause {
                    id: ast::DUMMY_NODE_ID,
                    predicates: vec![],
                }
            })
        );

    println!("*** START OF SERIALIZED CODE ****");
    for item in items.iter() {
        println!("{}", pprust::item_to_string(&**item));
    }
    println!("*** END OF SERIALIZED CODE ****");

    base::MacItems::new(items.into_iter())
}


#[inline]
fn _get_template_variables(tree: &TemplateAST) -> Vec<String> {
    let mut variables = HashSet::<&str>::new();

    for expr in tree.children.iter() {
        match expr {
            &TemplateExpr::ShowVariable(ref var, _) => {
                variables.insert(var.as_slice());
            },
            _ => {},
        }
    }

    let mut result: Vec<String> = variables.into_iter().map(
        |s| s.to_owned()).collect();
    result.as_mut_slice().sort();
    result.shrink_to_fit();
    result
}
