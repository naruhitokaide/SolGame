use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct GlobalState {
    pub owner: Pubkey, 
    pub total_round: u32,
    pub slot_token_price: u64,
    pub vault: Pubkey
}

#[account]
#[derive(Default)]
pub struct Round {
    pub round_index: u32,
    pub total_slot_number: u64,
    pub current_slot_number: u64,
    pub status: bool, // true: processing false: finish
}

#[account]
#[derive(Default)]
pub struct RoundUserInfo {
    pub round_index: u32,
    pub buy_slot_number: u64,
    pub claimed: bool,
}