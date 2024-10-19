use crate::Solution;

impl Solution {
    pub fn three_sum(mut nums:Vec<i32>) -> Vec<Vec<i32>> {
        nums.sort();
        let mut ret = vec![];
        for (index,key) in nums.iter().enumerate(){
            if index>0&&nums[index]==nums[index-1]{
                continue;
            }
            let mut left=index+1;
            let mut right=nums.len()-1;
            while left<right{
                let sum=nums[left]+nums[right]+key;
                if sum==0{
                    ret.push(vec![*key,nums[left],nums[right]]);
                    while left<right && nums[left]==nums[left+1]{
                        left+=1;
                    }
                    while left<right && nums[right]==nums[right-1]{
                        right-=1;
                    }
                    left+=1;
                    right-=1;
                }else if sum<0{
                    left+=1;
                }else{
                    right-=1;
                }
            }
        }
        ret
    }
}