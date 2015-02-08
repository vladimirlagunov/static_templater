#![feature(plugin)]
#[plugin] extern crate static_templater;

use std::collections::HashMap;


// #[static_templater]
// mod templater {
//     const SOURCE: &'static str = 
//         "{{ str2int[\"foo\"] }} + {{ str2int[\"bar\"] }} + {{ int2int[1] }} + {{ int2str[0] }}";
// }


// #[test]
// fn test_hashmap_vector() {
//     let args = templater::Args {
//         str2int: vec![("foo", 123), ("bar", 456)].into_iter().collect::<HashMap<&'static str, u64>>(),
//         int2int: vec![5, 7, 9],
//         int2str: vec!["zero", "one", "two"],
//     };
//     let result = templater::render(args);
//     assert!(result.as_slice() == "123 + 456 + 7 + zero");
// }
