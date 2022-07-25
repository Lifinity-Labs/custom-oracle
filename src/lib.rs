#![deny(missing_docs)]

//! An Uniswap-like program for the Solana blockchain.

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

extern crate arrayref;

solana_program::declare_id!("8BR3zs8zSXetpnDjCtHWnkpSkNSydWb3PTTDuVKku2uu");
