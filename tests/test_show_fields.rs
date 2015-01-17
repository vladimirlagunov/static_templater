#![feature(phase)]
#![feature(macro_rules)]
#[phase(link, plugin)] extern crate static_templater;


use std::fmt::Show;


pub struct _Herp<Derp> {
    pub derp: Derp,
}


pub struct _Bar<Baz, Derp> {
    pub baz: Baz,
    pub herp: _Herp<Derp>,
}


pub struct _Example<Foo, Baz, Derp> {
    pub foo: Foo,
    pub bar: _Bar<Baz, Derp>,
}


#[static_templater]
mod templater {
    use std::fmt::Show;
    type ObjType<X: Show, Y: Show, Z: Show> = super::_Example<X, Y, Z>;

    const SOURCE: &'static str =
        "{{ obj.foo }} --- {{ obj.bar.baz }} --- {{ obj.bar.herp.derp }}";
}


fn render<Foo: Show, Baz: Show, Derp: Show>(foo: Foo, baz: Baz, derp: Derp) -> String {
    let val = _Example {foo: foo, bar: _Bar {baz: baz, herp: _Herp {derp: derp}}};
    let args = templater::Args {obj: val};
    templater::render(args)
}


#[test]
fn test_str_string_int() {
    assert!(render("&str", "String", 98765u32).as_slice() ==
            "&str --- String --- 98765");
}
