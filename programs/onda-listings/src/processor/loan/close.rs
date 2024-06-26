use {
    anchor_lang::{
        prelude::*,
        solana_program::{
            program::{invoke_signed},
            system_instruction::{transfer}
        },
        AccountsClose
    },
    anchor_spl::token::{Mint, Token, TokenAccount}
};

use crate::state::{Collection, Loan, LoanState, LoanOffer, TokenManager};
use crate::error::{ErrorCodes};
use crate::utils::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct CloseLoan<'info> {
    #[account(
        constraint = signer.key() == SIGNER_PUBKEY
    )]
    pub signer: Signer<'info>,
    pub borrower: Signer<'info>,
    #[account(
        mut,
        constraint = deposit_token_account.owner == borrower.key(),
    )]
    pub deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// CHECK: validated in cpi
    pub deposit_token_record: Option<UncheckedAccount<'info>>,
    #[account(
        mut,
        seeds = [
            Loan::PREFIX,
            mint.key().as_ref(),
            borrower.key().as_ref(),
        ],
        bump,
        has_one = mint,
        has_one = borrower,
        constraint = loan.state != LoanState::Active @ ErrorCodes::InvalidState,
        close = borrower,
    )]
    pub loan: Box<Account<'info, Loan>>,
    #[account(
        mut,
        seeds = [
            TokenManager::PREFIX,
            mint.key().as_ref(),
        ],
        bump,
    )]   
    pub token_manager: Box<Account<'info, TokenManager>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut)]
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
}



pub fn handle_close_loan(ctx: Context<CloseLoan>) -> Result<()> {
    let token_manager = &mut ctx.accounts.token_manager;
    let borrower = &ctx.accounts.borrower;
    let deposit_token_account = &ctx.accounts.deposit_token_account;
    let mint = &ctx.accounts.mint;
    let metadata = &ctx.accounts.metadata;
    let edition = &ctx.accounts.edition;
    let deposit_token_record = &ctx.accounts.deposit_token_record;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let sysvar_instructions = &ctx.accounts.sysvar_instructions;
    let authorization_rules_program = &ctx.accounts.authorization_rules_program;
    let authorization_rules = &ctx.accounts.authorization_rules;

    msg!("Loan state: {:?}", ctx.accounts.loan.state);

    // IMPORTANT CHECK!
    if token_manager.authority.unwrap().eq(&borrower.key()) {
        // IMPORTANT CHECK!
        if token_manager.accounts.rental == false {
            handle_thaw_and_revoke(
                token_manager,
                borrower.to_account_info(),
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
                },
            )?;
        
            token_manager.close(borrower.to_account_info())?;    
        } else {
            token_manager.accounts.loan = false;
        }   
    }
  
    Ok(())
}

#[derive(Accounts)]
#[instruction(id: u8)]
pub struct CloseLoanOffer<'info> {
    #[account(
        constraint = signer.key() == SIGNER_PUBKEY
    )]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub lender: Signer<'info>,
    #[account(
        mut,
        seeds = [
            LoanOffer::PREFIX,
            collection.mint.as_ref(),
            lender.key().as_ref(),
            &[id],
        ],
        close = lender,
        bump,
    )]
    pub loan_offer: Box<Account<'info, LoanOffer>>,
    /// CHECK: seeds
    #[account(
        mut,
        seeds=[
            LoanOffer::VAULT_PREFIX,
            loan_offer.key().as_ref()
        ],
        bump,
    )]
    pub escrow_payment_account: AccountInfo<'info>,
    #[account(
        seeds = [
            Collection::PREFIX,
            collection.mint.as_ref(),
        ],
        bump,
    )]
    pub collection: Box<Account<'info, Collection>>,
    /// Misc
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle_close_loan_offer(ctx: Context<CloseLoanOffer>, _id: u8) -> Result<()> {
    let loan_offer = &ctx.accounts.loan_offer;
    let escrow_payment_account = &ctx.accounts.escrow_payment_account;

    let loan_offer_pubkey = loan_offer.key();
    let signer_bump = &[loan_offer.escrow_bump];
    let signer_seeds = &[&[
        LoanOffer::VAULT_PREFIX,
        loan_offer_pubkey.as_ref(),
        signer_bump
    ][..]];

    invoke_signed(
        &transfer(
            &escrow_payment_account.key(),
            &loan_offer.lender,
            loan_offer.amount.unwrap(),
        ),
        &[
            escrow_payment_account.to_account_info(),
            ctx.accounts.lender.to_account_info(),
        ],
        signer_seeds
    )?;

    Ok(())
}