pub fn to_camel_case<S: Str + Sized>(src: S) -> String {
    let mut result = String::new();
    let mut make_upper = true;
    for ch in src.as_slice().chars() {
        match ch {
            '_' => {
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
}

// pub fn remove_extension<'a>(filename: &'a str) -> &'a str {
//     match filename.rfind('.') {
//         Some(index) => filename.slice_to(index),
//         None => filename
//     }
// }


// #[test]
// fn test_remove_extension() {
//     let a = remove_extension("foo.c");
//     assert!(a == "foo");

//     let b = remove_extension("fgsfds");
//     assert!(b == "fgsfds");

//     let c = remove_extension("a.b.c.d.e");
//     assert!(c == "a.b.c.d");
// }


// pub fn basename<'a>(filename: &'a str) -> &'a str {
//     match filename.rfind('/') {
//         Some(index) => filename.slice_to(index),
//         None => filename,
//     }
// }


// #[test]
// fn test_basename() {
//     let a = basename("/foo");
//     assert!(a == "foo");

//     let b = basename("fgsfds");
//     assert!(b == "fgsfds");

//     let c = basename("a/b/c/d/e");
//     assert!(c == "a/b/c/d");
// }
