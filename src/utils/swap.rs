//! Swap tokens with the raydium pool instruction

use {
    crate::{
        utils::raydium::{SwapRouteIn, SwapRouteOut, RaydiumSwap},
        utils::account,
        utils::tokens::{
            TokenTransferParams,
            spl_token_transfer,
            PREFIX,
        },
        protocol::raydium,
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        instruction::{AccountMeta, Instruction},
        system_instruction,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvar::{
            rent::Rent,
            Sysvar,
        },
    },
    std::convert::TryInto,
};

pub fn create_program_account(
    program_id : &Pubkey,
    accounts: &[AccountInfo],
    size: u64,
) -> ProgramResult {
    
    let account_info_iter = &mut accounts.iter();
    let program_account_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    let seed = &[
        PREFIX.as_bytes(),
    ];

    let (program_account, bump_seed) = Pubkey::find_program_address(seed, &program_id);

    let program_account_signer_seeds = &[
        PREFIX.as_bytes(),
        &[bump_seed],
    ];

    create_or_allocate_account_raw(
        *program_id,
        program_account_info,
        rent_info,
        system_account_info,
        payer_account_info,
        size as usize,
        program_account_signer_seeds,
    )?;

    Ok(())
}

#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports*3);
        invoke(
            &system_instruction::transfer(&payer_info.key, new_account_info.key, required_lamports*3),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    let accounts = &[new_account_info.clone(), system_program_info.clone()];

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        accounts,
        &[&signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        accounts,
        &[&signer_seeds],
    )?;

    Ok(())
}

pub fn before_transfer(
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("Processing AmmInstruction::BeforeTransfer");
    msg!("amount {} ", amount);

    let account_info_iter = &mut accounts.iter();
    let user_account_info = next_account_info(account_info_iter)?;
    let token_program_id_info = next_account_info(account_info_iter)?;
    let user_sol_account_info = next_account_info(account_info_iter)?;
    let program_sol_account_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;

    spl_token_transfer(
        TokenTransferParams{
            source: user_sol_account_info.clone(),
            destination: program_sol_account_info.clone(),
            authority: user_transfer_authority_info.clone(),
            token_program: token_program_id_info.clone(),
            authority_signer_seeds: &[],
            amount: amount
        }
    )?;

    Ok(())
}

pub fn swap(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    token_a_amount_in: u64,
    token_b_amount_in: u64,
    min_token_amount_out: u64,
) -> ProgramResult {
    msg!("Processing AmmInstruction::Swap");
    msg!("token_a_amount_in {} ", token_a_amount_in);
    msg!("token_b_amount_in {} ", token_b_amount_in);
    msg!("min_token_amount_out {} ", min_token_amount_out);

    #[allow(clippy::deprecated_cfg_attr)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    if let [
        program_account,
        program_token_a_account,
        program_token_b_account,
        pool_program_id,
        pool_coin_token_account,
        pool_pc_token_account,
        spl_token_id,
        amm_id,
        amm_authority,
        amm_open_orders,
        amm_target,
        serum_market,
        serum_program_id,
        serum_bids,
        serum_asks,
        serum_event_queue,
        serum_coin_vault_account,
        serum_pc_vault_account,
        serum_vault_signer
        ] = accounts
    {
        if !raydium::check_pool_program_id(pool_program_id.key) {
            return Err(ProgramError::IncorrectProgramId);
        }

        let seed = &[
            PREFIX.as_bytes(),
        ];
        
        let (_program_account_address, bump_seed) = Pubkey::find_program_address(seed, &program_id);
        let program_authority_seed = &[
            PREFIX.as_bytes(),
            &[bump_seed],
        ];

        let (amount_in, mut min_amount_out) = raydium::get_pool_swap_amounts(
            pool_coin_token_account,
            pool_pc_token_account,
            amm_open_orders,
            amm_id,
            token_a_amount_in,
            token_b_amount_in,
        )?;
        if min_token_amount_out > min_amount_out {
            min_amount_out = min_token_amount_out;
        }

        let initial_balance_in = if token_a_amount_in == 0 {
            account::get_token_balance(program_token_b_account)?
        } else {
            account::get_token_balance(program_token_a_account)?
        };
        let initial_balance_out = if token_a_amount_in == 0 {
            account::get_token_balance(program_token_a_account)?
        } else {
            account::get_token_balance(program_token_b_account)?
        };

        let mut raydium_accounts = Vec::with_capacity(18);
        raydium_accounts.push(AccountMeta::new_readonly(*spl_token_id.key, false));
        raydium_accounts.push(AccountMeta::new(*amm_id.key, false));
        raydium_accounts.push(AccountMeta::new_readonly(*amm_authority.key, false));
        raydium_accounts.push(AccountMeta::new(*amm_open_orders.key, false));
        raydium_accounts.push(AccountMeta::new(*amm_target.key, false));
        raydium_accounts.push(AccountMeta::new(*pool_coin_token_account.key, false));
        raydium_accounts.push(AccountMeta::new(*pool_pc_token_account.key, false));
        raydium_accounts.push(AccountMeta::new_readonly(*serum_program_id.key, false));
        raydium_accounts.push(AccountMeta::new(*serum_market.key, false));
        raydium_accounts.push(AccountMeta::new(*serum_bids.key, false));
        raydium_accounts.push(AccountMeta::new(*serum_asks.key, false));
        raydium_accounts.push(AccountMeta::new(*serum_event_queue.key, false));
        raydium_accounts.push(AccountMeta::new(*serum_coin_vault_account.key, false));
        raydium_accounts.push(AccountMeta::new(*serum_pc_vault_account.key, false));
        raydium_accounts.push(AccountMeta::new_readonly(*serum_vault_signer.key, false));
        if token_a_amount_in == 0 {
            raydium_accounts.push(AccountMeta::new(*program_token_b_account.key, false));
            raydium_accounts.push(AccountMeta::new(*program_token_a_account.key, false));
        } else {
            raydium_accounts.push(AccountMeta::new(*program_token_a_account.key, false));
            raydium_accounts.push(AccountMeta::new(*program_token_b_account.key, false));
        }
        raydium_accounts.push(AccountMeta::new_readonly(*program_account.key, true));

        let instruction = Instruction {
            program_id: *pool_program_id.key,
            accounts: raydium_accounts,
            data: RaydiumSwap {
                instruction: 9,
                amount_in,
                min_amount_out,
            }
            .to_vec()?,
        };
        invoke_signed(&instruction, accounts, &[program_authority_seed])?;

        account::check_tokens_spent(
            if token_a_amount_in == 0 {
                program_token_b_account
            } else {
                program_token_a_account
            },
            initial_balance_in,
            amount_in,
        )?;
        account::check_tokens_received(
            if token_a_amount_in == 0 {
                program_token_a_account
            } else {
                program_token_b_account
            },
            initial_balance_out,
            min_amount_out,
        )?;
    } else {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    msg!("AmmInstruction::Swap complete");
    Ok(())
}

