use std::collections::HashMap;

use derive_macro::{HelloMacro, IntoHashMapDerive};

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
