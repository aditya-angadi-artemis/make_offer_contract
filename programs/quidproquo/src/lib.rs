use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use metaplex_token_metadata;

declare_id!("HXu9ZCMB6AsL9s3yV76H72T3cZbKc6UxZA4oebaUwGy7");

#[program]
pub mod quidproquo {
    use super::*;

    pub fn new(
        ctx: Context<Initialize>,
        _data_bump: u8,
        mk_cut: u64,
        rent_cut: u64
    ) -> ProgramResult {
        let data_acc = &mut ctx.accounts.data_acc;
        data_acc.market_place = ctx.accounts.beneficiary.key();
        data_acc.rent = ctx.accounts.rent_account.key();
        data_acc.market_place_cut = mk_cut;
        data_acc.rent_cut = rent_cut;
        Ok(())
    }

    // Make a binding offer of `offer_maker_amount` of one kind of tokens in
    // exchange for `offer_taker_amount` of some other kind of tokens. This
    // will store the offer maker's tokens in an escrow account.
    pub fn make(
        ctx: Context<Make>,
        escrowed_maker_tokens_bump: u8,
        offer_bump: u8,
        offer_taker_amount: u64,
    ) -> ProgramResult {
        // Store some state about the offer being made. We'll need this later if
        // the offer gets accepted or cancelled.
        let offer = &mut ctx.accounts.offer;
        offer.maker = ctx.accounts.offer_maker.key();
        offer.taker_amount = offer_taker_amount;
        offer.escrowed_maker_tokens_bump = escrowed_maker_tokens_bump;

        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.offer_maker.key,
            ctx.accounts.tokenrent.key,
            10385941,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.offer_maker.to_account_info(),
                ctx.accounts.tokenrent.to_account_info(),
                ctx.accounts.offer.to_account_info(),
            ],
        )?;

        // Transfer the maker's tokens to the escrow account.
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.offer_makers_maker_tokens.to_account_info(),
                    to: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                    // The offer_maker had to sign from the client
                    authority: ctx.accounts.offer_maker.to_account_info(),
                },
            ),
            1,
        )
    }



    // Accept an offer by providing the right amount + kind of tokens. This
    // unlocks the tokens escrowed by the offer maker.
    pub fn accept(ctx: Context<Accept>, _offer_bump:u8) -> ProgramResult {
        
       
       let mut taker_amount = ctx.accounts.offer.taker_amount;
       // Multi by 10
       let market_cut = ctx.accounts.data_acc.market_place_cut * taker_amount / 1000;
       let sfb = metaplex_token_metadata::state::Metadata::from_account_info(&ctx.accounts.token_metadata_account)?.data.seller_fee_basis_points;
       let sfb_cut = sfb as u64 * taker_amount / 10000;
       taker_amount = taker_amount - (market_cut + sfb_cut);

        if *ctx.accounts.tokenrent.key != ctx.accounts.data_acc.rent {
            return Err(ProgramError::Custom(0x1));
        }
        
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.offer_taker.key,
            ctx.accounts.tokenrent.key,
            10385941,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.offer_taker.to_account_info(),
                ctx.accounts.tokenrent.to_account_info(),
                ctx.accounts.offer.to_account_info(),
            ],
        )?;

        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.offer_taker.key,
            ctx.accounts.offer_maker.key,
             taker_amount,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.offer_taker.to_account_info(),
                ctx.accounts.offer_maker.to_account_info(),
                ctx.accounts.offer.to_account_info(),
            ],
        )?;
        if *ctx.accounts.market_maker.key != ctx.accounts.data_acc.market_place {
            return Err(ProgramError::Custom(0x1));
        }
        
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.offer_taker.key,
            ctx.accounts.market_maker.key,
             market_cut,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.offer_taker.to_account_info(),
                ctx.accounts.market_maker.to_account_info(),
                ctx.accounts.offer.to_account_info(),
            ],
        )?;

        if sfb_cut > 0 {    
          // stick those CPIs in here
            if let Some(x) = metaplex_token_metadata::state::Metadata::from_account_info(&ctx.accounts.token_metadata_account)?.data.creators {
                let mut y = 0;

            for i in x {
                    if y == 0 {
                        if i.address != *ctx.accounts.creator0.key {
                            return Err(ProgramError::Custom(0x1));
                        }
                        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                            ctx.accounts.offer_taker.key,
                            ctx.accounts.creator0.key,
                            sfb_cut as u64 * i.share as u64 / 100,
                        );
                        
                        anchor_lang::solana_program::program::invoke(
                            &transfer_ix,
                            &[
                                ctx.accounts.offer_taker.to_account_info(),
                                ctx.accounts.creator0.to_account_info(),
                                ctx.accounts.offer.to_account_info(),
                            ],
                        )?;
                    }
                    else if y == 1 {
                        if i.address != *ctx.accounts.creator1.key {
                            return Err(ProgramError::Custom(0x1));
                        }
                        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                            ctx.accounts.offer_taker.key,
                            ctx.accounts.creator1.key,
                            sfb_cut as u64 * i.share as u64 / 100,
                        );
                        
                        anchor_lang::solana_program::program::invoke(
                            &transfer_ix,
                            &[
                                ctx.accounts.offer_taker.to_account_info(),
                                ctx.accounts.creator1.to_account_info(),
                                ctx.accounts.offer.to_account_info(),
                            ],
                        )?;
                    }
                    else if y == 2 {
                        if i.address != *ctx.accounts.creator2.key {
                            return Err(ProgramError::Custom(0x1));
                        }
                        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                            ctx.accounts.offer_taker.key,
                            ctx.accounts.creator2.key,
                            sfb_cut as u64 * i.share as u64 / 100,
                        );
                        
                        anchor_lang::solana_program::program::invoke(
                            &transfer_ix,
                            &[
                                ctx.accounts.offer_taker.to_account_info(),
                                ctx.accounts.creator2.to_account_info(),
                                ctx.accounts.offer.to_account_info(),
                            ],
                        )?;
                    }
                    else if y == 3 {
                        if i.address != *ctx.accounts.creator3.key {
                            return Err(ProgramError::Custom(0x1));
                        }
                        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                            ctx.accounts.offer_taker.key,
                            ctx.accounts.creator3.key,
                            sfb_cut as u64 * i.share as u64 / 100,
                        );
                        
                        anchor_lang::solana_program::program::invoke(
                            &transfer_ix,
                            &[
                                ctx.accounts.offer_taker.to_account_info(),
                                ctx.accounts.creator3.to_account_info(),
                                ctx.accounts.offer.to_account_info(),
                            ],
                        )?;
                    }
                    else if y == 4 {
                        if i.address != *ctx.accounts.creator1.key {
                            return Err(ProgramError::Custom(0x1));
                        }
                        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                            ctx.accounts.offer_taker.key,
                            ctx.accounts.creator4.key,
                            sfb_cut as u64 * i.share as u64 / 100,
                        );
                        
                        anchor_lang::solana_program::program::invoke(
                            &transfer_ix,
                            &[
                                ctx.accounts.offer_taker.to_account_info(),
                                ctx.accounts.creator4.to_account_info(),
                                ctx.accounts.offer.to_account_info(),
                            ],
                        )?;
                    }
                    y = y + 1;
                    msg!("address {}", i.address);
                    msg!("share in pc {}", i.share);
            }

            }

        }

        // Transfer the maker's tokens (the ones they escrowed) to the taker.
            anchor_spl::token::transfer(
                            CpiContext::new_with_signer(
                                ctx.accounts.token_program.to_account_info(),
                                anchor_spl::token::Transfer {
                                    from: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                                    to: ctx.accounts.offer_takers_maker_tokens.to_account_info(),
                                    // Cute trick: the escrowed_maker_tokens is its own
                                    // authority/owner (and a PDA, so our program can sign for
                                    // it just below)
                                    authority: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                                },
                                &[&[
                                    ctx.accounts.offer.key().as_ref(),
                                    &[ctx.accounts.offer.escrowed_maker_tokens_bump],
                                ]],
                            ),
                            // The amount here is just the entire balance of the escrow account.
                          1,
            )?;
            msg!("About to close account");
            //Finally, close the escrow account and refund the maker (they paid for
            // its rent-exemption).
            anchor_spl::token::close_account(CpiContext::new_with_signer(
                            ctx.accounts.token_program.to_account_info(),
                            anchor_spl::token::CloseAccount {
                                account: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                                destination: ctx.accounts.tokenrent.to_account_info(),
                                authority: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                            },
                            &[&[
                                ctx.accounts.offer.key().as_ref(),
                                &[ctx.accounts.offer.escrowed_maker_tokens_bump],
                            ]],
            ))?;
            msg!("Function End");
            Ok(())
 

    }

    pub fn cancel(ctx: Context<Cancel>, _offer_bump:u8) -> ProgramResult {

        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.offer_maker.key,
            ctx.accounts.tokenrent.key,
            10385941,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.offer_maker.to_account_info(),
                ctx.accounts.tokenrent.to_account_info(),
                ctx.accounts.offer.to_account_info(),
            ],
        )?;


        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                    to: ctx.accounts.offer_makers_maker_tokens.to_account_info(),
                    // Cute trick: the escrowed_maker_tokens is its own
                    // authority/owner (and a PDA, so our program can sign for
                    // it just below)
                    authority: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                },
                &[&[
                    ctx.accounts.offer.key().as_ref(),
                    &[ctx.accounts.offer.escrowed_maker_tokens_bump],
                ]],
            ),
            1,
        )?;

        // Close the escrow's token account and refund the maker (they paid for
        // its rent-exemption).
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.escrowed_maker_tokens.to_account_info(),
                destination: ctx.accounts.tokenrent.to_account_info(),
                authority: ctx.accounts.escrowed_maker_tokens.to_account_info(),
            },
            &[&[
                ctx.accounts.offer.key().as_ref(),
                &[ctx.accounts.offer.escrowed_maker_tokens_bump],
            ]],
        ))
    }
}

