use {
    anchor_lang::{
        prelude::*,
        solana_program::{
            program::{invoke_signed},
            system_instruction::{transfer}
        },
    },
    anchor_spl::token::{Mint, Token, TokenAccount}
};
use crate::state::{CallOption, CallOptionBid, Collection, TokenManager};
use crate::utils::*;
use crate::error::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(id: u8)]
pub struct SellCallOption<'info> {
    #[account(
        constraint = signer.key() == SIGNER_PUBKEY
    )]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    /// CHECK: seeds
    pub buyer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = deposit_token_account.amount == 1,
        constraint = deposit_token_account.mint == mint.key(),
    )]
    pub deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// CHECK: validated in cpi
    pub deposit_token_record: Option<UncheckedAccount<'info>>,
    #[account(
        init,
        payer = seller,
        seeds = [
            CallOption::PREFIX,
            mint.key().as_ref(),
            seller.key().as_ref(),
        ],
        space = CallOption::space(),
        bump,
    )]
    pub call_option: Box<Account<'info, CallOption>>, 
    #[account(
        mut,
        seeds = [
            CallOptionBid::PREFIX,
            collection.mint.as_ref(),
            buyer.key().as_ref(),
            &[id],
        ],
        close = buyer,
        bump,
    )]
    pub call_option_bid: Box<Account<'info, CallOptionBid>>,
    #[account(
        mut,
        seeds=[
            CallOptionBid::VAULT_PREFIX,
            call_option_bid.key().as_ref()
        ],
        bump,
    )]
    /// CHECK: seeds
    pub escrow_payment_account: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = seller,
        seeds = [
            TokenManager::PREFIX,
            seller.key().as_ref()
        ],
        space = TokenManager::space(),
        bump,
        constraint = token_manager.authority.unwrap() == seller.key() @ ErrorCodes::Unauthorized,
    )]
    pub token_manager: Box<Account<'info, TokenManager>>,
    #[account(
        seeds = [
            Collection::PREFIX,
            collection.mint.as_ref(),
        ],
        bump,
        constraint = collection.config.option_enabled == true
    )]
    pub collection: Box<Account<'info, Collection>>,
    #[account(constraint = mint.supply == 1)]
    pub mint: Box<Account<'info, Mint>>,
    /// CHECK: deserialized and checked
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: validated in cpi
    pub edition: UncheckedAccount<'info>,
    /// CHECK: validated in cpi
    pub metadata_program: UncheckedAccount<'info>,
    /// CHECK: validated in cpi
    pub authorization_rules_program: UncheckedAccount<'info>,
    /// CHECK: validated in cpi
    pub authorization_rules: Option<UncheckedAccount<'info>>, 
    /// Misc
    /// CHECK: not supported by anchor? used in cpi
    pub sysvar_instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handle_sell_call_option<'info>(
  ctx: Context<'_, '_, '_, 'info, SellCallOption<'info>>,
  _id: u8,
) -> Result<()> {
    let call_option = &mut ctx.accounts.call_option;
    let bid = &mut ctx.accounts.call_option_bid;
    let seller = &ctx.accounts.seller;
    let buyer = &ctx.accounts.buyer;
    let token_manager = &mut ctx.accounts.token_manager;
    let collection = &ctx.accounts.collection;
    let escrow_payment_account = &mut ctx.accounts.escrow_payment_account;
    let deposit_token_account = &mut ctx.accounts.deposit_token_account;
    let deposit_token_record = &ctx.accounts.deposit_token_record;
    let mint = &ctx.accounts.mint;
    let metadata = &ctx.accounts.metadata;
    let edition = &ctx.accounts.edition;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let sysvar_instructions = &ctx.accounts.sysvar_instructions;
    let authorization_rules_program = &ctx.accounts.authorization_rules_program;
    let authorization_rules = &ctx.accounts.authorization_rules;
    let unix_timestamp = ctx.accounts.clock.unix_timestamp;
    let remaining_accounts = &mut ctx.remaining_accounts.iter();

    assert_collection_valid(
        &ctx.accounts.metadata,
        ctx.accounts.mint.key(),
        collection.key(),
        ctx.program_id.clone(),
    )?;

    require_eq!(token_manager.accounts.loan, false, ErrorCodes::InvalidState);

    // Init
    call_option.seller = seller.key();
    call_option.buyer = Some(buyer.key());
    call_option.mint = mint.key();
    call_option.bump = *ctx.bumps.get("call_option").unwrap();
    //
    CallOption::init_ask_state(call_option, bid.amount, collection.config.option_basis_points, bid.strike_price, bid.expiry)?;
    CallOption::set_active(call_option, unix_timestamp)?;
    //
    token_manager.accounts.call_option = true;
    token_manager.bump = *ctx.bumps.get("token_manager").unwrap();

    let call_option_bid_pubkey = bid.key();
    let signer_bump = &[bid.escrow_bump];
    let signer_seeds = &[&[
        CallOptionBid::VAULT_PREFIX,
        call_option_bid_pubkey.as_ref(),
        signer_bump
    ][..]];

    let fee_basis_points = collection.config.option_basis_points;
    pay_creator_fees_with_signer(
        call_option.amount,
        fee_basis_points, 
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.metadata.to_account_info(),
        &mut ctx.accounts.buyer.to_account_info(),
        remaining_accounts,
        signer_seeds
    )?;

    invoke_signed(
        &transfer(
            &escrow_payment_account.key(),
            &call_option.seller,
            bid.amount,
        ),
        &[
            escrow_payment_account.to_account_info(),
            ctx.accounts.seller.to_account_info(),
        ],
        signer_seeds
    )?;

    // Freeze deposit token account
    if deposit_token_account.delegate.is_some() {
        if deposit_token_account.delegate.unwrap() != token_manager.key() {
            return err!(ErrorCodes::InvalidState);
        }
    } else {
        handle_delegate_and_freeze(
            token_manager,
            seller.to_account_info(),
            deposit_token_account.to_account_info(),
            match deposit_token_record {
                Some(token_record) => Some(token_record.to_account_info()),
                None => None,
            },
            mint.to_account_info(),
            metadata.to_account_info(),
            edition.to_account_info(),
            token_program.to_account_info(),
            system_program.to_account_info(),
            sysvar_instructions.to_account_info(),
            authorization_rules_program.to_account_info(),
            match authorization_rules {
                Some(authorization_rules) => Some(authorization_rules.to_account_info()),
                None => None,
            }
        )?;
    }

    Ok(())
}