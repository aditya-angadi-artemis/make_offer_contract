use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use metaplex_token_metadata;

declare_id!("8WLksD98G274bjxzrPRxLhdLQKbD1YJjJfY6FuTJZtZi");

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

        // let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        //     ctx.accounts.payer.key,
        //     ctx.accounts.tokenrent.to_account_info().key,
        //     10000000000,
        // );

        // anchor_lang::solana_program::program::invoke(
        //     &transfer_ix,
        //     &[
        //         ctx.accounts.payer.to_account_info(),
        //         ctx.accounts.tokenrent.to_account_info(),
        //     ],
        // )?;
        Ok(())
    }

    // Make a binding offer of `offer_maker_amount` of one kind of tokens in
    // exchange for `offer_taker_amount` of some other kind of tokens. This
    // will store the offer maker's tokens in an escrow account.
    pub fn make_offer_for_nft(
        ctx: Context<Make>,
        _offer_bump: u8,
        nft_offer_amount: u64,
        offer_valid: i64,
    ) -> ProgramResult {
        // Store some state about the offer being made. We'll need this later if
        // the offer gets accepted or cancelled.
        msg!("Function start");
        let offer = &mut ctx.accounts.offer;
        offer.maker = ctx.accounts.offer_maker.key();
        offer.offer_amount = nft_offer_amount;
        offer.offer_made_for = ctx.accounts.nft_mint.key();

        let clock = &ctx.accounts.clock;
        msg!("timestamp is {}", clock.unix_timestamp);
        if clock.unix_timestamp >= offer_valid {
            return Err(ProgramError::Custom(0x4));
        }
        offer.offer_valid_till = Some(offer_valid);



        // let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        //     ctx.accounts.offer_maker.key,
        //     ctx.accounts.tokenrent.to_account_info().key,
        //     10385941,
        // );

        // anchor_lang::solana_program::program::invoke(
        //     &transfer_ix,
        //     &[
        //         ctx.accounts.offer_maker.to_account_info(),
        //         ctx.accounts.tokenrent.to_account_info(),
        //         ctx.accounts.offer.to_account_info(),
        //     ],
        // )?;

        msg!("Here");
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.offer_maker.key,
            ctx.accounts.offer.to_account_info().key,
            nft_offer_amount,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.offer_maker.to_account_info(),
                ctx.accounts.tokenrent.to_account_info(),
                ctx.accounts.offer.to_account_info(),
            ],
        )?;

        Ok(())
    }



    // Accept an offer by providing the NFT and accepting the SOL + kind of tokens. This
    // unlocks the tokens escrowed by the offer maker.
    pub fn accept(ctx: Context<Accept>, offer_bump:u8, _data_bump:u8) -> ProgramResult {
        
       
       let mut taker_amount = ctx.accounts.offer.offer_amount;
       // Multi by 10
       let market_cut = ctx.accounts.data_acc.market_place_cut * taker_amount / 1000;
       let sfb = metaplex_token_metadata::state::Metadata::from_account_info(&ctx.accounts.token_metadata_account)?.data.seller_fee_basis_points;
       let sfb_cut = sfb as u64 * taker_amount / 10000;
       taker_amount = taker_amount - (market_cut + sfb_cut);
        
       let clock = &ctx.accounts.clock;
       if clock.unix_timestamp >= ctx.accounts.offer.offer_valid_till.unwrap() {
           return Err(ProgramError::Custom(0x4));
       }
        if *ctx.accounts.tokenrent.key != ctx.accounts.data_acc.rent {
            return Err(ProgramError::Custom(0x1));
        }

        if *ctx.accounts.offer_maker.key != ctx.accounts.offer.maker {
            return Err(ProgramError::Custom(0x11));
        }

        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.offer_takers_nft_token.to_account_info(),
                    to: ctx.accounts.offer_makers_nft_account.to_account_info(),
                    // The offer_maker had to sign from the client
                    authority: ctx.accounts.offer_taker.to_account_info(),
                },
            ),
            1,
        )?;
        //Transfer to Offer Taker
        **ctx.accounts.offer.to_account_info().try_borrow_mut_lamports()? -= taker_amount;
        **ctx.accounts.offer_taker.to_account_info().try_borrow_mut_lamports()? += taker_amount;

        if *ctx.accounts.market_maker.key != ctx.accounts.data_acc.market_place {
            return Err(ProgramError::Custom(0x1));
        }
        
        //Transfer to Market Maker
        **ctx.accounts.offer.to_account_info().try_borrow_mut_lamports()? -= market_cut;
        **ctx.accounts.market_maker.to_account_info().try_borrow_mut_lamports()? += market_cut;

 

        if sfb_cut > 0 {    
      
            if let Some(x) = metaplex_token_metadata::state::Metadata::from_account_info(&ctx.accounts.token_metadata_account)?.data.creators {
                let mut y = 0;

            for i in x {
                    if y == 0 {
                        if i.address != *ctx.accounts.creator0.key {
                            return Err(ProgramError::Custom(0x1));
                        }

                        let temp =  sfb_cut as u64 * i.share as u64 / 100;
                        **ctx.accounts.offer.to_account_info().try_borrow_mut_lamports()? -= temp;
                        **ctx.accounts.creator0.to_account_info().try_borrow_mut_lamports()? += temp;
                    }
                    else if y == 1 {
                        if i.address != *ctx.accounts.creator1.key {
                            return Err(ProgramError::Custom(0x1));
                        }
                                      
                        let temp =  sfb_cut as u64 * i.share as u64 / 100;
                        **ctx.accounts.offer.to_account_info().try_borrow_mut_lamports()? -= temp;
                        **ctx.accounts.creator1.to_account_info().try_borrow_mut_lamports()? += temp;
                    }
                    else if y == 2 {
                        if i.address != *ctx.accounts.creator2.key {
                            return Err(ProgramError::Custom(0x1));
                        }
       
                        let temp =  sfb_cut as u64 * i.share as u64 / 100;
                    
                        **ctx.accounts.offer.to_account_info().try_borrow_mut_lamports()? -= temp;
                        **ctx.accounts.creator2.to_account_info().try_borrow_mut_lamports()? += temp;
                    }
                    else if y == 3 {
                        if i.address != *ctx.accounts.creator3.key {
                            return Err(ProgramError::Custom(0x1));
                        }

                        let temp =  sfb_cut as u64 * i.share as u64 / 100;
                     
                        **ctx.accounts.offer.to_account_info().try_borrow_mut_lamports()? -= temp;
                        **ctx.accounts.creator3.to_account_info().try_borrow_mut_lamports()? += temp;
                    }
                    else if y == 4 {
                        if i.address != *ctx.accounts.creator1.key {
                            return Err(ProgramError::Custom(0x1));
                        }

        
                        let temp =  sfb_cut as u64 * i.share as u64 / 100;
                        
                        **ctx.accounts.offer.to_account_info().try_borrow_mut_lamports()? -= temp;
                        **ctx.accounts.creator4.to_account_info().try_borrow_mut_lamports()? += temp;
                    }
                    y = y + 1;

            }
     
            }

        }
        msg!("Doing transfer");
        msg!("from: {}", ctx.accounts.offer_takers_nft_token.to_account_info().key);
        msg!("to: {}", ctx.accounts.offer_makers_nft_account.to_account_info().key);
        msg!("auth: {}", ctx.accounts.offer_taker.to_account_info().key);

        Ok(())
 
    }

    pub fn cancel(ctx: Context<Cancel>, _offer_bump:u8) -> ProgramResult {

        let temp =  ctx.accounts.offer.offer_amount;
        **ctx.accounts.offer.to_account_info().try_borrow_mut_lamports()? -= temp;
        **ctx.accounts.offer_maker.to_account_info().try_borrow_mut_lamports()? += temp;

        Ok(())
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

    pub offer_made_for: Pubkey,
    
    pub offer_amount: u64,

    pub offer_valid_till: Option<i64>,

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

    #[account(mut)]
    pub tokenrent: AccountInfo<'info>

}

