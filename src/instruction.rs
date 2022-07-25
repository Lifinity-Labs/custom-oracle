//! Instruction types

#![allow(clippy::too_many_arguments)]

use crate::error::OracleError;
use solana_program::{
    program_error::ProgramError,
};
use std::convert::TryInto;
use std::mem::size_of;

/// Update instruction data
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct Update {
    /// price used to update oracle data
    pub price: i64,
    /// confidence used to update oracle data
    pub confidence: u64,
    /// status used to update oracle data
    pub status: u32,
}

/// Instructions supported by the update oracle program.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum OracleInstruction {
    ///   Oracle update..
    Update(Update),
}

impl OracleInstruction {
    /// Unpacks a byte buffer into a [OracleInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(OracleError::InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (price, rest) = Self::unpack_i64(rest)?;
                let (confidence, rest) = Self::unpack_u64(rest)?;
                let (status, _rest) = Self::unpack_u32(rest)?;
                Self::Update(Update {
                    price,
                    confidence,
                    status,
                })
            }
            _ => return Err(OracleError::InvalidInstruction.into()),
        })
    }

    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() >= 8 {
            let (value, rest) = input.split_at(8);
            let value = value
                .get(..8)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(OracleError::InvalidInstruction)?;
            Ok((value, rest))
        } else {
            Err(OracleError::InvalidInstruction.into())
        }
    }

    fn unpack_i64(input: &[u8]) -> Result<(i64, &[u8]), ProgramError> {
        if input.len() >= 8 {
            let (value, rest) = input.split_at(8);
            let value = value
                .get(..8)
                .and_then(|slice| slice.try_into().ok())
                .map(i64::from_le_bytes)
                .ok_or(OracleError::InvalidInstruction)?;
            Ok((value, rest))
        } else {
            Err(OracleError::InvalidInstruction.into())
        }
    }

    fn unpack_u32(input: &[u8]) -> Result<(u32, &[u8]), ProgramError> {
        if input.len() >= 4 {
            let (value, rest) = input.split_at(4);
            let value = value
                .get(..4)
                .and_then(|slice| slice.try_into().ok())
                .map(u32::from_le_bytes)
                .ok_or(OracleError::InvalidInstruction)?;
            Ok((value, rest))
        } else {
            Err(OracleError::InvalidInstruction.into())
        }
    }

    /// Packs a [OracleInstruction] into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match &*self {
            Self::Update(Update {
                price,
                confidence,
                status,
            }) => {
                buf.push(0);
                buf.extend_from_slice(&price.to_le_bytes());
                buf.extend_from_slice(&confidence.to_le_bytes());
                buf.extend_from_slice(&status.to_le_bytes());
            }
        }
        buf
    }
}
