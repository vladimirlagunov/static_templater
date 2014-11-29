#![feature(phase)]
#![feature(globs)]
#[phase(link, plugin)] extern crate log;
#[phase(link, plugin)] extern crate peg_syntax_ext;


use std::collections::{HashSet, HashMap};
use std::io::IoResult;
use std::io::fs::File;
use std::num::pow;
use std::path::posix::Path;
use std::os;
use std::vec::Vec;

use template_parser::template_parser;
use template_ast::{TemplateAST, TemplateExpr};

peg_file! template_parser("template_parser.rustpeg")
mod template_ast;
mod utils;


fn main() {
    for argument in os::args().iter().skip(1) {
        make_template(argument.as_slice()).unwrap();
    }
}


fn make_template(filename: &str) -> IoResult<()> {
    let mut dest_file: String = utils::remove_extension(filename).into_string();
    if !dest_file.ends_with(".rs") {
        dest_file.push_str(".rs");
    }

    info!("Converting {} to {}", filename, dest_file);
    let mut source_file = try!(File::open(&Path::new(filename)));

    let mut reader = &mut try!(File::open(&Path::new(filename)));
    let mut writer = &mut try!(File::create(&Path::new(dest_file)));

    let source = try!(reader.read_to_string());
    let template_ast = match template_parser(source.as_slice()) {
        Ok(x) => x,
        Err(msg) => panic!("Tried to generate code for {}: {}",
                           filename, msg)
    };

    let code = generate_template_code("example", &template_ast);

    try!(writer.write_str(code.as_slice()));
    Ok(())
}


const TRAIT_SHOW: u16 = 0;
const TRAIT_PARTIAL_EQ: u16 = 1;
const TRAIT_EQ: u16 = 2;

const ALL_TRAIT_FLAGS: [&'static str, ..3] = [
    "Show",
    "PartialEq",
    "Eq",
    ];


fn generate_template_code(name: &str, template_ast: &TemplateAST) -> String
{
    let mut fn_code = String::new();
    let mut variables = HashMap::<String, u16>::new();

    for expr in template_ast.children.iter() {
        fn_code.push_str((match expr {
            &TemplateExpr::Text(ref s) => 
                format!("    print!(\"{{}}\", r#\"{}\"#);\n", s.as_slice()),
            &TemplateExpr::ShowVariable(ref var, ref fmt) => {
                _hash_map_add_flag(&mut variables, var, TRAIT_SHOW);
                match fmt {
                    &Some(ref fmt) =>
                        format!("    print!(\"{{{}}}\", {});\n", fmt, var),
                    &None =>
                        format!("    print!(\"{{}}\", {});\n", var),
                }
            }
        }).as_slice());
    }

    let mut result = String::new();

    result.push_str("struct TemplateArgs");
    _push_capitalized(&mut result, name);

    result.push_str("\n<");
    for (var, trait_flag) in variables.iter() {
        let trait_flag: u16 = *trait_flag;
        result.push_str("\n    Type");
        _push_capitalized(&mut result, var.as_slice());
        
        let mut separator_str = ": ";
        for (ask_trait_flag, trait_flag_name) in ALL_TRAIT_FLAGS.iter().enumerate() {
            if trait_flag & pow(2, ask_trait_flag) != 0 {
                result.push_str(separator_str);
                result.push_str(*trait_flag_name);
                separator_str = " + ";
            }
        }
        result.push_str(",");
    }
    result.push_str("\n>");

    result.push_str(" {");
    for var in variables.keys() {
        result.push_str("\n    ");
        result.push_str(var.as_slice());
        result.push_str(": Type");
        _push_capitalized(&mut result, var.as_slice());
        result.push(',');
    }
    result.push_str("\n}\n\n");
    
    result.push_str("fn ");
    result.push_str(name);
    result.push_str("(args: &TemplateArgs");
    _push_capitalized(&mut result, name);
    result.push_str(") {\n");

    result.push_str(fn_code.as_slice());
    result.push_str("}\n\n");
    result
}


fn _push_capitalized(container: &mut String, what: &str) {
    for chunk in what.split('_') {
        container.push(chunk.char_at(0).to_ascii().to_uppercase().to_char());
        container.push_str(chunk.slice_from(1));
    }
}


fn _hash_map_add_flag(holder: &mut HashMap<String, u16>, key: &String, flag: u16) {
    let new_flag = *holder.get(key).unwrap_or(&0) | pow(2, flag as uint);
    holder.insert(key.clone(), new_flag);
}
