//! Program state processor
use crate::{
    error::OracleError,
    instruction::{
        Update, OracleInstruction,
    },
    state::{OracleV1, OracleVersion},
};
use num_traits::{FromPrimitive};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    msg,
    program_error::{PrintProgramError, ProgramError},
    pubkey::Pubkey,
    clock::Clock,
    sysvar::Sysvar,
};

/// magic number.
pub const MAGIC: u32 = 0xa1b2c3d4;
/// program version.
pub const VERSION: u32 = 2;
/// account type.
pub const ATYPE: u32 = 3;
/// account size.
pub const SIZE: u32 = 3312;
/// price type.
pub const TYPE: u32 = 1;
/// price exponent.
pub const EXPONENT: i32 = -8;
/// numerator state.
pub const NUMERATOR: u64 = 0;
/// denominator state.
pub const DENOMINATOR: u64 = 0;
/// number of quoters that make up aggregate.
pub const NUM_COMPONENT: u32 = 10;
/// slot of last valid aggregate price.
pub const NUM_QUOTERS: u32 = 1;
/// min publishers for valid price.
pub const MIN_PUBLISHERS: u8 = 1;
/// notification of any corporate action.
pub const ACTION: u32 = 0;

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Processes an [Update].
    pub fn process_update(
        program_id: &Pubkey,
        price: i64,
        confidence: u64,
        status: u32,
        accounts: &[AccountInfo],
    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let data_account_info = next_account_info(account_info_iter)?;

        if data_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if !data_account_info.is_signer {
           return Err(OracleError::IncorrectSigner.into());
        }

        let clock = Clock::get().unwrap();
        let solana_slot = clock.slot;
        let solana_timestamp = clock.unix_timestamp;

        let src = OracleVersion::OracleV1(OracleV1 {
            magic: MAGIC,
            version: VERSION,
            acctype: ATYPE,
            size: SIZE,
            price_type: TYPE,
            exponent: EXPONENT,
            num_component_prices: NUM_COMPONENT,
            num_quoters: NUM_QUOTERS,
            last_slot: solana_slot,
            valid_slot: solana_slot,
            ema_price_value: price as u64,
            ema_price_numerator: NUMERATOR,
            ema_price_denominator: DENOMINATOR,
            ema_confidence_value: confidence,
            ema_confidence_numerator: NUMERATOR,
            ema_confidence_denominator: DENOMINATOR,
            timestamp: solana_timestamp,
            min_publishers: MIN_PUBLISHERS,
            drv2: 0,
            drv3: 0,
            drv4: 0,
            product_account_key : *data_account_info.key,
            next_price_account_key : *data_account_info.key,
            previous_slot: solana_slot,
            previous_price_component: price,
            previous_confidence_component: confidence,
            previous_timestamp: solana_timestamp,
            price_component: price,
            confidence_component: confidence,
            status: status,
            corporate_action: ACTION,
            publish_slot: solana_slot,
            buffer: [0;192],
        });

        OracleVersion::pack(src, &mut data_account_info.data.borrow_mut())?;
        Ok(())
    }

    /// Processes an [Instruction].
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        Self::process_with_constraints(program_id, accounts, input)
    }

    /// Processes an instruction given extra constraint
    pub fn process_with_constraints(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        let instruction = OracleInstruction::unpack(input)?;
        match instruction {
            OracleInstruction::Update(Update {
                price,
                confidence,
                status,
            }) => {
                Self::process_update(
                    program_id,
                    price,
                    confidence,
                    status,
                    accounts,
                )
            }
        }
    }
}

impl PrintProgramError for OracleError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            OracleError::InvalidInstruction => msg!("Error: InvalidInstruction"),
            OracleError::IncorrectSigner => {
                msg!("Error: Address of the provided signer account is incorrect")
            }
        }
    }
}
