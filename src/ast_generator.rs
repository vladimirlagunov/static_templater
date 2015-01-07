use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Entry;
use std::path::Path;
use std::io::fs::File;
use std::os;

use syntax::{ast, abi};
use syntax::codemap::{Span, Spanned};
use syntax::ext::base;
use syntax::ext::build::AstBuilder;
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token;
use syntax::ptr::P;

use super::template_ast::{TemplateAST, TemplateExpr, RustExpr, RustExprValue};
use super::utils::to_camel_case;

use self::template_parser::template_parser;
peg_file! template_parser("template_parser.rustpeg");


static TEMPLATE_FROM_FILE_USAGE: &'static str = "Usage: #[template_from_file(path=\"path/to/file.html\")] mod fgsfds {}";

pub fn make_templater_module_from_file(ecx: &mut base::ExtCtxt, sp: Span, meta_item: &ast::MetaItem, item: P<ast::Item>) -> P<ast::Item> {
    use syntax::print::pprust;

    let file_relative_path: String = {
        let mut file_relative_path = None;
        match meta_item.node {
            ast::MetaList(_, ref param_vec) => {
                for param in param_vec.iter() {
                    match param.node {
                        ast::MetaNameValue(ref name, Spanned{node: ast::LitStr(ref interned_value, _), ..})
                            if name.get() == "path" =>
                        {
                            file_relative_path = Some(interned_value.get())
                        },
                        _ => {
                            ecx.span_err(sp, TEMPLATE_FROM_FILE_USAGE);
                            return item;
                        },
                    }
                }
            },
            _ => {
                ecx.span_err(sp, TEMPLATE_FROM_FILE_USAGE);
                return item;
            },
        }

        if file_relative_path.is_none() {
            ecx.span_err(sp, TEMPLATE_FROM_FILE_USAGE);
            return item;
        }

        file_relative_path.unwrap().to_string()
    };

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
                    module.view_items.push(
                        ecx.view_use_simple(sp, ast::Inherited, ecx.path_ident(
                            sp, ecx.ident_of("std"))));

                    let result_item = ast::Item {
                        ident: item.ident.clone(),
                        attrs: item.attrs.clone(),
                        id: item.id.clone(),
                        node: ast::ItemMod(module),
                        vis: item.vis.clone(),
                        span: item.span.clone(),
                    };

                    if !os::getenv("STATIC_TEMPLATER_DEBUG").is_none() {
                        ecx.parse_sess.span_diagnostic.span_help(
                            sp, pprust::item_to_string(&result_item).as_slice());
                    }

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

    let template_variables = _get_template_variables(ecx, &template_tree);
    let mut items = Vec::<P<ast::Item>>::new();

    let args_generics = ast::Generics {
        lifetimes: vec![],
        ty_params: OwnedSlice::from_vec(template_variables.iter().map(|var| ast::TyParam {
            ident: var.type_,
            id: ast::DUMMY_NODE_ID,
            bounds: OwnedSlice::from_vec(var.traits.iter().map(
                |x| ecx.typarambound(ecx.path(sp, x.clone()))).collect()),
            unbound: None,
            default: None,
            span: sp,
        }).collect()),
        where_clause: ast::WhereClause {
            id: ast::DUMMY_NODE_ID,
            predicates: vec![],
        },
    };

    items.push(P(ast::Item {
        ident: ecx.ident_of("Args"),
        span: sp,
        vis: ast::Public,
        id: ast::DUMMY_NODE_ID,
        attrs: Vec::new(),
        node: ast::ItemStruct(
            P(ast::StructDef {
                fields: template_variables.iter().map(|var| ast::StructField {
                    span: sp,
                    node: ast::StructField_ {
                        id: ast::DUMMY_NODE_ID,
                        kind: ast::NamedField(var.name, ast::Public),
                        ty: ecx.ty(sp, ast::TyPath(
                            ecx.path_ident(sp, var.type_),
                            ast::DUMMY_NODE_ID)),
                        attrs: vec![],
                    },
                }).collect(),
                ctor_id: None,
            }),
            args_generics.clone()),
    }));

    let mut fn_block_statements = vec![ecx.stmt_let_typed(
        sp,
        true,  // mutable
        ecx.ident_of("result"),
        ecx.ty_ident(sp, ecx.ident_of("String")),
        ecx.expr_call(
            sp,
            ecx.expr_path(ecx.path(sp, vec![ecx.ident_of("String"), ecx.ident_of("new")])),
            vec![]))];
    fn_block_statements.extend(_make_fn_block_statements(ecx, sp, &template_tree).into_iter());

    let fn_block = ecx.block(sp, fn_block_statements, Some(ecx.expr_ident(sp, ecx.ident_of("result"))));

    items.push(P(ast::Item {
        span: sp,
        ident: ecx.ident_of("render"),
        attrs: Vec::new(),
        vis: ast::Public,
        id: ast::DUMMY_NODE_ID,
        node: ast::ItemFn(
            P(ast::FnDecl {
                inputs: vec![ecx.arg(
                    sp,
                    ecx.ident_of("args"),
                    ecx.ty_path(ecx.path_all(
                        sp,
                        false,  // global
                        vec![ecx.ident_of("self"), ecx.ident_of("Args")],
                        vec![],  // lifetimes
                        template_variables.iter().map(
                            |var| ecx.ty_ident(sp, var.type_)).collect(),
                        vec![],
                        )))],
                output: ast::Return(ecx.ty_path(ecx.path_ident(sp, ecx.ident_of("String")))),
                variadic: false,
            }),
            ast::Unsafety::Normal,
            abi::Abi::Rust,
            args_generics,
            fn_block)
    }));

    Ok(items)
}


