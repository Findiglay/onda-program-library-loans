use anchor_lang::prelude::*;
use crate::constants::*;
use crate::error::*;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Debug)]
pub enum CallOptionState {
    Listed,
    Active,
    Exercised,
}

#[account]
pub struct CallOption {
    /// Whether the option is active
    pub state: CallOptionState,
    /// The cost of the call option
    pub amount: u64,
    /// The creator fee
    pub creator_basis_points: u16,
    /// The issuer of the call option
    pub seller: Pubkey,
    /// The buyer of the call option
    pub buyer: Option<Pubkey>,
    /// Duration of the loan in seconds
    pub expiry: i64,
    /// The start date of the loan
    pub strike_price: u64,    
    /// The mint of the token being used for collateral
    pub mint: Pubkey,
    /// (Optional) The mint of the spl-token mint
    pub token_mint: Option<Pubkey>,
    /// Misc
    pub bump: u8,
}

impl CallOption {
    pub fn init_ask_state<'info>(
        call_option: &mut Account<'info, CallOption>,
        amount: u64,
        creator_basis_points: u16,
        strike_price: u64,
        expiry: i64
    ) -> Result<()> {
        call_option.state = CallOptionState::Listed;
        call_option.amount = amount;
        call_option.creator_basis_points = creator_basis_points;
        call_option.strike_price = strike_price;
        call_option.expiry = expiry;
    
        Ok(())
    }
    
    pub fn set_active<'info>(call_option: &mut Account<'info, CallOption>, unix_timestamp: i64) -> Result<()> {
        if call_option.state != CallOptionState::Listed {
            return err!(ErrorCodes::InvalidState);
        }
        
        require!(call_option.buyer.is_some(), ErrorCodes::InvalidState);
        require_keys_neq!(call_option.seller, SYSTEM_ACCOUNT, ErrorCodes::InvalidState);
        require_gt!(call_option.amount, 0, ErrorCodes::InvalidState);
        require_gt!(call_option.expiry, unix_timestamp, ErrorCodes::InvalidExpiry);
        require_gt!(call_option.strike_price, 0, ErrorCodes::InvalidState);
    
        call_option.state = CallOptionState::Active;
    
        Ok(())
    } 

    pub fn space() -> usize {
        8 + // key
        1 + // state
        8 + // amount
        2 + // create_basis_points
        32 + // seller
        1 + 32 + // buyer
        8 + // expiry
        8 + // strike price
        32 + // mint
        32 + // token mint
        1 // bump
    }

    pub const PREFIX: &'static [u8] = b"call_option";
}

#[account]
pub struct CallOptionBid {
    pub id: u8,
    /// The buyer making the offer
    pub buyer: Pubkey,
    /// Duration of the loan in seconds
    pub expiry: i64,
    /// The start date of the loan
    pub strike_price: u64,
    /// The cost of the call option
    pub amount: u64,
    /// The collection
    pub collection: Pubkey,
    /// misc
    pub bump: u8,
    pub escrow_bump: u8,
}

impl CallOptionBid {
    pub fn space() -> usize {
        8 + // key
        1 + // id
        32 + // buyer
        8 + // expiry
        8 + // strike_price
        8 + // amount
        32 + // collection
        1 + // bump
        1 // escrow_bump
    }

    pub const PREFIX: &'static [u8] = b"call_option_bid";
    pub const VAULT_PREFIX: &'static [u8] = b"call_option_bid_vault";
}