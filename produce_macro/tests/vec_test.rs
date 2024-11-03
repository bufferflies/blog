use std::vec;

#[test]
fn test_my_vec() {
    let arr = produce_macro::my_vec! {1,2,3};
    assert_eq!(vec![1, 2, 3], arr);
    let arr = produce_macro::my_vec!(1;3);
    assert_eq!(vec![1, 1, 1], arr);
}

#[test]
fn test_my_map() {
    let map = produce_macro::my_map! {"1"=>2,"3"=>4};
    let mut expect = std::collections::HashMap::new();
    expect.insert("1", 2);
    expect.insert("3", 4);
    assert_eq!(expect, map);
}
