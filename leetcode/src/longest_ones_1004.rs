use std::collections::VecDeque;

use crate::Solution;

impl Solution {
    pub fn longest_ones(nums: Vec<i32>, k: i32) -> i32 {
        let mut res = 0_i32;
        let mut queue = VecDeque::with_capacity(k as usize);
        let mut left = -1_i32;
        for (index, val) in nums.iter().enumerate() {
            let index = index as i32;
            if *val == 0 {
                if queue.len() == k as usize {
                    left = queue.pop_front().unwrap_or(index);
                }
                if k > 0 {
                    queue.push_back(index);
                }
            }
            res = res.max(index - left as i32);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::Solution;

    #[test]
    fn test_longest_ones() {
        for (arr, k, expect) in [
            (vec![1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0], 2, 6),
            (vec![1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0], 2, 6),
            (vec![1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0], 0, 4),
            (
                vec![0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1],
                3,
                10,
            ),
        ] {
            let res = Solution::longest_ones(arr, k);
            assert_eq!(res, expect);
        }
    }
}
