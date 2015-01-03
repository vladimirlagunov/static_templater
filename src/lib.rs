#![crate_name = "static_templater"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]
#![feature(globs)]
#![feature(phase)]
#![feature(plugin_registrar)]
#[phase(link, plugin)] extern crate log;
#[phase(link, plugin)] extern crate peg_syntax_ext;
#[phase(link, plugin)] extern crate syntax;
extern crate rustc;

// use std::collections::HashMap;
// use std::io::IoResult;
// use std::io::fs::File;
// use std::num::pow;
// use std::path::posix::Path;

use syntax::ast::{MetaItem, Item};
use syntax::ptr::P;
use syntax::codemap::Span;
use syntax::parse::token;
// use syntax::ast::{TokenTree, TtToken};
use syntax::ext::base::{ExtCtxt, Modifier};//, MacResult, DummyResult, MacExpr, Decorator};
// use syntax::ext::build::AstBuilder;  // trait for expr_uint
use rustc::plugin::registry::Registry;

// use template_parser::template_parser;
// use template_ast::{TemplateAST, TemplateExpr};

// peg_file! template_parser("template_parser.rustpeg");

mod template_ast;
mod utils;
mod ast_generator;


// pub fn make_template(filename: &str) -> IoResult<()> {
//     let mut dest_file: String = utils::remove_extension(filename).into_string();
//     if !dest_file.ends_with(".rs") {
//         dest_file.push_str(".rs");
//     }

//     info!("Converting {} to {}", filename, dest_file);
//     // let mut source_file = try!(File::open(&Path::new(filename)));

//     let mut reader = &mut try!(File::open(&Path::new(filename)));
//     let mut writer = &mut try!(File::create(&Path::new(dest_file)));

//     let source = try!(reader.read_to_string());
//     let template_ast = match template_parser(source.as_slice()) {
//         Ok(x) => x,
//         Err(msg) => panic!("Tried to generate code for {}: {}",
//                            filename, msg)
//     };

//     let code = generate_template_code("example", &template_ast, true);

//     try!(writer.write_str(code.as_slice()));
//     Ok(())
// }


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    // reg.register_macro("old_templater_from_file", make_templater_from_file);
    reg.register_syntax_extension(
        token::intern("print_ast"),
        Modifier(box print_ast_item_modifier));

    reg.register_macro("templater_from_file", ast_generator::templater_from_file);
}



fn print_ast_item_modifier(ecx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, item: P<Item>) -> P<Item> {
    use syntax::print::pprust;
    println!("****** DEBUG AST:\n{}", item);
    println!("****** SERIALIZED BACK INTO CODE:\n{}", pprust::item_to_string(&*item));
    item.clone()
}


// fn make_templater_from_file(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
//                             -> Box<MacResult + 'static>
// {
//     let (templater_ident, context_struct_ident, source_file) = match args {
//         [TtToken(_, token::Ident(ref templater_ident, token::Plain)),
//          TtToken(_, token::Ident(ref context_struct_ident, token::Plain)),
//          TtToken(_, token::Literal(token::Lit::Str_(ref source_file), _))] 
//             => (templater_ident.name.as_str(),
//                 context_struct_ident.name.as_str(), 
//                 source_file.as_str()),

//         _ => {
//             cx.span_err(sp, "There should be three arguments: templater identifier, context struct identifier and filename of source template");
//             return DummyResult::any(sp);
//         }
//     };

//     let reader = &mut match File::open(&Path::new(source_file)) {
//         Ok(reader) => reader,
//         Err(e) => {
//             cx.span_err(sp, format!("I/O Error when generating {} from \"{}\": {}",
//                                     templater_ident, source_file, e).as_slice());
//             return DummyResult::any(sp);
//         }
//     };
//     let source = match reader.read_to_string() {
//         Ok(source) => source,
//         Err(e) => {
//             cx.span_err(sp, format!("I/O Error when generating {} from \"{}\": {}",
//                                     templater_ident, source_file, e).as_slice());
//             return DummyResult::any(sp);
//         }
//     };

//     let template_ast = match template_parser(source.as_slice()) {
//         Ok(x) => x,
//         Err(msg) => {
//             cx.span_err(sp, format!("Syntax error in \"{}\": {}",
//                                     source_file, msg).as_slice());
//             return DummyResult::any(sp);
//         }
//     };

//     let code = generate_template_code("example", &template_ast, true);

//     MacExpr::new(cx.expr_uint(sp, 0))
// }


// const TRAIT_SHOW: u16 = 0;
// const TRAIT_PARTIAL_EQ: u16 = 1;
// const TRAIT_EQ: u16 = 2;

// const ALL_TRAIT_FLAGS: [&'static str, ..3] = [
//     "Show",
//     "PartialEq",
//     "Eq",
//     ];


// pub fn generate_template_code(name: &str, template_ast: &TemplateAST, as_pub: bool) -> String
// {
//     let mut fn_code = String::new();
//     let mut variables = HashMap::<String, u16>::new();

//     for expr in template_ast.children.iter() {
//         fn_code.push_str((match expr {
//             &TemplateExpr::Text(ref s) => 
//                 format!("    print!(\"{{}}\", r#\"{}\"#);\n", s.as_slice()),
//             &TemplateExpr::ShowVariable(ref var, ref fmt) => {
//                 _hash_map_add_flag(&mut variables, var, TRAIT_SHOW);
//                 match fmt {
//                     &Some(ref fmt) =>
//                         format!("    print!(\"{{{}}}\", {});\n", fmt, var),
//                     &None =>
//                         format!("    print!(\"{{}}\", {});\n", var),
//                 }
//             }
//         }).as_slice());
//     }

//     let mut result = String::new();

//     if as_pub { result.push_str("pub "); }
//     result.push_str("struct TemplateArgs");
//     _push_capitalized(&mut result, name);

//     result.push_str("\n<");
//     for (var, trait_flag) in variables.iter() {
//         let trait_flag: u16 = *trait_flag;
//         result.push_str("\n    Type");
//         _push_capitalized(&mut result, var.as_slice());
        
//         let mut separator_str = ": ";
//         for (ask_trait_flag, trait_flag_name) in ALL_TRAIT_FLAGS.iter().enumerate() {
//             if trait_flag & pow(2, ask_trait_flag) != 0 {
//                 result.push_str(separator_str);
//                 result.push_str(*trait_flag_name);
//                 separator_str = " + ";
//             }
//         }
//         result.push_str(",");
//     }
//     result.push_str("\n>");

//     result.push_str(" {");
//     for var in variables.keys() {
//         result.push_str("\n    ");
//         result.push_str(var.as_slice());
//         result.push_str(": Type");
//         _push_capitalized(&mut result, var.as_slice());
//         result.push(',');
//     }
//     result.push_str("\n}\n\n");
    
//     if (as_pub) { result.push_str("pub "); }
//     result.push_str("fn ");
//     result.push_str(name);
//     result.push_str("(args: &TemplateArgs");
//     _push_capitalized(&mut result, name);
//     result.push_str(") {\n");

//     result.push_str(fn_code.as_slice());
//     result.push_str("}\n\n");
//     result
// }


// fn _push_capitalized(container: &mut String, what: &str) {
//     for chunk in what.split('_') {
//         container.push(chunk.char_at(0).to_ascii().to_uppercase().as_char());
//         container.push_str(chunk.slice_from(1));
//     }
// }


// fn _hash_map_add_flag(holder: &mut HashMap<String, u16>, key: &String, flag: u16) {
//     let new_flag = *holder.get(key).unwrap_or(&0) | pow(2, flag as uint);
//     holder.insert(key.clone(), new_flag);
// }