#[derive(Accounts)]
#[instruction(offer_bump:u8)]
pub struct Make<'info> {
    #[account(init, payer = offer_maker, seeds = [offer_maker.to_account_info().key.as_ref(), nft_mint.to_account_info().key.as_ref()], bump = offer_bump,  space = 8 + 32 + 32 + 32 + 8 + 1 + 128)]
    pub offer: Account<'info, Offer>,

    #[account(mut)]
    pub offer_maker: Signer<'info>,

    pub nft_mint: Account<'info, Mint>,

    #[account(init_if_needed, payer = offer_maker, associated_token::mint = nft_mint, associated_token::authority = offer_maker)]
    pub offer_makers_nft_account: Box<Account<'info, TokenAccount>>,

    pub data_acc: Account<'info, Data>,
  
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,

    #[account(mut)]
    pub tokenrent: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(offer_bump:u8, data_bump:u8)]
pub struct Accept<'info> {
    #[account(
        mut,
        seeds = [offer_maker.key.as_ref(), nft_mint.to_account_info().key.as_ref()],
        bump = offer_bump,
        // make sure the offer_maker account really is whoever made the offer!
        constraint = offer.maker == *offer_maker.key,
        // at the end of the instruction, close the offer account (don't need it
        // anymore) and send its rent back to the offer_maker
        close = tokenrent
    )]
    pub offer: Box<Account<'info, Offer>>,

    pub nft_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub offer_maker: AccountInfo<'info>,

    #[account(mut)]
    pub offer_taker: Signer<'info>,

    #[account(mut, constraint = offer_takers_nft_token.mint == nft_mint.key(), close = tokenrent)]
    pub offer_takers_nft_token: Box<Account<'info, TokenAccount>>,

 
    #[account( mut, constraint = offer_makers_nft_account.mint == nft_mint.key())]
    pub offer_makers_nft_account: Box<Account<'info, TokenAccount>>,

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
    pub clock: Sysvar<'info, Clock>,

    #[account(mut)]
    pub tokenrent: AccountInfo<'info>,

    #[account(seeds = [b"data".as_ref()], bump = data_bump)]
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
    pub creator4: AccountInfo<'info>,

}

#[derive(Accounts)]
#[instruction(offer_bump:u8)]
pub struct Cancel<'info> {
    #[account(
        mut,
        seeds = [offer_maker.key.as_ref(), nft_mint.to_account_info().key.as_ref()],
        bump = offer_bump,
        // make sure the offer_maker account really is whoever made the offer!
        constraint = offer.maker == *offer_maker.key,
        // at the end of the instruction, close the offer account (don't need it
        // anymore) and send its rent lamports back to the offer_maker
        close = tokenrent
    )]
    pub offer: Account<'info, Offer>,

    #[account(mut)]
    // the offer_maker needs to sign if they really want to cancel their offer
    pub offer_maker: Signer<'info>,



    pub nft_mint: Account<'info, Mint>,


    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    pub data_acc: Account<'info, Data>,

    pub clock: Sysvar<'info, Clock>,

    #[account(mut)]
    pub tokenrent: AccountInfo<'info>
}
