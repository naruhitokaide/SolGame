use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod constants;

use instructions::*;

declare_id!("BoB8rikXuQDYqDet9YSpqYmXcCBYiRxPSeLW7iXz8VdG");

#[program]
pub mod round {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, slot_token_price: u64, fee: u64,) -> Result<()> {
        instructions::initialize(ctx,slot_token_price, fee)
    }

    pub fn update_fee(ctx: Context<Update>, new_fee: u64) -> Result<()> {
        instructions::update_fee(ctx,new_fee)
    }

    pub fn create_round(ctx: Context<CreateRound>, round_index: u32) -> Result<()> {
        instructions::create_round(ctx,round_index)
    }

    pub fn buy_slot(ctx: Context<BuySlot>, round_index: u32, amount: u64) -> Result<()> {
        instructions::buy_slot(ctx,round_index, amount)
    }

    pub fn claim_slot(ctx: Context<ClaimSlot>) -> Result<()> {
        instructions::claim_slot(ctx)
    }

    pub fn withdraw_sol(ctx: Context<WithDrawSOL>, amount: u64) -> Result<()> {
        instructions::withdraw_sol(ctx, amount)
    }

    pub fn withdraw_sol(ctx: Context<WithDrawSOL>, amount: u64) -> Result<()> {
        instructions::withdraw_sol(ctx, amount)
    }
}
