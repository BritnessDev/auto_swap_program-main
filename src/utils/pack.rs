//! Common routines for packing/unpacking

use {
    solana_program::{program_error::ProgramError},
};

/// Checks if the slice has at least min_len size
pub fn check_data_len(data: &[u8], min_len: usize) -> Result<(), ProgramError> {
    if data.len() < min_len {
        Err(ProgramError::AccountDataTooSmall)
    } else {
        Ok(())
    }
}