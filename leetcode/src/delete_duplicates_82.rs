use crate::{ListNode, Solution};

impl Solution {
    pub fn delete_duplicates(mut head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        let mut res = ListNode::new(0);
        let mut p = &mut res;
        let mut pre = 101;
        while let Some(mut cur) = head {
            // 删除 next 节点，并且 head 后移动
            head = cur.next.take();
            //如果当前访问的值与下一个值相等或与上一个值相等，则当前值不加进去。
            if (head.is_some() && head.as_ref().unwrap().val == cur.val)
                || cur.val == pre {
                pre = cur.val;
            } else {
                pre = cur.val;
                p.next = Some(cur);
                p = p.next.as_mut().unwrap();
            }
        }
        res.next
    }

    pub fn delete_duplicates_1(mut head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        let mut res = ListNode::new(0);
        let mut p = &mut res;
        while let Some(mut cur) = head {
            // 删除 next 节点，并且 head 后移动
            head = cur.next.take();
            //如果当前访问的值与下一个值相等或与上一个值相等，则当前值不加进去。
            if head.is_some() && head.as_ref().unwrap().val == cur.val{
                continue;
            } else {
                p.next = Some(cur);
                p = p.next.as_mut().unwrap();
            }
        }
        res.next
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn delete_duplicates() {
        for (arr, expect) in [(vec![1, 2, 3, 3, 4, 4, 5], "1->2->5")] {
            let list: crate::ListNode = arr.into();
            let result = crate::Solution::delete_duplicates(Some(Box::new(list)));
            assert_eq!(expect, result.unwrap().to_string());
        }
    }

    #[test]
    fn delete_duplicates_1() {
        for (arr, expect) in [(vec![1, 2, 3, 3, 4, 4, 5], "1->2->3->4->5")] {
            let list: crate::ListNode = arr.into();
            let result = crate::Solution::delete_duplicates_1(Some(Box::new(list)));
            assert_eq!(expect, result.unwrap().to_string());
        }
    }
}