struct TemplateVariable {
    name: ast::Ident,
    type_: ast::Ident,
    traits: Vec<Vec<ast::Ident>>,
}


#[inline]
fn _get_template_variables<'cx, 'tree>(ecx: &'cx mut base::ExtCtxt, tree: &'tree TemplateAST) -> Vec<TemplateVariable> {
    let mut variables = HashMap::<&'tree str, HashSet<Vec<&'tree str>>>::new();
    {
        let add_trait = |varname: &'tree str, vartrait: Vec<&'tree str>| {
            let mut traits = match variables.entry(varname) {
                Entry::Occupied(v) => v.into_mut(),
                Entry::Vacant(v) => v.set(HashSet::new()),
            };
            traits.insert(vartrait);
        };

        for expr in tree.children.iter() {
            match expr {
                &TemplateExpr::Show(RustExpr::Value(
                    RustExprValue::Ident(ref ident))) =>
                {
                    add_trait(ident.as_slice(),
                              vec!["std", "string", "ToString"]);
                },
                &TemplateExpr::Text(_) => {},
                e => {
                    panic!("{} does not implemented yet", e);
                },
            }
        }
    }

    variables.iter().map(
        |(varname, vartraits): (&&'tree str, &HashSet<Vec<&'tree str>>)|
        TemplateVariable {
            name: ecx.ident_of(*varname),
            type_: ecx.ident_of({
                let mut s = to_camel_case(*varname);
                s.push_str("Type");
                s
            }.as_slice()),
            traits: vartraits.iter().map(
                |pathvec| pathvec.iter().map(
                    |path| ecx.ident_of(*path)).collect()).collect(),
        }).collect()
}


#[inline]
fn _make_fn_block_statements<'cx>(ecx: &'cx mut base::ExtCtxt, sp: Span, tree: &TemplateAST) -> Vec<P<ast::Stmt>> {
    let mut result: Vec<P<ast::Expr>> = Vec::new();

    {
        let push_str_item = |item| ecx.expr_method_call(
            sp,
            ecx.expr_ident(sp, ecx.ident_of("result")),
            ecx.ident_of("push_str"),
            vec![item]);

        let cooked_str = |s: String| ecx.expr_lit(
            sp, ast::LitStr(
                token::intern_and_get_ident(s.as_slice()),
                ast::CookedStr));

        for item in tree.children.iter() {
            match item {
                &TemplateExpr::Text(ref text) => {
                    result.push(push_str_item(cooked_str(text.clone())));
                },
                &TemplateExpr::Show(ref expr) => {
                    let value_expr = _convert_rust_expr_to_ast(ecx, sp, expr);
                    result.push(push_str_item(
                        ecx.expr_method_call(
                            sp, ecx.expr_method_call(
                                sp, value_expr,
                                ecx.ident_of("to_string"), vec![]),
                            ecx.ident_of("as_slice"), vec![])));
                },
            }
        }
    }

    result.into_iter().map(|expr| ecx.stmt_expr(expr)).collect()
}


fn _convert_rust_expr_to_ast(ecx: &base::ExtCtxt, sp: Span, expr: &RustExpr) -> P<ast::Expr> {
    match expr {
        &RustExpr::Value(RustExprValue::Ident(ref ident)) =>
            ecx.expr_field_access(
                sp, ecx.expr_ident(sp, ecx.ident_of("args")),
                ecx.ident_of(ident.as_slice())),
        &RustExpr::Value(RustExprValue::StringLiteral(ref val)) =>
            ecx.expr_str(sp, token::intern_and_get_ident(val.as_slice())),
        &RustExpr::Value(RustExprValue::IntLiteral(ref val)) =>
            ecx.expr_int(sp, *val as int),
        &RustExpr::Value(RustExprValue::FloatLiteral(ref val)) =>
            ecx.expr_lit(sp, ast::LitFloat(
                token::intern_and_get_ident(val.to_string().as_slice()),
                ast::TyF64)),
        &RustExpr::Value(RustExprValue::BoolLiteral(ref val)) =>
            ecx.expr_bool(sp, *val),
        e => {
            panic!("{} does not implemented yet", e);
        }
    }
}
