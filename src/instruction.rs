//! Raydium router instructions.

use {
    crate::utils::pack::check_data_len,
    arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs},
    num_enum::TryFromPrimitive,
    solana_program::program_error::ProgramError,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AmmInstruction {
    /// Swap tokens in the AMM Pool
    /// # Account references are protocol specific,
    ///   see particular Router instructions handlers for more info
    BeforeTransfer {
        amount: u64,
    },
    Swap {
        token_a_amount_in: u64,
        token_b_amount_in: u64,
        min_token_amount_out: u64,
    },
    AfterTransfer {
        amount: u64,
    },
    CreateAccount {
        size: u64,
    },
    Harvest {
        amount: u64,
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum AmmInstructionType {
    BeforeTransfer,
    Swap,
    AfterTransfer,
    CreateAccount,
    Harvest,
}

impl AmmInstruction {
    pub const LEN: usize = 9;
    pub const SWAP_LEN: usize = 25;

    pub fn pack(&self, output: &mut [u8]) -> Result<usize, ProgramError> {
        match self {
            Self::BeforeTransfer { .. } => self.pack_before_transfer(output),
            Self::Swap { .. } => self.pack_swap(output),
            Self::AfterTransfer { .. } => self.pack_after_transfer(output),
            Self::CreateAccount { .. } => self.pack_create_account(output),
            Self::Harvest { .. } => self.pack_harvest(output),
        }
    }

    pub fn unpack(input: &[u8]) -> Result<AmmInstruction, ProgramError> {
        check_data_len(input, 1)?;
        let instruction_type = AmmInstructionType::try_from_primitive(input[0])
            .or(Err(ProgramError::InvalidInstructionData))?;
        match instruction_type {
            AmmInstructionType::BeforeTransfer => AmmInstruction::unpack_before_transfer(input),
            AmmInstructionType::Swap => AmmInstruction::unpack_swap(input),
            AmmInstructionType::AfterTransfer => AmmInstruction::unpack_after_transfer(input),
            AmmInstructionType::CreateAccount => AmmInstruction::unpack_create_account(input),
            AmmInstructionType::Harvest => AmmInstruction::unpack_harvest(input),
        }
    }

    fn pack_before_transfer(&self, output: &mut [u8]) -> Result<usize, ProgramError> {
        check_data_len(output, AmmInstruction::LEN)?;
        if let AmmInstruction::BeforeTransfer {
            amount,
        } = self
        {
            let output = array_mut_ref![output, 0, AmmInstruction::LEN];
            let (
                instruction_type_pack,
                amount_pack,
            ) = mut_array_refs![output, 1, 8];

            instruction_type_pack[0] = AmmInstructionType::BeforeTransfer as u8;

            *amount_pack = amount.to_le_bytes();

            Ok(AmmInstruction::LEN)
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }

    fn pack_swap(&self, output: &mut [u8]) -> Result<usize, ProgramError> {
        check_data_len(output, AmmInstruction::SWAP_LEN)?;

        if let AmmInstruction::Swap {
            token_a_amount_in,
            token_b_amount_in,
            min_token_amount_out,
        } = self
        {
            let output = array_mut_ref![output, 0, AmmInstruction::SWAP_LEN];
            let (
                instruction_type_pack,
                token_a_amount_in_pack,
                token_b_amount_in_pack,
                min_token_amount_out_pack,
            ) = mut_array_refs![output, 1, 8, 8, 8];

            instruction_type_pack[0] = AmmInstructionType::Swap as u8;

            *token_a_amount_in_pack = token_a_amount_in.to_le_bytes();
            *token_b_amount_in_pack = token_b_amount_in.to_le_bytes();
            *min_token_amount_out_pack = min_token_amount_out.to_le_bytes();

            Ok(AmmInstruction::SWAP_LEN)
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }

    fn pack_after_transfer(&self, output: &mut [u8]) -> Result<usize, ProgramError> {
        check_data_len(output, AmmInstruction::LEN)?;
        if let AmmInstruction::AfterTransfer {
            amount,
        } = self
        {
            let output = array_mut_ref![output, 0, AmmInstruction::LEN];
            let (
                instruction_type_pack,
                amount_pack,
            ) = mut_array_refs![output, 1, 8];

            instruction_type_pack[0] = AmmInstructionType::AfterTransfer as u8;

            *amount_pack = amount.to_le_bytes();

            Ok(AmmInstruction::LEN)
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }

    fn pack_create_account(&self, output: &mut [u8]) -> Result<usize, ProgramError> {
        check_data_len(output, AmmInstruction::LEN)?;
        if let AmmInstruction::CreateAccount {
            size,
        } = self
        {
            let output = array_mut_ref![output, 0, AmmInstruction::LEN];
            let (
                instruction_type_pack,
                size_pack,
            ) = mut_array_refs![output, 1, 8];

            instruction_type_pack[0] = AmmInstructionType::CreateAccount as u8;

            *size_pack = size.to_le_bytes();

            Ok(AmmInstruction::LEN)
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }

    fn pack_harvest(&self, output: &mut [u8]) -> Result<usize, ProgramError> {
        check_data_len(output, AmmInstruction::LEN)?;
        if let AmmInstruction::Harvest {
            amount,
        } = self
        {
            let output = array_mut_ref![output, 0, AmmInstruction::LEN];
            let (
                instruction_type_pack,
                amount_pack,
            ) = mut_array_refs![output, 1, 8];

            instruction_type_pack[0] = AmmInstructionType::Harvest as u8;

            *amount_pack = amount.to_le_bytes();

            Ok(AmmInstruction::LEN)
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }

    fn unpack_before_transfer(input: &[u8]) -> Result<AmmInstruction, ProgramError> {
        check_data_len(input, AmmInstruction::LEN)?;

        let input = array_ref![input, 1, AmmInstruction::LEN - 1];
        #[allow(clippy::ptr_offset_with_cast)]
        let (amount, _) = array_refs![input, 8, 0];

        Ok(Self::BeforeTransfer {
            amount: u64::from_le_bytes(*amount),
        })
    }

    fn unpack_swap(input: &[u8]) -> Result<AmmInstruction, ProgramError> {
        check_data_len(input, AmmInstruction::SWAP_LEN)?;

        let input = array_ref![input, 1, AmmInstruction::SWAP_LEN - 1];
        #[allow(clippy::ptr_offset_with_cast)]
        let (token_a_amount_in, token_b_amount_in, min_token_amount_out) =
            array_refs![input, 8, 8, 8];

        Ok(Self::Swap {
            token_a_amount_in: u64::from_le_bytes(*token_a_amount_in),
            token_b_amount_in: u64::from_le_bytes(*token_b_amount_in),
            min_token_amount_out: u64::from_le_bytes(*min_token_amount_out),
        })
    }

    fn unpack_after_transfer(input: &[u8]) -> Result<AmmInstruction, ProgramError> {
        check_data_len(input, AmmInstruction::LEN)?;

        let input = array_ref![input, 1, AmmInstruction::LEN - 1];
        #[allow(clippy::ptr_offset_with_cast)]
        let (amount, _) = array_refs![input, 8, 0];

        Ok(Self::AfterTransfer {
            amount: u64::from_le_bytes(*amount),
        })
    }

    fn unpack_create_account(input: &[u8]) -> Result<AmmInstruction, ProgramError> {
        check_data_len(input, AmmInstruction::LEN)?;

        let input = array_ref![input, 1, AmmInstruction::LEN - 1];
        #[allow(clippy::ptr_offset_with_cast)]
        let (size, _) = array_refs![input, 8, 0];

        Ok(Self::CreateAccount {
            size: u64::from_le_bytes(*size),
        })
    }

    fn unpack_harvest(input: &[u8]) -> Result<AmmInstruction, ProgramError> {
        check_data_len(input, AmmInstruction::LEN)?;

        let input = array_ref![input, 1, AmmInstruction::LEN - 1];
        #[allow(clippy::ptr_offset_with_cast)]
        let (amount, _) = array_refs![input, 8, 0];

        Ok(Self::Harvest {
            amount: u64::from_le_bytes(*amount),
        })
    }
}

impl std::fmt::Display for AmmInstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AmmInstructionType::BeforeTransfer => write!(f, "before transfer"),
            AmmInstructionType::Swap => write!(f, "Swap"),
            AmmInstructionType::AfterTransfer => write!(f, "before transfer"),
            AmmInstructionType::CreateAccount => write!(f, "create account"),
            AmmInstructionType::Harvest => write!(f, "harvest"),
        }
    }
}