use crate::{ListNode, Solution};

impl Solution {
    pub fn reverse_list(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        let mut dummy = None;
        let mut cur = head;
        while let Some(mut node) = cur {
            cur = node.next;
            node.next = dummy;
            dummy = Some(node);
        }
        dummy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_1() {
        let mut head = ListNode::new(1);
        let mut node2 = ListNode::new(2);
        let mut node3 = ListNode::new(3);
        let mut node4 = ListNode::new(4);
        let node5 = ListNode::new(5);
        node4.next = Some(Box::new(node5));
        node3.next = Some(Box::new(node4));
        node2.next = Some(Box::new(node3));
        head.next = Some(Box::new(node2));
        let mut result = Solution::reverse_list(Some(Box::new(head)));
        while result.is_some() {
            println!("{}", result.as_ref().unwrap().val);
            result = result.unwrap().next;
        }
    }
}
