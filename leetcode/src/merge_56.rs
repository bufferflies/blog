use crate::Solution;

impl Solution {
    pub fn merge(mut intervals: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
        let mut res: Vec<Vec<i32>> = Vec::new();
        intervals.sort_by_key(|a| a.first().unwrap().clone());
        for interval in intervals {
            match res.last_mut() {
                Some(last) => {
                    if last[1] >= interval[0] {
                        last[1] = last[1].max(interval[1]);
                    } else {
                        res.push(interval);
                    }
                }
                None => {
                    res.push(interval);
                }
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::Solution;

    //[[1,3],[2,6],[8,10],[15,18]]
    #[test]
    fn test_merge() {
        for (arr, expect) in [
            (
                vec![vec![1, 3], vec![2, 6], vec![8, 10], vec![15, 18]],
                vec![vec![1, 6], vec![8, 10], vec![15, 18]],
            ),
            (vec![vec![1, 4], vec![0, 4]], vec![vec![0, 4]]),
            (vec![vec![1, 4], vec![2, 3]], vec![vec![1, 4]]),
        ] {
            let res = Solution::merge(arr);
            assert_eq!(res, expect);
        }
    }
}
