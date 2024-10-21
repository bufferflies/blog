use crate::Solution;

impl Solution {
    pub fn least_bricks(walls: Vec<Vec<i32>>) -> i32 {
        use std::collections::HashMap;
        let mut count_map: HashMap<i32, i32> = HashMap::default();
        for wall in walls.iter() {
            let mut acc = 0;
            for (index, row) in wall.iter().enumerate() {
                if index == wall.len() - 1 {
                    continue;
                }
                acc += row;
                *count_map.entry(acc).or_default() += 1
            }
        }
        walls.len() as i32 - count_map.values().max().unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_1() {
        assert_eq!(
            Solution::least_bricks(vec![
                vec![1, 2, 2, 1],
                vec![3, 1, 2],
                vec![1, 3, 2],
                vec![2, 4],
                vec![3, 1, 2],
                vec![1, 3, 1, 1]
            ]),
            2
        );
    }

    #[test]
    fn test_2() {
        assert_eq!(
            Solution::least_bricks(vec![vec![1, 1], vec![2], vec![1, 1],]),
            1
        );
    }
}
