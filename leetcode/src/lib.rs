#![feature(let_chains)]
#![allow(clippy::all)]

use std::{cell::RefCell, fmt::Display, rc::Rc};
mod can_jump_55;
mod daily_temperatures;
mod delete_middle_2095;
mod huawei;
mod is_symmetric_101;
mod least_bricks;
mod longest_ones_1004;
mod longest_substring;
mod lru_cache_146;
mod merge_56;
mod merge_two_lists_21;
mod num_islands_200;
mod permute_46;
mod remove_nth_from_end_19;
mod restore_ip_addresses_93;
mod reverse_list_206;
mod sort_list_148;
mod stock_span;
mod swap_pair_21;
mod three_num;

pub struct Solution {}

// Definition for a binary tree node.
#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }
}
impl Display for TreeNode {
    // 中序遍历
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 递归格式化左子树
        if let Some(ref left) = self.left {
            write!(f, "({}, ", left.borrow())?; // 打印左子树
        } else {
            write!(f, "(None, ")?; // 如果左子树为空
        }

        // 打印当前节点的值
        write!(f, "{}", self.val)?;

        // 递归格式化右子树
        if let Some(ref right) = self.right {
            write!(f, ", {})", right.borrow()) // 打印右子树
        } else {
            write!(f, ", None)") // 如果右子树为空
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl Display for ListNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut cur = self;
        while let Some(node) = &cur.next {
            write!(f, "{}->", cur.val)?;
            cur = node;
        }
        write!(f, "{}", cur.val)
    }
}

impl ListNode {
    #[inline]
    fn new(val: i32) -> Self {
        ListNode { next: None, val }
    }
}

impl From<Vec<i32>> for ListNode {
    fn from(v: Vec<i32>) -> Self {
        let mut head = Box::new(ListNode::new(0));
        let mut cur = &mut head;
        for i in v {
            cur.next = Some(Box::new(ListNode::new(i)));
            cur = cur.next.as_mut().unwrap();
        }
        *head.next.take().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc, vec};

    use crate::ListNode;

    #[test]
    fn test_from_vec() {
        let arr = vec![1, 2, 3];
        let list: ListNode = arr.into();
        assert_eq!(list.to_string(), "1->2->3");
    }

    #[test]
    fn test_display_listnode() {
        let list = crate::ListNode {
            val: 1,
            next: Some(Box::new(crate::ListNode {
                val: 2,
                next: Some(Box::new(crate::ListNode { val: 3, next: None })),
            })),
        };

        assert_eq!(list.to_string(), "1->2->3");
    }

    #[test]
    fn test_display_treenode() {
        let tree = crate::TreeNode {
            val: 1,
            left: Some(Rc::new(RefCell::new(crate::TreeNode {
                val: 2,
                left: None,
                right: None,
            }))),
            right: Some(Rc::new(RefCell::new(crate::TreeNode {
                val: 3,
                left: None,
                right: None,
            }))),
        };
        let str = tree.to_string();
        assert_eq!("((None, 2, None), 1, (None, 3, None))", str, "str:{str}");
    }
}
