use crate::{constants::*, errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use solana_program::{program::{invoke, invoke_signed}, system_instruction};

use std::mem::size_of;

pub fn initialize(ctx: Context<Initialize>, slot_token_price: u64) -> Result<()> {
    let accts = ctx.accounts;

    // init the global state account
    accts.global_state.owner = accts.owner.key();
    accts.global_state.total_round = 0;
    accts.global_state.slot_token_price = slot_token_price;
    accts.global_state.vault = accts.vault.key();

    Ok(())
}

pub fn create_round(ctx: Context<CreateRound>, round_index: u32) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.global_state.owner == accts.owner.key(), RoundError::NotAllowedOwner);
    require!(accts.global_state.total_round + 1 == round_index, RoundError::InvalidRoundIndex) ;

    let current_index = accts.global_state.total_round;
    // create the new round
    accts.round.round_index = current_index + 1;
    accts.round.total_slot_number = 2_u64.pow(round_index - 1);
    accts.round.current_slot_number = 0;
    accts.round.status = true;
    // update the global state
    accts.global_state.total_round += 1;

    Ok(())
}

pub fn buy_slot(ctx: Context<BuySlot>, round_index: u32, amount: u64) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.round.status == true, RoundError::AlreadyFinish);

    let mut amount = amount;
    
    if accts.round.current_slot_number + amount <= accts.round.total_slot_number {
        if accts.round.current_slot_number < accts.round.total_slot_number {
            amount = accts.round.total_slot_number - accts.round.current_slot_number;
            accts.round.status = false;
        } else {
            return Err(RoundError::OverMaxSolt.into());
        }
    }

    // send sol to vault
    invoke(
        &system_instruction::transfer(
            &accts.user.key(),
            &accts.vault.key(),
            accts.global_state.slot_token_price * amount
        ),
        &[
            accts.user.to_account_info().clone(),
            accts.vault.clone(),
            accts.system_program.to_account_info().clone(),
        ],
    )?;

    // update the round data
    accts.round.current_slot_number += amount;
    // update the user round info data
    accts.round_user_info.round_index = round_index;
    accts.round_user_info.buy_slot_number += amount;
    accts.round_user_info.claimed = false;

    Ok(())
}

pub fn claim_slot(ctx: Context<ClaimSlot>, round_index: u32) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.round.status == false, RoundError::Processing);
    require!(accts.next_round.status == false, RoundError::Processing);
    require!(accts.round_user_info.claimed == false, RoundError::AlreadyClaim);
    
    let amount = accts.round_user_info.buy_slot_number * accts.global_state.slot_token_price;

    let (_, bump) = Pubkey::find_program_address(&[VAULT_SEED], &crate::ID);

    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.user.key(), amount),
        &[
            accts.vault.to_account_info().clone(),
            accts.user.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        &[&[VAULT_SEED, &[bump]]],
    )?;
    // update the user round info data
   
    accts.round_user_info.claimed = true;

    Ok(())
}


pub fn withdraw_sol(ctx: Context<WithDrawSOL>, amount:u64) -> Result<()> {
    let accts = ctx.accounts;

    require!(accts.global_state.owner == accts.owner.key(), RoundError::NotAllowedOwner);

    let (_, bump) = Pubkey::find_program_address(&[VAULT_SEED], &crate::ID);

    invoke_signed(
        &system_instruction::transfer(&accts.vault.key(), &accts.owner.key(), amount),
        &[
            accts.vault.to_account_info().clone(),
            accts.owner.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        &[&[VAULT_SEED, &[bump]]],
    )?;

    Ok(())
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [GLOBAL_STATE_SEED], 
        bump,
        space = 8 + size_of::<GlobalState>()
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(round_index: u32)]
pub struct CreateRound<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED], 
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        payer = owner,
        seeds = [ROUND_SEED, &round_index.to_le_bytes()],
        bump, 
        space = 8 + size_of::<Round>()
    )]
    pub round: Account<'info, Round>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(round_index: u32)]
pub struct BuySlot<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED], 
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [ROUND_SEED, &round_index.to_le_bytes()],
        bump, 
    )]
    pub round: Account<'info, Round>,

    #[account(
        mut,
        address = global_state.vault
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    #[account(
        init_if_needed,
        payer = user,
        seeds = [ROUN_USER_INFO_SEED,  &round_index.to_le_bytes(),user.key().as_ref()],
        bump,
        space = 8 + size_of::<RoundUserInfo>()
    )]
    pub round_user_info: Account<'info, RoundUserInfo>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(round_index: u32)]
pub struct ClaimSlot<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED], 
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [ROUND_SEED, &(round_index+1).to_le_bytes()],
        bump, 
    )]
    pub next_round: Account<'info, Round>,

    #[account(
        mut,
        seeds = [ROUND_SEED, &round_index.to_le_bytes()],
        bump, 
    )]
    pub round: Account<'info, Round>,

    #[account(
        mut,
        address = global_state.vault
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    #[account(
        mut,
        seeds = [ROUN_USER_INFO_SEED,  &round_index.to_le_bytes(), user.key().as_ref()],
        bump,
    )]
    pub round_user_info: Account<'info, RoundUserInfo>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct WithDrawSOL<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED], 
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        address = global_state.vault
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    pub system_program: Program<'info, System>,
}
