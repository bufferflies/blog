use crate::{ListNode, Solution};

impl Solution {
    pub fn sort_list(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        if head.is_none() || head.as_ref().unwrap().next.is_none() {
            return head;
        }
        let (left, right) = Solution::split_list(head);
        let sorted_left = Solution::sort_list(left);
        let sorted_right = Solution::sort_list(right);
        Solution::merge_two_lists(sorted_left, sorted_right)
    }

    fn split_list(
        mut header: Option<Box<ListNode>>,
    ) -> (Option<Box<ListNode>>, Option<Box<ListNode>>) {
        let mut slow = &mut header as *mut Option<Box<ListNode>>;
        let mut fast = &header;
        while let Some(cur) = fast
            && let Some(next) = &cur.next
        {
            fast = &next.next;
            slow = unsafe {
                &mut slow.as_mut().unwrap().as_mut().unwrap().next as *mut Option<Box<ListNode>>
            };
        }
        let second = unsafe { slow.as_mut().unwrap().take() as Option<Box<ListNode>> };
        (header, second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_list() {
        let arr = vec![4, 3, 2, 1];
        let list: crate::ListNode = arr.into();
        assert_eq!(
            "1->2->3->4",
            Solution::sort_list(Some(Box::new(list)))
                .unwrap()
                .to_string()
        );
    }

    #[test]
    fn test_split_list() {
        for (arr, left, right) in [
            (vec![1, 2, 3, 4], "1->2", "3->4"),
            (vec![1, 2, 3], "1", "2->3"),
            (vec![1, 2], "1", "2"),
            (vec![1], "None", "1"),
        ] {
            let list: ListNode = arr.into();
            let (l, r) = Solution::split_list(Some(Box::new(list)));
            assert_eq!(right, r.unwrap().to_string());
            if left == "None" {
                assert!(l.is_none());
            } else {
                assert_eq!(left, l.unwrap().to_string());
            }
        }
    }
}
