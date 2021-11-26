use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, Promise};

use std::{collections::HashMap, collections::HashSet,io::StderrLock};
use std::fs::read_to_string;
use near_sdk::collections::LookupMap;

const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Voting {
    votes_received: HashMap<String, i32>,
    //aha : LookupMap<String,i32>
    voters : HashSet<String>,
    voter_result : HashMap<String, String>,
    voter_balance : HashMap<String, u128>,
    sum_pool : u128,
    is_lock : bool,
    winner : String,
    title : String,
    durance : u64,
}

#[near_bindgen]
impl Voting {
    #[init]
    pub fn new(title : String,time : u32) -> Self {
        let mut nowTime = env::block_timestamp();
        nowTime += time as u64 * 1000000000 *60;
        Self {
            votes_received: HashMap::new(),
            voters : HashSet::new(),
            voter_result :HashMap::new(),
            voter_balance :HashMap::new(),
            is_lock:false,
            winner:"who will be the winner?".to_string(),
            title : title,
            durance : nowTime,
            sum_pool : 0,
        }
    }

    pub fn check_out_of_time(&mut self) -> bool{
        if env::block_timestamp() > self.durance{
            self.is_lock = true;
            let mut max = -1;
            let mut name= "";
            for (candidate, votes) in self.votes_received.iter() {
                if votes > &max {
                    max = *votes;
                    name = candidate;
                }
            }
            self.winner = name.to_string();
            self.transfer_winner();
            return true;
        }
        return false;
    }

    pub fn get_title(&self) ->String{
        return self.title.to_string();
    }

    pub fn add_candidate(&mut self, candidate: String) -> String{
        if self.check_out_of_time(){
            return "the voting is over, please use get_winner function to check!".to_string();
        }
        if self.is_lock {
            return "the voting is over, please use get_winner function to check!".to_string();
        }
         self.votes_received.insert(candidate, 0);
        return "Success to add the candidate!".to_string();
    }

    pub fn get_total_votes_for(self, name: String) -> Option::<i32> {
        if !self.valid_candidate(&name) {
            ()
        }
        self.votes_received.get(&name).cloned()
    }

    pub fn vote_without_near(&mut self, name: String) -> String{
        if self.check_out_of_time(){
            return "the voting is over, please use get_winner function to check!".to_string();
        }
        let mut z2 = String::new();
        z2.push_str(&name);
        if !self.valid_candidate(&name) {
            return "before vote, please add the candidate".to_string();
        }
        let result = self.voters.insert(env::predecessor_account_id());
        if !result{
            return "you have voted, anyone only have one time".to_string();
        }
        if self.is_lock {
            return "the voting is over, please use get_winner function to check!".to_string();
        }else {
            let counter = self.votes_received.entry(name).or_insert(0);
            *counter += 1;
            self.voter_result.insert(env::predecessor_account_id(),z2.to_string());
            return "success to vote!".to_string();
        }
    }

    #[payable]
    pub fn vote_with_near(&mut self, name: String) -> String{
        if self.check_out_of_time(){
            return "the voting is over, please use get_winner function to check!".to_string();
        }
        if !self.valid_candidate(&name) {
            return "before vote, please add the candidate".to_string();
        }
        let result = self.voters.insert(env::predecessor_account_id());
        if !result{
            return "you have voted, anyone only have one time".to_string();
        }
        let mut z1 = String::new();
        z1.push_str(&name);
        let mut z2 = String::new();
        z2.push_str(&name);
        if self.is_lock {
            return "the voting is over, please use get_winner function to check!".to_string();
        }else {
            let mut deposit = env::attached_deposit();
            if deposit < ONE_NEAR{
                return "please vote with the deposit over one near".to_string();
            }else{
                let num = deposit / ONE_NEAR;
                let counter = self.votes_received.entry(name).or_insert(0);
                *counter += num as i32 + 1;
                let  balance = self.voter_balance.entry(z1).or_insert(0);
                *balance += deposit;
                self.voter_result.insert(env::predecessor_account_id(),z2.to_string());
                self.sum_pool += deposit;
                return "thx for your near, your candidate will get more support :)".to_string();
            }
        }
    }

