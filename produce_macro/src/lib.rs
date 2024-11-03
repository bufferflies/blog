#[macro_export]
macro_rules! my_vec {
    () =>{
        Vec::new()
    };
    ($exp:expr;$n:expr) => {
        {
            std::vec::from_elem($exp,$n)
        }
    };
    ($($exp:expr),*) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($exp);
            )*
            temp_vec
        }
    };
    ($expr:expr)=>{
        $expr.to_vec()
    };
}

#[macro_export]
macro_rules! my_map {
    () => {
        std::collections::HashMap::new()
    };
    ($($key:expr=>$value:expr),*) => {
        {
            let mut temp_map = std::collections::HashMap::new();
            $(
                temp_map.insert($key,$value);
            )*
            temp_map
        }
    };
}
