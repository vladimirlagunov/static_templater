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


pub fn make_templater_module_from_file(ecx: &mut base::ExtCtxt, sp: Span, meta_item: &ast::MetaItem, item: P<ast::Item>) -> P<ast::Item> {
    use syntax::print::pprust;

    println!("******** MetaItem = {}", meta_item);

    let file_relative_path: String = String::from_str("data/test.rs.html");

    let source = File::open(&Path::new(file_relative_path.clone())).and_then(
        |mut f| f.read_to_string());
    let source = match source {
        Ok(source) => source,
        Err(e) => {
            ecx.span_err(sp, format!("unexpected error: {}", e).as_slice());
            return item;
        },
    };

    // Генерация AST
    match item.node {
        ast::ItemMod(ref module) if module.items.len() == 0 => {
            let new_items = make_templater_ast(ecx, sp, source, file_relative_path);
            match new_items {
                Ok(new_items) => {
                    let mut module = module.clone();
                    module.items.extend(new_items.into_iter());
                    let result_item = ast::Item {
                        ident: item.ident.clone(),
                        attrs: item.attrs.clone(),
                        id: item.id.clone(),
                        node: ast::ItemMod(module),
                        vis: item.vis.clone(),
                        span: item.span.clone(),
                    };
                    println!("*** START OF SERIALIZED CODE ****\n{}\n*** END OF SERIALIZED CODE ****",
                             pprust::item_to_string(&result_item));
                    P(result_item)
                },
                Err((sp, msg)) => {
                    ecx.span_err(sp, msg.as_slice());
                    item.clone()
                },
            }
        },
        _ => {
            ecx.span_err(sp, "Expected empty module declaration after decorator.");
            item.clone()
        },
    }
}


fn make_templater_ast<'cx>(
    ecx: &'cx mut base::ExtCtxt,
    sp: Span,
    source: String,
    source_file: String)
    -> Result<Vec<P<ast::Item>>, (Span, String)>
{
    let template_tree = match template_parser(source.as_slice()) {
        Ok(x) => x,
        Err(e) => {
            return Err((sp, format!("Syntax error in \"{}\": {}", source_file, e)));
        }
    };

    let template_variables = _get_template_variables(&template_tree);
    let template_variable_types: Vec<String> = template_variables.iter().map(
        |ref varname| {
            let mut s = to_camel_case(varname);
            s.push_str("Type");
            s
        }).collect();

    let args_generics = ast::Generics {
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
    };

    let mut items = Vec::<P<ast::Item>>::new();

    items.push(ecx.item_struct_poly(
        sp,
        ecx.ident_of("Args"),
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
        args_generics.clone()));

    items.push(ecx.item_fn_poly(
        sp,
        ecx.ident_of("render"),
        vec![ecx.arg(
            sp,
            ecx.ident_of("args"),
            ecx.ty_path(ecx.path_all(
                sp,
                false,  // global
                vec![ecx.ident_of("self"), ecx.ident_of("Args")],
                vec![],  // lifetimes
                template_variable_types.iter().map(
                    |name| ecx.ty_ident(sp, ecx.ident_of(name.as_slice()))).collect(),
                vec![],
                )))],
        ecx.ty_path(ecx.path_ident(sp, ecx.ident_of("String"))),
        args_generics,
        ecx.block(
            sp,
            vec![ecx.stmt_let_typed(
                sp,
                true,  // mutable
                ecx.ident_of("result"),
                ecx.ty_ident(sp, ecx.ident_of("String")),
                ecx.expr_call(
                    sp,
                    ecx.expr_path(ecx.path(sp, vec![ecx.ident_of("String"), ecx.ident_of("new")])),
                    vec![]))],
            Some(ecx.expr_ident(sp, ecx.ident_of("result")),
        ))));

    Ok(items)
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
