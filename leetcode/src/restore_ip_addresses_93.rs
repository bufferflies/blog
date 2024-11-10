use crate::Solution;

impl Solution {
    pub fn restore_ip_addresses(s: String) -> Vec<String> {
        let mut res = vec![];
        let mut tmp = vec![];
        Solution::dfs(&s, 0, 0, &mut res, &mut tmp);
        res
    }

    fn dfs(
        s: &String,
        index: usize,
        sep_count: usize,
        res: &mut Vec<String>,
        arr: &mut Vec<String>,
    ) {
        if sep_count == 4 {
            if index < s.len() {
                return;
            }
            if arr.len() == 4 {
                let ip = arr.join(".");
                res.push(ip);
            }
        }
        for i in index + 1..s.len() + 1 {
            let seq = s[index..i].to_owned();
            if seq.len() > 1 && &seq[0..1] == "0" {
                continue;
            }
            if let Ok(num) = seq.parse::<i32>() {
                if num > 255 {
                    continue;
                }
            } else {
                continue;
            }
            arr.push(seq);
            Solution::dfs(s, i, sep_count + 1, res, arr);
            arr.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let s = String::from("1231231231234");
        let res = Solution::restore_ip_addresses(s);
        assert_eq!(vec!["0.0.0.0"], res);
    }
}
