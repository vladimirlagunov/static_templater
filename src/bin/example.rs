#![feature(phase)]
#![feature(globs)]
#[phase(link, plugin)] extern crate log;
#[phase(link, plugin)] extern crate peg_syntax_ext;
#[phase(link, plugin)] extern crate static_templater;

// use static_templater::make_template;


templater_from_file!(test_generator, "data/test.rs.html");


fn main() {
    // println!("{}", test_generator());
    // for argument in os::args().iter().skip(1) {
    //     make_template(argument.as_slice()).unwrap();
    // }
}
