use crate::{ListNode, Solution};

impl Solution {
    pub fn remove_nth_from_end(mut head: Option<Box<ListNode>>, n: i32) -> Option<Box<ListNode>> {
        let mut slow = &mut head as *mut Option<Box<ListNode>>;
        let mut fast = &head;
        for _ in 0..n {
            fast = &fast.as_ref().unwrap().next;
        }
        while let Some(next) = fast {
            fast = &next.next;
            unsafe {
                slow =
                    &mut slow.as_mut().unwrap().as_mut().unwrap().next as *mut Option<Box<ListNode>>
            };
        }
        unsafe {
            match slow.as_mut().unwrap().as_mut().unwrap().next.take() {
                Some(next) => {
                    slow.as_mut().unwrap().replace(next);
                }
                None => {
                    slow.as_mut().unwrap().take();
                }
            }
        }
        head
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_nth_from_end() {
        for (arr, n, expect) in [
            (vec![1, 2, 3, 4, 5], 2, "1->2->3->5"),
            // (vec![1,2,3,4],1,"1->2->3->4"),
            // (vec![1,2,3,4],4,"2->3->4"),
            // (vec![1,2],1,"1"),
            // (vec![1],1,""),
        ] {
            let list: crate::ListNode = arr.into();
            let result = crate::Solution::remove_nth_from_end(Some(Box::new(list)), n);
            assert_eq!(expect, result.unwrap().to_string());
        }
    }
}
