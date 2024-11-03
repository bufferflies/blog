use std::collections::HashMap;

fn main() {
    HelloStruct::hello();
    let person = HelloStruct {
        name: "hello".to_string(),
        age: 12,
    };
    let map: HashMap<String, String> = person.into();
    println!("{:?}", map);
}

#[derive(derive_macro::HelloMacro, derive_macro::IntoHashMapDerive)]
struct HelloStruct {
    name: String,
    pub age: i32,
}

trait HelloTrait {
    fn hello();
}
