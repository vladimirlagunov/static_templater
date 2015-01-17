use std::collections::HashMap;
use std::path::Path;
use std::io::fs::File;
use std::os;

use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base;
use syntax::ext::build::AstBuilder;
use syntax::parse::token;
use syntax::ptr::P;

pub use super::template_ast::{TemplateAST, TemplateExpr, RustExpr, RustExprValue};
pub use super::utils::to_camel_case;


pub fn make_templater_module(ecx: &mut base::ExtCtxt, sp: Span, _: &ast::MetaItem, item: P<ast::Item>) -> P<ast::Item> {
    use syntax::print::pprust;

    match item.node {
        ast::ItemMod(ref module) => {
            let options = match TemplaterOptions::from_module_node(sp.clone(), module) {
                Ok(o) => o,
                Err((sp, msg)) => {
                    ecx.span_err(sp, msg.as_slice());
                    return item.clone();
                }
            };

            let new_items = ast_gen::make(
                ecx, sp, options.source.as_slice(), &options.defined_types);

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
            ecx.span_err(sp, "Expected module declaration after decorator.");
            item.clone()
        },
    }
}


struct TemplaterOptions {
    pub source: String,
    pub defined_types: HashMap<ast::Ident, (P<ast::Ty>, ast::Generics)>,
}


impl TemplaterOptions {
    pub fn from_module_node(sp: Span, module: &ast::Mod)
                            -> Result<Self, (Span, String)>
    {
        let mut result = TemplaterOptions {
            source: "".to_string(),
            defined_types: HashMap::new(),
        };

        {
            let mut template_source = None;
            let mut defined_types = &mut result.defined_types;

            for item in module.items.iter() {
                let &ast::Item {ref ident, ref node, ref span, ..} = item.deref();
                match (token::get_ident(*ident).get(), node) {
                    (_, &ast::ItemTy(ref ty, ref generics)) => {
                        defined_types.insert(ident.clone(), (ty.clone(), generics.clone()));
                    },

                    ("SOURCE", &ast::ItemConst(_, ref expr)) => {
                        if ! template_source.is_none() {
                            return Err((sp, "Template source already specified.".into_string()));
                        }
                        match TemplaterOptions::_str_literal_value(expr.deref(), sp) {
                            Ok(s) => {
                                template_source = Some(s);
                            },
                            Err((sp, msg)) => {
                                return Err((sp, msg.into_string()));
                            }
                        }
                    },

                    ("SOURCE", _) => {
                        return Err((*span, "Expected const &'static str".into_string()));
                    },

                    ("SOURCE_FILE", &ast::ItemConst(_, ref expr)) => {
                        if ! template_source.is_none() {
                            return Err((sp, "Template source already specified.".into_string()));
                        }
                        match TemplaterOptions::_str_literal_value(expr.deref(), sp) {
                            Ok(s) => {
                                let s = File::open(&Path::new(s)).and_then(
                                    |mut f| f.read_to_string());
                                match s {
                                    Ok(s) => {
                                        template_source = Some(s)
                                    },
                                    Err(msg) => {
                                        return Err((sp, format!("{}", msg)));
                                    }
                                }
                            },
                            Err((sp, msg)) => {
                                return Err((sp, msg.into_string()));
                            }
                        }
                    },

                    ("SOURCE_FILE", _) => {
                        return Err((*span, "Expected const &'static str".into_string()));
                    },

                    _ => {},
                }
            }

            result.source = match template_source {
                Some(r) => r,
                None => {
                    return Err((sp, "Define constant SOURCE or SOURCE_FILE".into_string()));
                }
            }
        }

        Ok(result)
    }

    fn _str_literal_value(expr: &ast::Expr, sp: Span)
                          -> Result<String, (Span, &'static str)>
    {
        const EXPECTED_STR_LITERAL: &'static str = "Expected &'static str";

        let spanned_literal =
            if let ast::Expr {node: ast::ExprLit(ref l), ..} = *expr {
                l
            } else {
                return Err((sp, EXPECTED_STR_LITERAL))
            };
        let sp = spanned_literal.span;
        let interned_string =
            if let ast::Lit_::LitStr(ref s, _) = spanned_literal.node {
                s
            } else {
                return Err((sp, EXPECTED_STR_LITERAL))
            };
        Ok(interned_string.get().into_string())
    }
}



mod ast_gen {
    use std::collections::{HashSet, HashMap};
    use std::collections::hash_map::Entry;

