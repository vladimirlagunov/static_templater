#![feature(phase)]
#![feature(macro_rules)]
#[phase(link, plugin)] extern crate static_templater;


use std::fmt::Show;


#[static_templater]
mod templater {
    const SOURCE: &'static str = "Text {{ value }} text";
}


fn render<'r, T: Show>(v: T) -> String {
    let args = templater::Args::<T> {value: v};
    templater::render(args)
}


#[test]
fn test_string() {
    assert!(render("foobar").as_slice() == "Text foobar text");
    assert!(render("foobar".to_string()).as_slice() == "Text foobar text");
}


#[test]
fn test_int() {
    assert!(render(123u8).as_slice() == "Text 123 text");
    assert!(render(123u16).as_slice() == "Text 123 text");
    assert!(render(123u32).as_slice() == "Text 123 text");
    assert!(render(123u64).as_slice() == "Text 123 text");

    assert!(render(-123i8).as_slice() == "Text -123 text");
    assert!(render(-123i16).as_slice() == "Text -123 text");
    assert!(render(-123i32).as_slice() == "Text -123 text");
    assert!(render(-123i64).as_slice() == "Text -123 text");
}


#[test]
fn test_bool() {
    assert!(render(false).as_slice() == "Text false text");
    assert!(render(true).as_slice() == "Text true text");
}


#[test]
fn test_float() {
    assert!(render(2.72f32).as_slice() == "Text 2.72 text");
    assert!(render(3.14f64).as_slice() == "Text 3.14 text");
}


#[test]
fn test_custom() {
    struct X {
        foo: &'static str,
        bar: u64,
    }

    impl Show for X {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "foo={} and bar={}", self.foo, self.bar)
        }
    }

    assert!(render(X {foo: "HeLlOwOrLd", bar: 13579}).as_slice() ==
            "Text foo=HeLlOwOrLd and bar=13579 text");
}
