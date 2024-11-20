use crate::{ListNode, Solution};

// Definition for singly-linked list.
// #[derive(PartialEq, Eq, Clone, Debug)]
// pub struct ListNode {
//   pub val: i32,
//   pub next: Option<Box<ListNode>>
// }
//
// impl ListNode {
//   #[inline]
//   fn new(val: i32) -> Self {
//     ListNode {
//       next: None,
//       val
//     }
//   }
// }
impl Solution {
    pub fn swap_pairs(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        let mut cur = head;
        let mut dummy = ListNode::new(0);
        let mut tail = &mut dummy;
        while let Some(mut node) = cur {
            cur = node.next.take(); // take()将n1打断，这样n1只有一个值，返回值是除n1节点外的剩余节点
            if let Some(mut next) = cur {
                cur = next.next.take(); // take()将n2打断，n2只有一个值，返回值是除n2节点外的剩余节点
                next.next = Some(node);
                tail.next = Some(next);
                tail = tail.next.as_mut().unwrap().next.as_mut().unwrap();
            } else {
                tail.next = Some(node);
                break;
            }
        }
        dummy.next
    }
}

#[cfg(test)]
mod tests {
    use crate::Solution;

    #[test]
    fn test_swap_paires() {
        let list = vec![1, 2, 3, 4].into();
        let ret = Solution::swap_pairs(Some(Box::new(list))).unwrap();
        assert_eq!("2->1->4->3", ret.to_string());
    }
}