    use syntax::{ast, abi};
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::ext::base;
    use syntax::ext::build::AstBuilder;
    use syntax::owned_slice::OwnedSlice;
    use syntax::parse::token;
    use syntax::ptr::P;

    use super::{TemplateAST, TemplateExpr, RustExpr, RustExprValue, to_camel_case};

    use self::template_parser::template_parser;
    peg_file! template_parser("template_parser.rustpeg");

    enum TemplateVariableType {
        Type(P<ast::Ty>, ast::Generics),
        Traits(Vec<ast::Path>),
    }

    struct TemplateVariable {
        pub name: ast::Ident,
        pub type_: TemplateVariableType,
    }

    type KnownTypesSet = HashSet<ast::Ident>;

    pub fn make<'cx>(
        ecx: &'cx mut base::ExtCtxt,
        sp: Span,
        source: &str,
        defined_types: &HashMap<ast::Ident, (P<ast::Ty>, ast::Generics)>)
        -> Result<Vec<P<ast::Item>>, (Span, String)>
    {
        let template_tree = match template_parser(source) {
            Ok(x) => x,
            Err(e) => {
                return Err((sp, format!("Syntax error: {}", e)));
            }
        };

        let template_variables = make_template_variables(ecx, &template_tree.children, defined_types);
        let mut items = Vec::<P<ast::Item>>::new();

        let args_generics = ast::Generics {
            lifetimes: vec![],
            ty_params: OwnedSlice::from_vec({
                let mut result = Vec::new();
                for &TemplateVariable{ref name, ref type_, ..} in template_variables.iter() {
                    match type_ {
                        &TemplateVariableType::Type(_, ref generics) => {
                            result.extend(generics.ty_params.iter().map(
                                |typaram| {
                                    let mut typaram = typaram.clone();
                                    let mut name = to_camel_case(token::get_ident(name.clone()).get().into_string());
                                    name.push_str("Type");
                                    name.push_str(token::get_ident(typaram.ident).get());
                                    name.push_str("Trait");
                                    typaram.ident = ecx.ident_of(name.as_slice());

                                    // let mut bounds: Vec<ast::TyParamBound> = typaram.bounds.into_vec();
                                    // bounds.push(ecx.typarambound(ecx.path(sp, vec![
                                    //     ecx.ident_of("std"), ecx.ident_of("fmt"), ecx.ident_of("Show")])));
                                    // typaram.bounds = OwnedSlice::from_vec(bounds);

                                    typaram
                                }));
                        },
                        
                        &TemplateVariableType::Traits(_) => {
                            result.push(ast::TyParam {
                                ident: {
                                    let mut t = to_camel_case(token::get_ident(name.clone()).get());
                                    t.push_str("Type");
                                    ecx.ident_of(t.as_slice())
                                },
                                id: ast::DUMMY_NODE_ID,
                                bounds: OwnedSlice::from_vec(vec![
                                    ecx.typarambound(ecx.path(sp, vec![
                                        ecx.ident_of("std"), 
                                        ecx.ident_of("string"), 
                                        ecx.ident_of("ToString")])),
                                    ]),
                                unbound: None,
                                default: None,
                                span: sp,
                            });
                        },
                    }
                }
                result
            }),
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
                    fields: template_variables.iter().map(
                        |&TemplateVariable {ref name, ref type_, ..}|
                        ast::StructField {
                            span: sp,
                            node: ast::StructField_ {
                                id: ast::DUMMY_NODE_ID,
                                kind: ast::NamedField(name.clone(), ast::Public),
                                ty: {
                                    let mut t = to_camel_case(token::get_ident(name.clone()).get());
                                    t.push_str("Type");
                                    let t_ident = ecx.ident_of(t.as_slice());
                                    match type_ {
                                        &TemplateVariableType::Type(_, ref type_generics) =>
                                            ecx.ty_path(ecx.path_all(
                                                sp, false, 
                                                vec![ecx.ident_of("self"), t_ident],
                                                vec![],  // lifetimes
                                                type_generics.ty_params.iter().map(
                                                    |typaram| {
                                                        let mut t = t.clone();
                                                        t.push_str(token::get_ident(typaram.ident).get());
                                                        t.push_str("Trait");
                                                        ecx.ty_ident(sp, ecx.ident_of(t.as_slice()))
                                                    }).collect(),
                                                vec![], // bindings
                                                )),
                                        &TemplateVariableType::Traits(_) =>
                                            ecx.ty_ident(sp, t_ident),
                                    }
                                },
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
                            false,  // false is not global
                            vec![ecx.ident_of("self"), ecx.ident_of("Args")],
                            vec![],  // lifetimes
                            args_generics.ty_params.as_slice().iter().map(|ty_param| {
                                ecx.ty_ident(sp, ty_param.ident.clone())
                            }).collect::<Vec<_>>(),
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

    pub fn make_template_variables<'cx, 'tree> (
        ecx: &'cx mut base::ExtCtxt, exprs: &Vec<TemplateExpr>,
        defined_types: &HashMap<ast::Ident, (P<ast::Ty>, ast::Generics)>)
        -> Vec<TemplateVariable>
    {
        let mut variables = HashMap::<ast::Ident, HashSet<ast::Path>>::new();

        for expr in exprs.iter() {
            match expr {
                &TemplateExpr::Text(_) => {},
                &TemplateExpr::Show(ref expr) =>
                    _add_variables_from_rust_expr(ecx, &mut variables, expr),
            };
        }

        let mut result = Vec::<TemplateVariable>::new();

        for (varname, vartraits) in variables.into_iter() {
            let typename = {
                let mut t = to_camel_case(token::get_ident(varname).get());
                t.push_str("Type");
                ecx.ident_of(t.as_slice())
            };
            let defined_type_info = defined_types.get(&typename);

            result.push(TemplateVariable {
                name: varname,
                type_: match defined_type_info {
                    Some(&(ref ty, ref generics)) => TemplateVariableType::Type(ty.clone(), generics.clone()),
                    None => TemplateVariableType::Traits(vartraits.into_iter().collect()),
                },
            });
        }

        result
    }

    fn _add_variables_from_rust_expr(
        ecx: &mut base::ExtCtxt,
        variables: &mut HashMap<ast::Ident, HashSet<ast::Path>>,
        expr: &RustExpr)
    {
        match expr {
            &RustExpr::Value(RustExprValue::Ident(ref ident)) =>
            {
                _add_trait(ecx, variables, ident.as_slice(),
                           vec!["std", "string", "ToString"]);
            },

            &RustExpr::GetAttribute(box ref obj_expr, _) => {
                _add_variables_from_rust_expr(ecx, variables, obj_expr);
            },
            
            e => {
                panic!("{} does not implemented yet", e);
            },
        }
    }

    fn _add_trait(ecx: &base::ExtCtxt, variables: &mut HashMap<ast::Ident, HashSet<ast::Path>>,
                  varname: &str, vartrait: Vec<&str>) {
        let mut traits = match variables.entry(ecx.ident_of(varname)) {
            Entry::Occupied(v) => v.into_mut(),
            Entry::Vacant(v) => v.set(HashSet::new()),
        };
        traits.insert(ecx.path(DUMMY_SP, vartrait.iter().map(|s| ecx.ident_of(*s)).collect()));
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

            &RustExpr::GetAttribute(box ref source_expr, ref attr) =>
                ecx.expr_field_access(
                    sp,
                    _convert_rust_expr_to_ast(ecx, sp, source_expr),
                    ecx.ident_of(attr.as_slice())),
            
            e => {
                panic!("{} does not implemented yet", e);
            }
        }
    }
}
