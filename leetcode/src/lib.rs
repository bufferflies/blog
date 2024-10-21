#![feature(let_chains)]
#![allow(clippy::all)]

use std::{cell::RefCell, fmt::Display, rc::Rc};
mod daily_temperatures;
mod is_symmetric_101;
mod least_bricks;
mod longest_substring;
mod reverse_list_206;
mod stock_span;
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
            write!(f, "({}, ", left.borrow())?;  // 打印左子树
        } else {
            write!(f, "(None, ")?;  // 如果左子树为空
        }
        
        // 打印当前节点的值
        write!(f, "{}", self.val)?;

        // 递归格式化右子树
        if let Some(ref right) = self.right {
            write!(f, ", {})", right.borrow())  // 打印右子树
        } else {
            write!(f, ", None)")  // 如果右子树为空
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

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
    fn  test_display_treenode() {
        let tree= crate::TreeNode {
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
        let str= tree.to_string();
        assert_eq!( "((None, 2, None), 1, (None, 3, None))", str,"str:{str}");
        
    }
}
