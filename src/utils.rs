pub fn to_camel_case<S: Str + Sized>(src: S) -> String {
    let mut result = String::new();
    let mut make_upper = true;
    for ch in src.as_slice().chars() {
        match ch {
            '_' => {
                make_upper = true;
            },
            ch if ch >= '0' && ch <= '9' => {
                result.push(ch);
                make_upper = true;
            },
            ch => {
                if make_upper {
                    result.push(ch.to_uppercase());
                } else {
                    result.push(ch.to_lowercase());
                }
                make_upper = false;
            }
        };
    }
    result
}


#[test]
fn test_to_camel_case() {
    let s: &str = "foo_bar_baz";
    assert!(to_camel_case(s) == "FooBarBaz");

    let s: String = "foo_bar_baz".to_string();
    assert!(to_camel_case(s) == "FooBarBaz");

    assert!(to_camel_case("AbCdEf") == "Abcdef");
    assert!(to_camel_case("AbC_dEf") == "AbcDef");

    assert!(to_camel_case("str2int_type") == "Str2IntType");
}
