#![feature(phase)]
#![feature(globs)]
#[phase(link, plugin)] extern crate log;
#[phase(link, plugin)] extern crate peg_syntax_ext;

use std::io::IoResult;

use std::io::fs::File;
use std::path::posix::Path;
use std::os;
use std::vec::Vec;

use template_parser::template_parser;
use translate::{Expr, ExprText, ExprRustCode};

peg_file! template_parser("template_parser.rustpeg")
mod translate;
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

    generate_template_code::<File, File>(reader, writer)
}


fn generate_template_code<R: Reader, W: Writer>(reader: &mut Reader, writer: &mut Writer) -> IoResult<()>
{
    let nodes = template_parser(try!(reader.read_to_string()).as_slice());
    try!(writer.write_str(format!("{}", nodes).as_slice()));
    Ok(())
}
