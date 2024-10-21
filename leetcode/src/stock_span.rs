use crate::Solution;

struct StockSpanner {
    // (price, index)
    desc_stack: Vec<(i32, i32)>,
    count: i32,
}

/**
 * `&self` means the method takes an immutable reference.
 * If you need a mutable reference, change it to `&mut self` instead.
 */
impl StockSpanner {
    fn new() -> Self {
        Self {
            desc_stack: vec![],
            count: 0,
        }
    }

    fn next(&mut self, price: i32) -> i32 {
        let mut span = 1;
        self.count += 1;
        while let Some((top_price, top_index)) = self.desc_stack.last() {
            if &price >= top_price {
                self.desc_stack.pop();
            } else {
                span = self.count - top_index;
                break;
            }
        }

        self.desc_stack.push((price, self.count));
        span
    }
}

const fn is_single_bracket(left: u8, right: u8) -> bool {
    match left {
        b'(' => right == b')',
        b'{' => right == b'}',
        b'[' => right == b']',
        _ => false,
    }
}

impl Solution {
    pub fn is_valid(s: String) -> bool {
        let mut left_stack = vec![];
        for c in s.as_bytes().iter() {
            match c {
                b'(' | b'{' | b'[' => left_stack.push(c),
                b')' | b'}' | b']' => {
                    if left_stack.is_empty() {
                        return false;
                    }
                    let valid = is_single_bracket(*left_stack.pop().unwrap(), *c);
                    if !valid {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        left_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_1() {
        let mut obj = StockSpanner::new();
        assert_eq!(obj.next(100), 1);
        assert_eq!(obj.next(80), 1);
        assert_eq!(obj.next(60), 1);
        assert_eq!(obj.next(70), 2);
        assert_eq!(obj.next(60), 1);
        assert_eq!(obj.next(75), 4);
        assert_eq!(obj.next(85), 6);
    }
}
