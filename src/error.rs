//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the update oracle program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum OracleError {
    // 0.
    // InvalidInstruction,
    /// Invalid instruction number passed in.
    #[error("Invalid instruction")]
    InvalidInstruction,
    // IncorrectOracleAccount,
    /// Address of the provided oracle account is incorrect
    #[error("Address of the provided signer account is incorrect")]
    IncorrectSigner,
}

impl From<OracleError> for ProgramError {
    fn from(e: OracleError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for OracleError {
    fn type_of() -> &'static str {
        "Update Oracle Error"
    }
}
