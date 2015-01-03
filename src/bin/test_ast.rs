#![feature(phase)]
#[phase(link, plugin)] extern crate static_templater;

extern crate syntax;


use syntax::ast;
use syntax::print::pprust;
use syntax::ptr::P;
use syntax::codemap::{Spanned, DUMMY_SP};
use syntax::parse;
use syntax::owned_slice::OwnedSlice;


fn main() {
    let sess = syntax::parse::new_parse_sess();

    let simple_expr = ast::Expr {
        id: sess.next_node_id(),
        node: ast::ExprLit(P(Spanned {
            node: ast::LitInt(123, ast::UnsignedIntLit(ast::TyU32)),
            span: DUMMY_SP,
        })),
        span: DUMMY_SP,
    };

    let plus_expr = ast::Expr {
        id: sess.next_node_id(),
        span: DUMMY_SP,
        node: ast::ExprBinary(ast::BiAdd, P(simple_expr.clone()), P(simple_expr.clone())),
    };

    let mult_expr = ast::Expr {
        id: sess.next_node_id(),
        span: DUMMY_SP,
        node: ast::ExprBinary(
            ast::BiMul, 
            P(ast::Expr {
                id: sess.next_node_id(),
                span: DUMMY_SP,
                node: ast::ExprParen(P(plus_expr.clone())),
            }),
            P(ast::Expr {
                id: sess.next_node_id(),
                span: DUMMY_SP,
                node: ast::ExprParen(P(plus_expr.clone())),
            }),
            ),
    };

    let block_expr = ast::Expr {
        id: sess.next_node_id(), span: DUMMY_SP,
        node: ast::ExprBlock(P(ast::Block {
            id: sess.next_node_id(), span: DUMMY_SP,
            rules: ast::DefaultBlock,

            expr: Some(P(mult_expr.clone())),
            
            view_items: vec![],
            
            stmts: vec![
                P(ast::Stmt {
                    span: DUMMY_SP,
                    node: ast::StmtSemi(P(mult_expr.clone()), sess.next_node_id()),
                }),
                P(ast::Stmt {
                    span: DUMMY_SP,
                    node: ast::StmtDecl(P(ast::Decl {
                        span: DUMMY_SP,
                        node: ast::DeclLocal(P(ast::Local{
                            id: sess.next_node_id(), span: DUMMY_SP,
                            
                            init: Some(P(mult_expr.clone())),
                            source: ast::LocalLet,
                            ty: P(ast::Ty {
                                id: sess.next_node_id(), span: DUMMY_SP,
                                node: ast::TyInfer,
                            }),
                            pat: P(ast::Pat {
                                id: sess.next_node_id(), span: DUMMY_SP,
                                node: ast::PatIdent(
                                    ast::BindByValue(ast::MutImmutable),
                                    ast::SpannedIdent {
                                        span: DUMMY_SP,
                                        node: ast::Ident {
                                            name: parse::token::intern("foobar"),
                                            ctxt: 0,
                                        }
                                    },
                                    None,
                                    )
                            }),
                        })),
                        
                    }), sess.next_node_id()),
                }),
                P(ast::Stmt {
                    span: DUMMY_SP,
                    node: ast::StmtSemi(
                        P(ast::Expr {
                            id: sess.next_node_id(), span: DUMMY_SP,
                            node: ast::ExprBinary(
                                ast::BiAdd,
                                P(mult_expr.clone()), 
                                P(ast::Expr {
                                    id: sess.next_node_id(), span: DUMMY_SP,
                                    node: ast::ExprPath(ast::Path {
                                        span: DUMMY_SP,
                                        global: false,
                                        segments: vec![ast::PathSegment {
                                            identifier: parse::token::str_to_ident("foobar"),
                                            parameters: ast::AngleBracketedParameters(
                                                ast::AngleBracketedParameterData {
                                                    lifetimes: vec![],
                                                    types: OwnedSlice::empty(),
                                                    bindings: OwnedSlice::empty(),
                                                })
                                        }],
                                    }),
                                }))
                        }),
                        sess.next_node_id()),
                }),
                    ],
        })),
    };

    println!("{}", pprust::expr_to_string(&block_expr));
}


// #[print_ast()]
// fn helloworld(x: int) -> int {
//     x + 2
// }
