use crate::{ListNode, Solution};

impl Solution {
    pub fn reverse_list(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        let mut first = None;
        let mut cur = head;
        while let Some(mut node) = cur {
            cur = node.next;
            node.next = first;
            first = Some(node);
        }
        first
    }

    pub fn reverse_list_2(mut head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        if head.is_none() {
            return head;
        }

        let cur = &mut head;
        let mut dummy = ListNode::new(-1);
        while cur.is_some() {
            let first = dummy.next.take();
            let next = cur.as_mut().unwrap().next.take();
            dummy.next = cur.take();
            dummy.next.as_mut().unwrap().next = first;
            *cur = next;
        }
        dummy.next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_list_node() {
        for (arr, expect) in [
            (vec![1, 2, 3, 4, 5], "5->4->3->2->1"),
            (vec![1, 2, 3, 4], "4->3->2->1"),
            (vec![1, 2], "2->1"),
            (vec![1], "1"),
        ] {
            let list: ListNode = arr.into();
            let result = Solution::reverse_list(Some(Box::new(list)));
            assert_eq!(expect, result.unwrap().to_string());
        }
    }
}
