use std::{cell::RefCell, rc::Rc};

use crate::{Solution, TreeNode};

impl Solution {
    pub fn is_symmetric(root: Option<Rc<RefCell<TreeNode>>>) -> bool {
        if root.is_none() {
            return true;
        }

        is_same(
            root.clone().unwrap().borrow().left.clone(),
            root.unwrap().borrow().right.clone(),
        )
    }
}

fn is_same(left: Option<Rc<RefCell<TreeNode>>>, right: Option<Rc<RefCell<TreeNode>>>) -> bool {
    match (left, right) {
        (Some(lhs), Some(rhs)) => {
            let lhs = lhs.borrow();
            let rhs = rhs.borrow();
            is_same(lhs.left.clone(), rhs.right.clone())
                && is_same(lhs.right.clone(), rhs.left.clone())
                && lhs.val == rhs.val
        }
        (Some(_), None) | (None, Some(_)) => false,
        (None, None) => true,
    }
}

mod tests {
    #[test]
    fn test_1() {
        assert_eq!(crate::Solution::is_symmetric(None), true);
    }
}
