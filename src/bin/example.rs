#![feature(phase)]
#![feature(globs)]
#[phase(link, plugin)] extern crate log;
#[phase(link, plugin)] extern crate peg_syntax_ext;
#[phase(link, plugin)] extern crate static_templater;

extern crate time;

#[static_templater]
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

    print!("{}", example_templater::render(example_templater::Args {
        user: username,
        time: now(),
    }));
}
