use crate::Solution;

impl Solution {
    pub fn num_islands(mut grid: Vec<Vec<char>>) -> i32 {
        if grid.is_empty() {
            return 0;
        }

        let mut res = 0;
        fn dfs(i: usize, j: usize, grid: &mut Vec<Vec<char>>) {
            let (row, col) = (grid.len(), grid.first().unwrap().len());
            if i >= row || j >= col {
                return;
            }
            if grid[i][j] == '0' {
                return;
            }
            grid[i][j] = '0';
            dfs(i + 1, j, grid);
            dfs(i, j + 1, grid);
            if i != 0 {
                dfs(i - 1, j, grid);
            }
            if j != 0 {
                dfs(i, j - 1, grid);
            }
        }

        let (row, col) = (grid.len(), grid.first().unwrap().len());
        for i in 0..row {
            for j in 0..col {
                if grid[i][j] == '1' {
                    res += 1;
                }
                dfs(i, j, grid.as_mut());
            }
        }
        return res;
    }
}

#[cfg(test)]
mod tests {
    use crate::Solution;

    #[test]
    fn test_num_islands() {
        for (grid, expect) in [
            (
                [
                    ['1', '1', '1', '1', '0'].to_vec(),
                    ['1', '1', '0', '1', '0'].to_vec(),
                    ['1', '1', '0', '0', '0'].to_vec(),
                    ['0', '0', '0', '0', '0'].to_vec(),
                ],
                1,
            ),
            (
                [
                    ['1', '1', '0', '0', '0'].to_vec(),
                    ['1', '1', '0', '0', '0'].to_vec(),
                    ['0', '0', '1', '0', '0'].to_vec(),
                    ['0', '0', '0', '1', '1'].to_vec(),
                ],
                3,
            ),
        ] {
            let res = Solution::num_islands(grid.to_vec());
            assert_eq!(res, expect)
        }
    }
}
