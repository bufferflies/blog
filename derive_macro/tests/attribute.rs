use derive_macro::trace;

#[trace(alias="attribute",properties={"key1":"value1","key2":"value2"})]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
pub fn test_attribute_function() {
    assert_eq!(3, add(1, 2));
    let mut res = "".to_owned();
    res.push_str(stringify!(add));
}