    pub fn transfer_winner(&self){
        let mut winnerVec:HashSet<String> = HashSet::new();
        if self.is_lock{
            for (voter, candidate) in self.voter_result.iter() {
                if candidate.eq(&self.winner){
                    winnerVec.insert(voter.to_string());
                }
            }
        }
        let mut eachNum = self.sum_pool / (winnerVec.len() as u128);
        for (voter) in winnerVec.iter(){
            Promise::new(voter.to_string()).transfer(eachNum);
        }


    }

    pub fn get_tmp_winner(&self) -> String{
        let mut max = -1;
        let mut name= "";
        for (candidate, votes) in self.votes_received.iter() {
           if votes > &max {
               max = *votes;
               name = candidate;
           }
        }
        return name.to_string();
    }

    pub fn check_my_vote(&mut self) -> String{
        let mut id = env::predecessor_account_id();
        return self.voter_result.get(&*id).unwrap().clone();
    }



    pub fn lock(&mut self){
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "only the contract owner could end the voting"
        );
        self.is_lock = true;
        let mut max = -1;
        let mut name= "";
        for (candidate, votes) in self.votes_received.iter() {
           if votes > &max {
               max = *votes;
               name = candidate;
           }
        }
        self.winner = name.to_string();
        self.transfer_winner();
    }

    pub fn valid_candidate(&self, name: &String) -> bool {
        for (candidate, votes) in self.votes_received.iter() {
            if self.votes_received.contains_key(name) {
                return true
            }
        }
        false
    }

    pub fn restart(&mut self,title:String,time:u32){
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "only the contract owner could end the voting"
        );
        let mut nowTime = env::block_timestamp();
        nowTime += time as u64 * 1000000000 *60;
        self.votes_received=HashMap::new();
        self.voters = HashSet::new();
        self.voter_result =HashMap::new();
        self.voter_balance =HashMap::new();
        self.is_lock=false;
        self.winner="who will be the winner?".to_string();
        self.title = title;
        self.durance = nowTime;
        self.sum_pool =0;
    }
    
    pub fn get_candidates(&self) -> HashMap<String, i32> {
        self.votes_received.clone()
    }
    pub fn get_voters(&self) -> HashSet<String> {
        self.voters.clone()
    }
}

// #[cfg(not(target_arch = "wasm32"))]
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_bindgen::MockedBlockchain;
//     use near_bindgen::{testing_env, VMContext};

//     fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
//         VMContext {
//             current_account_id: "alice_near".to_string(),
//             signer_account_id: "bob_near".to_string(),
//             signer_account_pk: vec![0, 1, 2],
//             predecessor_account_id: "carol_near".to_string(),
//             input,
//             block_index: 0,
//             block_timestamp: 0,
//             account_balance: 0,
//             account_locked_balance: 0,
//             storage_usage: 0,
//             attached_deposit: 0,
//             prepaid_gas: 10u64.pow(18),
//             random_seed: vec![0, 1, 2],
//             is_view,
//             output_data_receivers: vec![],
//         }
//     }

//     #[test]
//     fn test_add_candidate() {
//         let context = get_context(vec![], false);
//         testing_env!(context);
//         let mut contract = Voting::new();
//         contract.add_candidate("Jeff".to_string());
//         assert_eq!(0, contract.get_total_votes_for("Jeff".to_string()).unwrap());
//     }

//     #[test]
//     fn test_get_total_votes_for() {
//         let context = get_context(vec![], true);
//         testing_env!(context);
//         let contract = Voting::new();
//         assert_eq!(None, contract.get_total_votes_for("Anna".to_string()));
//     }
// }