#![feature(core, plugin)]
#[plugin] extern crate static_templater;


#[static_templater]
mod templater {
    const SOURCE: &'static str = "Text {{ value }} text";
}


#[test]
fn test_string() {
    assert!(templater::Context{value: "foobar"}.render().as_slice() == "Text foobar text");
    assert!(templater::Context{value: "foobar".to_string()}.render().as_slice() == "Text foobar text");
}


#[test]
fn test_int() {
    assert!(templater::Context{value: 123u8}.render().as_slice() == "Text 123 text");
    assert!(templater::Context{value: 123u16}.render().as_slice() == "Text 123 text");
    assert!(templater::Context{value: 123u32}.render().as_slice() == "Text 123 text");
    assert!(templater::Context{value: 123u64}.render().as_slice() == "Text 123 text");

    assert!(templater::Context{value: -123i8}.render().as_slice() == "Text -123 text");
    assert!(templater::Context{value: -123i16}.render().as_slice() == "Text -123 text");
    assert!(templater::Context{value: -123i32}.render().as_slice() == "Text -123 text");
    assert!(templater::Context{value: -123i64}.render().as_slice() == "Text -123 text");
}


#[test]
fn test_bool() {
    assert!(templater::Context{value: false}.render().as_slice() == "Text false text");
    assert!(templater::Context{value: true}.render().as_slice() == "Text true text");
}


#[test]
fn test_float() {
    assert!(templater::Context{value: 2.72f32}.render().as_slice() == "Text 2.72 text");
    assert!(templater::Context{value: 3.14f64}.render().as_slice() == "Text 3.14 text");
}


#[test]
fn test_custom() {
    struct X {
        foo: &'static str,
        bar: u64,
    }

    impl std::fmt::Display for X {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "foo={} and bar={}", self.foo, self.bar)
        }
    }

    assert!(templater::Context{value: X {foo: "HeLlOwOrLd", bar: 13579}}.render().as_slice() ==
            "Text foo=HeLlOwOrLd and bar=13579 text");
}