pub fn after_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("Processing AmmInstruction::AfterTransfer");
    let account_info_iter = &mut accounts.iter();
    let token_program_id_info = next_account_info(account_info_iter)?;
    let program_account_info = next_account_info(account_info_iter)?;
    let program_kin_account_info = next_account_info(account_info_iter)?;
    let program_sol_account_info = next_account_info(account_info_iter)?;
    let destination_account_info = next_account_info(account_info_iter)?;
    let fee_recipient_info = next_account_info(account_info_iter)?;
    let seed = &[
        PREFIX.as_bytes(),
    ];
    
    let (program_account, bump_seed) = Pubkey::find_program_address(seed, &program_id);
    let transfer_authority_seed = &[
        PREFIX.as_bytes(),
        &[bump_seed],
    ];

    let token_amount = account::get_token_balance(program_kin_account_info)?;
    spl_token_transfer(
        TokenTransferParams{
            source: program_kin_account_info.clone(),
            destination: destination_account_info.clone(),
            authority: program_account_info.clone(),
            token_program: token_program_id_info.clone(),
            authority_signer_seeds: transfer_authority_seed,
            amount: token_amount,
        }
    )?;

    spl_token_transfer(
        TokenTransferParams{
            source: program_sol_account_info.clone(),
            destination: fee_recipient_info.clone(),
            authority: program_account_info.clone(),
            token_program: token_program_id_info.clone(),
            authority_signer_seeds: transfer_authority_seed,
            amount: (amount as f64 * 0.005) as u64,
        }
    )?;

    Ok(())
}

pub fn harvest(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("Processing AmmInstruction::AfterTransfer");
    let account_info_iter = &mut accounts.iter();
    let token_program_id_info = next_account_info(account_info_iter)?;
    let program_account_info = next_account_info(account_info_iter)?;
    let program_sol_account_info = next_account_info(account_info_iter)?;
    let user_account_info = next_account_info(account_info_iter)?;
    let seed = &[
        PREFIX.as_bytes(),
    ];
    
    let (program_account, bump_seed) = Pubkey::find_program_address(seed, &program_id);
    let transfer_authority_seed = &[
        PREFIX.as_bytes(),
        &[bump_seed],
    ];

    spl_token_transfer(
        TokenTransferParams{
            source: program_sol_account_info.clone(),
            destination: user_account_info.clone(),
            authority: program_account_info.clone(),
            token_program: token_program_id_info.clone(),
            authority_signer_seeds: transfer_authority_seed,
            amount: amount,
        }
    )?;

    Ok(())
}