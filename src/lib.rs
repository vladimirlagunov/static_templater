#![crate_name = "static_templater"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]
#![allow(unstable)]
#![feature(plugin_registrar, plugin, box_syntax)]

#[plugin] extern crate peg_syntax_ext;
extern crate syntax;
extern crate rustc;

use syntax::ast::{MetaItem, Item};
use syntax::ptr::P;
use syntax::codemap::Span;
use syntax::parse::token;
use syntax::ext::base::{ExtCtxt, Modifier};
use rustc::plugin::registry::Registry;


mod template_ast;
mod utils;
mod ast_generator;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("print_ast"),
        Modifier(box print_ast_item_modifier));

    reg.register_syntax_extension(
        token::intern("static_templater"),
        Modifier(box ast_generator::make_templater_module));
}


fn print_ast_item_modifier(_: &mut ExtCtxt, _: Span, _: &MetaItem, item: P<Item>) -> P<Item> {
    use syntax::print::pprust;
    println!("****** DEBUG AST:\n{:?}", item);
    println!("****** SERIALIZED BACK INTO CODE:\n{}", pprust::item_to_string(&*item));
    item.clone()
}
