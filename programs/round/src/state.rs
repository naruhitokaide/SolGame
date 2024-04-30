use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct GlobalState {
    pub owner: Pubkey, 
    pub total_round: u32,
    pub slot_token_price: u64,
    pub vault: Pubkey,
    pub fee: u64,
}

#[account]
#[derive(Default)]
pub struct RoundState {
    pub round_index: u32,
    pub total_slot_number: u64,
    pub current_slot_number: u64,
}


#[account]
#[derive(Default)]
pub struct UserInfo {
    pub total_slot_number: u64,
    pub last_slot_number: u64,
    pub last_round_index: u32,
    pub claimed_slot_number: u64,
}