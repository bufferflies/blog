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

    pub fn max_path_sum(root: Option<Rc<RefCell<TreeNode>>>) -> i32 {
        let mut res = i32::MIN;
        Solution::max_path(root, &mut res);
        res
    }

    fn max_path(root: Option<Rc<RefCell<TreeNode>>>, max: &mut i32) -> i32 {
        if root.is_none() {
            return 0;
        }
        let root_ref = root.unwrap();
        let val = root_ref.as_ref().borrow().val.clone();
        let left = Solution::max_path(root_ref.as_ref().borrow().left.clone(), max).max(0);
        let right = Solution::max_path(root_ref.as_ref().borrow().right.clone(), max).max(0);
        let mut current_path_sum = val + left + right;
        *max = *max.max(&mut current_path_sum);
        val + left.max(right)
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
