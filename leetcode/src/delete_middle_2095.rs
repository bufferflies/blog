use crate::{ListNode, Solution};

impl Solution {
    pub fn delete_middle(mut head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        let mut slow = &mut head as *mut Option<Box<ListNode>>;
        let mut fast = &head;
        while let Some(cur) = fast
            && let Some(next) = &cur.next
        {
            fast = &next.next;
            slow = unsafe {
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
        };
        head
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_delete_middle() {
        for (arr, expect) in [
            (vec![1, 2, 3, 4, 5], "1->2->4->5"),
            (vec![1, 2, 3, 4], "1->2->4"),
            (vec![2, 1], "2"),
            (vec![1], ""),
        ] {
            let list: ListNode = arr.into();
            let result = Solution::delete_middle(Some(Box::new(list)));
            assert_eq!(
                expect,
                result.map_or("".to_owned(), |x| x.to_string()).as_str()
            );
        }
    }
}
