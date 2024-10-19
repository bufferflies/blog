use crate::Solution;

impl Solution {
    pub fn daily_temperatures(temperatures: Vec<i32>) -> Vec<i32> {
        let len = temperatures.len();
        let mut ret: Vec<i32> = vec![0; len];
        let mut stack = vec![];
        for (index, temperature) in temperatures.iter().enumerate() {
            while let Some((top_index, top_temperature)) = stack.last()
                && temperature > top_temperature
            {
                ret[*top_index] = (index - top_index) as i32;
                stack.remove(stack.len() - 1);
            }
            stack.push((index, *temperature));
        }
        ret
    }
}

mod tests {
    #[test]
    fn test_1() {
        assert_eq!(
            crate::Solution::daily_temperatures(vec![73, 74, 75, 71, 69, 72, 76, 73]),
            vec![1, 1, 4, 2, 1, 1, 0, 0]
        );
    }
}
