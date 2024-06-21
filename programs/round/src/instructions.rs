use crate::{constants::*, errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use solana_program::{program::{invoke, invoke_signed}, system_instruction};

use std::mem::size_of;

pub fn initialize(ctx: Context<Initialize>, slot_token_price: u64, fee: u64) -> Result<()> {
    let accts = ctx.accounts;
    require!(fee < 100, RoundError::MaxFeeError);

    // init the global state account
    accts.global_state.owner = accts.owner.key();
    accts.global_state.total_round = 0;
    accts.global_state.slot_token_price = slot_token_price;
    accts.global_state.vault = accts.vault.key();
    accts.global_state.fee = fee;

    Ok(())
}

pub fn update_fee(ctx: Context<Update>, new_fee: u64) -> Result<()> {
    let accts = ctx.accounts;
    require!(accts.global_state.owner == accts.owner.key(), RoundError::NotAllowedOwner);

    accts.global_state.fee = new_fee;
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
    // update the global state
    accts.global_state.total_round += 1;

    Ok(())
}

pub fn buy_slot(ctx: Context<BuySlot>, round_index: u32, amount: u64) -> Result<()> {
    let accts = ctx.accounts;

    let mut amount = amount;
    
    if accts.round.current_slot_number + amount >= accts.round.total_slot_number {
        if accts.round.current_slot_number <= accts.round.total_slot_number {
            amount = accts.round.total_slot_number - accts.round.current_slot_number;
            // create the new round by the last buyer
            accts.round.round_index = round_index + 1;
            accts.round.total_slot_number =  2_u64.pow(round_index);
            accts.round.current_slot_number = 0;
        } else {
            return Err(RoundError::OverMaxSlot.into());
        }
    } else {
        // update the round data
        accts.round.current_slot_number += amount;
    }

    // send sol to vault
    let transfer_amount = accts.global_state.slot_token_price * amount * (1000 - accts.global_state.fee) / 1000;
    let fee_amount = accts.global_state.slot_token_price * amount * accts.global_state.fee / 1000;

    invoke(
        &system_instruction::transfer(
            &accts.user.key(),
            &accts.vault.key(),
            transfer_amount
        ),
        &[
            accts.user.to_account_info().clone(),
            accts.vault.clone(),
            accts.system_program.to_account_info().clone(),
        ],
    )?;

    invoke(
        &system_instruction::transfer(
            &accts.user.key(),
            &accts.owner.key(),
            fee_amount
        ),
        &[
            accts.user.to_account_info().clone(),
            accts.owner.clone(),
            accts.system_program.to_account_info().clone(),
        ],
    )?;
    // update the user info data
    if accts.user_info.last_round_index == 0 {
        accts.user_info.total_slot_number = 0;
        accts.user_info.last_slot_number = amount;
        accts.user_info.last_round_index = round_index;
    } else {
        accts.user_info.total_slot_number += accts.user_info.last_slot_number;
        accts.user_info.last_slot_number = amount;
        accts.user_info.last_round_index = round_index;
    }

    Ok(())
}

pub fn claim_slot(ctx: Context<ClaimSlot>) -> Result<()> {
    let accts = ctx.accounts;
    let mut amount = 0;
   
    if accts.global_state.total_round < accts.user_info.last_round_index + 1 {
        amount = (2000 - accts.global_state.fee) * accts.user_info.total_slot_number * accts.global_state.slot_token_price / 1000;
        accts.user_info.total_slot_number = 0;
        accts.user_info.claimed_slot_number += amount;
    } else {
        amount = (2000 - accts.global_state.fee) * (accts.user_info.total_slot_number + accts.user_info.last_slot_number) * accts.global_state.slot_token_price / 1000;
        accts.user_info.total_slot_number = 0;
        accts.user_info.last_slot_number = 0;
        accts.user_info.claimed_slot_number += amount;
    }

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
pub struct Update<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED], 
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
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
        init_if_needed,
        payer = owner,
        seeds = [ROUND_SEED],
        bump, 
        space = 8 + size_of::<RoundState>()
    )]
    pub round: Account<'info, RoundState>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct BuySlot<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED], 
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [ROUND_SEED],
        bump, 
    )]
    pub round: Account<'info, RoundState>,

    #[account(
        mut,
        address = global_state.vault
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    #[account(
        init_if_needed,
        payer = user,
        seeds = [ROUN_USER_INFO_SEED, user.key().as_ref()],
        bump,
        space = 8 + size_of::<UserInfo>()
    )]
    pub user_info: Account<'info, UserInfo>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
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
        address = global_state.vault
    )]
    /// CHECK: this should be set by admin
    pub vault: AccountInfo<'info>,  // to receive SOL

    #[account(
        mut,
        seeds = [ROUN_USER_INFO_SEED, user.key().as_ref()],
        bump,
    )]
    pub user_info: Account<'info, UserInfo>,

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
