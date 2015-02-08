#![feature(plugin)]
#[plugin] extern crate peg_syntax_ext;
#[plugin] extern crate static_templater;

extern crate time;


#[static_templater]
#[allow(dead_code)]
mod example_templater {
    use time::Tm;

    type TimeType = Tm;

    const SOURCE_FILE: &'static str = "data/test.rs.html";
}


fn main() {
    use std::os;
    use time::now;

    let username = match os::args().as_slice() {
        [_, ref username] => username.clone(),
        _ => "%username%".to_string(),
    };

    print!("{}", example_templater::Context {
        user: username,
        time: now(),
    }.render());
}
