#![feature(plugin)]
#[plugin] extern crate static_templater;


use std::string::ToString;


pub struct _Herp<Derp: ToString> {
    pub derp: Derp,
}


pub struct _Bar<Baz, Derp: ToString> {
    pub baz: Baz,
    pub herp: _Herp<Derp>,
}


pub struct _Example<Foo: ToString, Baz: ToString, Derp: ToString> {
    pub foo: Foo,
    pub bar: _Bar<Baz, Derp>,
}


#[static_templater]
mod templater {
    use std::string::ToString;

    type ObjType<X: ToString, Y: ToString, Z: ToString> = super::_Example<X, Y, Z>;

    const SOURCE: &'static str =
        "{{ obj.foo }} --- {{ obj.bar.baz }} --- {{ obj.bar.herp.derp }}";
}


fn render<Foo: ToString, Baz: ToString, Derp: ToString>(foo: Foo, baz: Baz, derp: Derp) -> String {
    let val = _Example {foo: foo, bar: _Bar {baz: baz, herp: _Herp {derp: derp}}};
    templater::Context {obj: val}.render()
}


#[test]
fn test_str_string_int() {
    assert!(render("&str", "String", 98765u32).as_slice() ==
            "&str --- String --- 98765");
}
