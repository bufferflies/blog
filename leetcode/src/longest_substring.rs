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

    pub fn coin_change(coins: Vec<i32>, amount: i32) -> i32 {
        if amount <= 0 {
            return 0;
        }
        let mut dp = vec![i32::MAX; amount as usize + 1];
        dp[0] = 0;
        for i in 0..amount + 1 {
            let index = i as usize;
            for val in coins.iter() {
                if val <= &i {
                    let offset = index - *val as usize;
                    if dp[offset] == i32::MAX {
                        continue;
                    }
                    dp[index] = dp[index].min(dp[offset] + 1)
                }
            }
        }
        dp[amount as usize]
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

    #[test]
    fn test_coin_change() {
        assert_eq!(Solution::coin_change(vec![1, 2, 5], 11), 3);
    }
}
