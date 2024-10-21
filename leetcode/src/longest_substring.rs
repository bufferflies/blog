use crate::Solution;

impl Solution {
    pub fn length_of_longest_substring(s: String) -> i32 {
        use std::collections::HashMap;
        let mut dict = HashMap::new();
        let mut begin = -1_i32;
        let mut max = 0_i32;
        for (i, c) in s.as_bytes().iter().enumerate() {
            if let Some(index) = dict.get(c)
                && index >= &begin
            {
                begin = *index;
            }
            dict.insert(c, i as i32);
            max = max.max(i as i32 - begin);
        }
        max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        print!("test_1\n");
        assert_eq!(
            Solution::length_of_longest_substring("abcabcbb".to_string()),
            3
        );
    }

    #[test]
    fn test_2() {
        assert_eq!(
            Solution::length_of_longest_substring("bbbbb".to_string()),
            1
        );
    }

    #[test]
    fn test_3() {
        assert_eq!(
            Solution::length_of_longest_substring("pwwkew".to_string()),
            3
        );
    }

    #[test]
    fn test_4() {
        assert_eq!(Solution::length_of_longest_substring("".to_string()), 0);
    }
}