#[account]
pub struct Data {

    pub market_place: Pubkey,
    
    pub market_place_cut: u64,

    pub rent: Pubkey,

    pub rent_cut: u64,
}

#[account]
pub struct Offer {
    // We store the offer maker's key so that they can cancel the offer (we need
    // to know who should sign).
    pub maker: Pubkey,
    
    pub taker_amount: u64,
    // When the maker makes their offer, we store their offered tokens in an
    // escrow account that lives at a program-derived address, with seeds given
    // by the `Offer` account's address. Storing the corresponding bump here
    // means the client doesn't have to keep passing it.
    pub escrowed_maker_tokens_bump: u8,
}

#[derive(Accounts)]
#[instruction(data_bump: u8)]

pub struct Initialize<'info> {
    #[account(init, payer=payer, seeds = [b"data".as_ref()], bump = data_bump, space = 8 + 32 + 8 + 32 + 8)]
    pub data_acc: Account<'info, Data>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account()]

    pub beneficiary: AccountInfo<'info>,

    #[account()]
    pub rent_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

}

#[derive(Accounts)]
#[instruction(escrowed_maker_tokens_bump: u8, offer_bump:u8)]
pub struct Make<'info> {
    #[account(init, payer = offer_maker, seeds = [offer_maker.to_account_info().key.as_ref(), maker_mint.to_account_info().key.as_ref()], bump = offer_bump,  space = 8 + 32 + 8 + 1)]
    pub offer: Account<'info, Offer>,

    #[account(mut)]
    pub offer_maker: Signer<'info>,
    #[account(mut, constraint = offer_makers_maker_tokens.mint == maker_mint.key())]
    pub offer_makers_maker_tokens: Account<'info, TokenAccount>,

    // This is where we'll store the offer maker's tokens.
    #[account(
        init,
        payer = offer_maker,
        seeds = [offer.key().as_ref()],
        bump = escrowed_maker_tokens_bump,
        token::mint = maker_mint,
        // We want the program itself to have authority over the escrow token
        // account, so we need to use some program-derived address here. Well,
        // the escrow token account itself already lives at a program-derived
        // address, so we can set its authority to be its own address.
        token::authority = escrowed_maker_tokens,
    )]
    pub escrowed_maker_tokens: Account<'info, TokenAccount>,

    pub maker_mint: Account<'info, Mint>,

    pub data_acc: Account<'info, Data>,
  
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub tokenrent: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(offer_bump:u8)]
pub struct Accept<'info> {
    #[account(
        mut,
        seeds = [offer_maker.key.as_ref(), maker_mint.to_account_info().key.as_ref()],
        bump = offer_bump,
        // make sure the offer_maker account really is whoever made the offer!
        constraint = offer.maker == *offer_maker.key,
        // at the end of the instruction, close the offer account (don't need it
        // anymore) and send its rent back to the offer_maker
        close = offer_maker
    )]
    pub offer: Box<Account<'info, Offer>>,

    #[account(
        mut,
        seeds = [offer.key().as_ref()],
        bump = offer.escrowed_maker_tokens_bump
    )]
    pub escrowed_maker_tokens: Box<Account<'info, TokenAccount>>,

    pub maker_mint: Account<'info, Mint>,

    #[account(mut)]
    pub offer_maker: AccountInfo<'info>,
    pub offer_taker: Signer<'info>,

 
    #[account(init_if_needed, payer = offer_taker, associated_token::mint = maker_mint, associated_token::authority = offer_taker)]
    pub offer_takers_maker_tokens: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    
    #[account()]
    pub token_metadata_account: AccountInfo<'info>,
    
    #[account()]
    pub token_metadata_program: AccountInfo<'info>,

    #[account(mut)]
    pub market_maker: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub tokenrent: AccountInfo<'info>,

    pub data_acc: Account<'info, Data>,

     #[account(mut)]
    pub creator0: AccountInfo<'info>,

    #[account(mut)]
    pub creator1: AccountInfo<'info>,

    #[account(mut)]
    pub creator2: AccountInfo<'info>,

    #[account(mut)]
    pub creator3: AccountInfo<'info>,

    #[account(mut)]
    pub creator4: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(offer_bump:u8)]
pub struct Cancel<'info> {
    #[account(
        mut,
        seeds = [offer_maker.key.as_ref(), maker_mint.to_account_info().key.as_ref()],
        bump = offer_bump,
        // make sure the offer_maker account really is whoever made the offer!
        constraint = offer.maker == *offer_maker.key,
        // at the end of the instruction, close the offer account (don't need it
        // anymore) and send its rent lamports back to the offer_maker
        close = offer_maker
    )]
    pub offer: Account<'info, Offer>,

    #[account(mut)]
    // the offer_maker needs to sign if they really want to cancel their offer
    pub offer_maker: Signer<'info>,

    #[account(mut)]
    // this is where to send the previously-escrowed tokens to
    pub offer_makers_maker_tokens: Account<'info, TokenAccount>,

    pub maker_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [offer.key().as_ref()],
        bump = offer.escrowed_maker_tokens_bump
    )]
    pub escrowed_maker_tokens: Account<'info, TokenAccount>,


    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    pub data_acc: Account<'info, Data>,

    #[account(mut)]
    pub tokenrent: AccountInfo<'info>
}
