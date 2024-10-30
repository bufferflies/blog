use crate::Solution;

impl Solution {
    pub fn permute(nums: Vec<i32>) -> Vec<Vec<i32>> {
        let mut res: Vec<Vec<i32>> = Vec::new();
        let mut visited = vec![false; nums.len()];
        let mut arr = Vec::new();
        dfs(&nums, &mut arr, &mut res, &mut visited);
        res
    }
}

fn dfs(nums: &Vec<i32>, arr: &mut Vec<i32>, res: &mut Vec<Vec<i32>>, visited: &mut Vec<bool>) {
    if nums.len() == arr.len() {
        res.push(arr.clone());
        return;
    }
    for i in 0..nums.len() {
        if visited[i] {
            continue;
        }
        visited[i] = true;
        arr.push(nums[i]);
        dfs(nums, arr, res, visited);
        visited[i] = false;
        arr.pop();
    }
}

#[cfg(test)]
mod tests {
    use crate::Solution;

    #[test]
    fn test_permute() {
        for (arr, expect) in [(vec![1, 2, 3], vec![
            vec![1, 2, 3],
            vec![1, 3, 2],
            vec![2, 1, 3],
            vec![2, 3, 1],
            vec![3, 1, 2],
            vec![3, 2, 1],
        ])] {
            let res = Solution::permute(arr);
            assert_eq!(res, expect);
        }
    }
}
