// Xuming x00805572

pub fn is_sub_string(s1: String, s2: String) -> bool {
    let arr1 = count_char(s1.as_str());
    let mut arr2 = count_char(&s2[0..s1.len()]);
    if is_equal(&arr1, &arr2) {
        return true;
    }
    let s2 = s2.as_bytes();
    for (index, value) in s2[s1.len()..].iter().enumerate() {
        let pre = s2[index] as usize - 'a' as usize;
        arr2[pre] -= 1;
        let pos = *value as usize - 'a' as usize;
        arr2[pos] += 1;
        if is_equal(&arr1, &arr2) {
            return true;
        }
    }
    false
}

fn is_equal(arr1: &Vec<usize>, arr2: &Vec<usize>) -> bool {
    for (index, value) in arr1.iter().enumerate() {
        if *value == 0 {
            continue;
        }
        if let Some(target_count) = arr2.get(index)
            && target_count != value
        {
            return false;
        }
    }
    true
}

fn count_char(s1: &str) -> Vec<usize> {
    let mut arr = vec![0; 26];
    for c in s1.as_bytes().iter() {
        let index = *c as usize - 'a' as usize;
        arr[index] += 1;
    }
    arr
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_sub_string() {
        for (s1, s2, expect) in [
            ("ab".to_owned(), "eidbaooo".to_owned(), true),
            ("ab".to_owned(), "eidboaoo".to_owned(), false),
            ("aab".to_owned(), "aba".to_owned(), true),
        ] {
            let result = super::is_sub_string(s1.clone(), s2.clone());
            assert_eq!(result, expect, "s1:{s1:?},s2:{s2:?}");
        }
    }
}
