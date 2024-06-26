use anchor_lang::{prelude::*};
use solana_program::pubkey;

pub const SECONDS_PER_DAY: i64 = 86_400;
pub const SECONDS_PER_YEAR: i64 = 31_536_000;
pub const LATE_REPAYMENT_FEE_BASIS_POINTS: u128 = 500;
pub const SIGNER_PUBKEY: Pubkey = pubkey!("4RfijtGGJnnaLYYByWGTbkPrGgvmKeAP1bZBhwZApLPq");
pub const SYSTEM_ACCOUNT: Pubkey = pubkey!("11111111111111111111111111111111");
pub const ADMIN_PUBKEY: Pubkey = pubkey!("AH7F2EPHXWhfF5yc7xnv1zPbwz3YqD6CtAqbCyE9dy7r");