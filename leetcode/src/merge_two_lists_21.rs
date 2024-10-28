use crate::{ListNode, Solution};

impl Solution {
    pub fn merge_two_lists(
        mut list1: Option<Box<ListNode>>,
        mut list2: Option<Box<ListNode>>,
    ) -> Option<Box<ListNode>> {
        let mut dummy = ListNode::new(-1);
        let mut cur = &mut dummy;
        while let (Some(node1), Some(node2)) = (list1.as_ref(), list2.as_ref()) {
            let l = if node1.val <= node2.val {
                &mut list1
            } else {
                &mut list2
            };
            cur.next = l.take();
            // 向前移动
            cur = cur.next.as_mut().unwrap();
            *l = cur.next.take();
        }
        cur.next = list1.or(list2);
        dummy.next
    }
}

#[cfg(test)]
mod tests {
    use crate::{ListNode, Solution};

    #[test]
    fn test_merge_list() {
        let l1 = Some(Box::new(ListNode::new(0)));
        let l2 = Some(Box::new(ListNode::new(2)));
        let l = Solution::merge_two_lists(l1, l2);
        assert_eq!(l.unwrap().to_string(), "0->2");
    }
}
