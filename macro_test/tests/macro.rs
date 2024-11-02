use std::collections::HashMap;

use macro_test::{HelloMacro, IntoHashMapDerive};

trait HelloTrait {
    fn hello();
}

#[derive(HelloMacro, IntoHashMapDerive)]
struct HelloStruct {
    name: String,
    pub age: i32,
}

#[test]
fn test_derive() {
    HelloStruct::hello();
    let person = HelloStruct {
        name: "hello".to_string(),
        age: 12,
    };
    let map: HashMap<String, String> = person.into();
    println!("{:?}", map);
}

#[test]
fn test_from_vec() {
    let arr = vec![1, 2, 3];
    let list: ListNode = arr.into();
    assert_eq!(list.to_string(), "1->2->3");
}
