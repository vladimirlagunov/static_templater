pub fn remove_extension<'a>(filename: &'a str) -> &'a str {
    match filename.rfind('.') {
        Some(index) => filename.slice_to(index),
        None => filename
    }
}


#[test]
fn test_remove_extension() {
    let a = remove_extension("foo.c");
    assert!(a == "foo");

    let b = remove_extension("fgsfds");
    assert!(b == "fgsfds");

    let c = remove_extension("a.b.c.d.e");
    assert!(c == "a.b.c.d");
}


pub fn basename<'a>(filename: &'a str) -> &'a str {
    match filename.rfind('/') {
        Some(index) => filename.slice_to(index),
        None => filename,
    }
}


#[test]
fn test_basename() {
    let a = basename("/foo");
    assert!(a == "foo");

    let b = basename("fgsfds");
    assert!(b == "fgsfds");

    let c = basename("a/b/c/d/e");
    assert!(c == "a/b/c/d");
}
