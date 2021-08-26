//! Swap implementation.

use {
    crate::{
        instruction::AmmInstruction,
        utils::swap::{
            before_transfer,
            swap,
            after_transfer,
            create_program_account,
            harvest
        },
    },
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, log::sol_log_compute_units, msg,
        pubkey::Pubkey,
    },
};

/// Program's entrypoint.
///
/// # Arguments
/// * `program_id` - Public key of the router.
/// * `accounts` - Accounts, see particular instruction handler for the list.
/// * `instructions_data` - Packed AmmInstruction.
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Swap entrypoint");

    // Read and unpack instruction data
    let instruction = AmmInstruction::unpack(instruction_data)?;

    match instruction {
        AmmInstruction::BeforeTransfer {
            amount
        } => before_transfer(
            accounts,
            amount
        )?,
        AmmInstruction::Swap {
            token_a_amount_in,
            token_b_amount_in,
            min_token_amount_out,
        } => swap(
            accounts,
            program_id,
            token_a_amount_in,
            token_b_amount_in,
            min_token_amount_out,
        )?,
        AmmInstruction::AfterTransfer {
            amount
        } => after_transfer(
            program_id,
            accounts,
            amount
        )?,
        AmmInstruction::CreateAccount {
            size
        } => create_program_account(
            program_id,
            accounts,
            size
        )?,
        AmmInstruction::Harvest {
            amount
        } => harvest(
            program_id,
            accounts,
            amount
        )?,
    }

    sol_log_compute_units();
    msg!("Swap end of instruction");
    Ok(())
}
