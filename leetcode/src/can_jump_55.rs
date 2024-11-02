use crate::Solution;

impl Solution {
    pub fn can_jump(nums: Vec<i32>) -> bool {
        let mut max_right = 0;
        for (index, val) in nums.iter().enumerate() {
            if index <= max_right {
                max_right = max_right.max(index + (*val as usize));
            }
        }
        max_right >= nums.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::Solution;

    #[test]
    fn test_can_jump() {
        for (nums, expect) in [(vec![2, 3, 1, 1, 4], true), (vec![3, 2, 1, 0, 4], false)] {
            let res = Solution::can_jump(nums);
            assert_eq!(res, expect);
        }
    }
}