use crate::raydium_launchpad::{
    cpi::{
        accounts::{BuyExactIn, InitializeV2},
        buy_exact_in, initialize_v2,
    },
    program::RaydiumLaunchpad,
    types::{AmmFeeOn, ConstantCurve, CurveParams, MintParams, VestingParams},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata,
    token::{Mint, Token, TokenAccount},
};

declare_id!("CJXZXURU7q9v1Tsz9gci2TNXbyxsVqq7k3Ja7mcsUVqk");
declare_program!(raydium_launchpad);

pub mod seeds {
    pub const CONFIG_SEED: &[u8] = b"global_config";
    pub const POOL_SEED: &[u8] = b"pool";
    pub const POOL_VAULT_SEED: &[u8] = b"pool_vault";
    pub const AUTH_SEED: &[u8] = b"vault_auth_seed";
    pub const EVENT_AUTHORITY: &[u8] = b"__event_authority";
    pub const METADATA_SEED: &[u8] = b"metadata";
}

#[program]
pub mod launchlabs_buy {
    use super::*;

    pub fn create<'info>(ctx: Context<'_, '_, '_, 'info, Create<'info>>) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.raydium_launchpad_program.to_account_info(),
            InitializeV2 {
                payer: ctx.accounts.user.to_account_info(),
                creator: ctx.accounts.user.to_account_info(),
                global_config: ctx.accounts.global_config.to_account_info(),
                platform_config: ctx.accounts.platform_config.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                pool_state: ctx.accounts.pool_state.to_account_info(),
                base_mint: ctx.accounts.base_token_mint.to_account_info(),
                quote_mint: ctx.accounts.quote_token_mint.to_account_info(),
                base_vault: ctx.accounts.base_vault.to_account_info(),
                quote_vault: ctx.accounts.quote_vault.to_account_info(),
                metadata_account: ctx.accounts.metadata_account.to_account_info(),
                base_token_program: ctx.accounts.token_program.to_account_info(),
                quote_token_program: ctx.accounts.token_program.to_account_info(),
                metadata_program: ctx.accounts.metadata_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent_program: ctx.accounts.rent_program.to_account_info(),
                event_authority: ctx.accounts.event_authority.to_account_info(),
                program: ctx.accounts.raydium_launchpad_program.to_account_info(),
            },
        );

        let params = MintParams {
            name: "Glamorous Cats".to_string(),
            symbol: "GLAM".to_string(),
            uri: "https://ipfs.io/ipfs/bafkreig2atoub5hapy5s7ymoomwhrttymhicthuftjkcyq6jgyf4yoekx4"
                .to_string(),
            decimals: 6,
        };

        // Align Raydium curve with the store so that t_out(F) equals the allocated base (D)
        // - total_base_sell (D) = total tokens allocated by the store across drops (1e6 units)
        // - total_quote_fund_raising (F) = total quote allocated/raised (lamports for SOL)
        let curve_params = CurveParams::Constant {
            data: ConstantCurve {
                supply: 1000000000000000,
                total_base_sell: 793100000000000,
                total_quote_fund_raising: 12500000000,
                migrate_type: 1,
            },
        };

        let vesting_params = VestingParams {
            total_locked_amount: 0,
            cliff_period: 0,
            unlock_period: 0,
        };

        let amm_fee_on = AmmFeeOn::QuoteToken;

        initialize_v2(cpi_ctx, params, curve_params, vesting_params, amm_fee_on)?;

        Ok(())
    }

    pub fn buy<'info>(
        ctx: Context<'_, '_, '_, 'info, Buy<'info>>,
        amount_in: u64,
        minimum_amount_out: u64,
        share_fee_rate: u64,
    ) -> Result<()> {
        let accounts_infos: Vec<AccountInfo> = ctx
            .remaining_accounts
            .iter()
            .map(|acc| AccountInfo { ..acc.clone() })
            .collect();

        let cpi_ctx = CpiContext::new(
            ctx.accounts.raydium_launchpad_program.to_account_info(),
            BuyExactIn {
                payer: ctx.accounts.user.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                global_config: ctx.accounts.global_config.to_account_info(),
                platform_config: ctx.accounts.platform_config.to_account_info(),
                pool_state: ctx.accounts.pool_state.to_account_info(),
                user_base_token: ctx.accounts.user_base_token.to_account_info(),
                user_quote_token: ctx.accounts.user_quote_token.to_account_info(),
                base_vault: ctx.accounts.base_vault.to_account_info(),
                quote_vault: ctx.accounts.quote_vault.to_account_info(),
                base_token_mint: ctx.accounts.base_token_mint.to_account_info(),
                quote_token_mint: ctx.accounts.quote_token_mint.to_account_info(),
                base_token_program: ctx.accounts.token_program.to_account_info(),
                quote_token_program: ctx.accounts.token_program.to_account_info(),
                event_authority: ctx.accounts.event_authority.to_account_info(),
                program: ctx.accounts.raydium_launchpad_program.to_account_info(),
            },
        )
        .with_remaining_accounts(accounts_infos);

        buy_exact_in(cpi_ctx, amount_in, minimum_amount_out, share_fee_rate)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Create<'info> {
    pub user: Signer<'info>,

    /// CHECK: checked by cpi
    #[account(
        seeds = [seeds::AUTH_SEED],
        bump,
        seeds::program = raydium_launchpad_program
    )]
    pub authority: AccountInfo<'info>,

    /// CHECK: raydium program checksS
    #[account(
        seeds = [
            seeds::CONFIG_SEED,
            quote_token_mint.key().as_ref(),
            0u8.to_le_bytes().as_ref(),
            0u16.to_le_bytes().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub global_config: AccountInfo<'info>,
    /// CHECK: raydium program checks
    pub platform_config: AccountInfo<'info>,

    /// CHECK: checked by cpi
    #[account(
        mut,
        seeds = [
            seeds::POOL_SEED,
            base_token_mint.key().as_ref(),
            quote_token_mint.key().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub pool_state: AccountInfo<'info>,

    /// CHECK: checked by cpi
    #[account(
        mut,
        seeds = [
            seeds::POOL_VAULT_SEED,
            pool_state.key().as_ref(),
            base_token_mint.key().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub base_vault: AccountInfo<'info>,

    /// CHECK: checked by cpi
    #[account(
        mut,
        seeds = [
            seeds::POOL_VAULT_SEED,
            pool_state.key().as_ref(),
            quote_token_mint.key().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub quote_vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            seeds::METADATA_SEED,
            metadata_program.key().as_ref(),
            base_token_mint.key().as_ref(),
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub metadata_account: SystemAccount<'info>,

    #[account(mut)]
    pub base_token_mint: Signer<'info>,
    pub quote_token_mint: Account<'info, Mint>,

    /// CHECK: checked by cpi
    #[account(
        seeds = [seeds::EVENT_AUTHORITY],
        bump,
        seeds::program = raydium_launchpad_program
    )]
    pub event_authority: AccountInfo<'info>,

    pub rent_program: Sysvar<'info, Rent>,
    pub metadata_program: Program<'info, Metadata>,
    pub raydium_launchpad_program: Program<'info, RaydiumLaunchpad>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    pub user: Signer<'info>,

    /// CHECK: checked by cpi
    #[account(
        seeds = [seeds::AUTH_SEED],
        bump,
        seeds::program = raydium_launchpad_program
    )]
    pub authority: AccountInfo<'info>,

    /// CHECK: raydium program checksS
    #[account(
        seeds = [
            seeds::CONFIG_SEED,
            quote_token_mint.key().as_ref(),
            0u8.to_le_bytes().as_ref(),
            0u16.to_le_bytes().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub global_config: AccountInfo<'info>,
    /// CHECK: raydium program checks
    pub platform_config: AccountInfo<'info>,

    /// CHECK: checked by cpi
    #[account(
        mut,
        seeds = [
            seeds::POOL_SEED,
            base_token_mint.key().as_ref(),
            quote_token_mint.key().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub pool_state: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = base_token_mint,
        associated_token::authority = user
    )]
    pub user_base_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = quote_token_mint,
        associated_token::authority = user
    )]
    pub user_quote_token: Account<'info, TokenAccount>,

    /// CHECK: checked by cpi
    #[account(
        mut,
        seeds = [
            seeds::POOL_VAULT_SEED,
            pool_state.key().as_ref(),
            base_token_mint.key().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub base_vault: AccountInfo<'info>,

    /// CHECK: checked by cpi
    #[account(
        mut,
        seeds = [
            seeds::POOL_VAULT_SEED,
            pool_state.key().as_ref(),
            quote_token_mint.key().as_ref(),
        ],
        seeds::program = raydium_launchpad_program.key(),
        bump,
    )]
    pub quote_vault: AccountInfo<'info>,

    pub base_token_mint: Account<'info, Mint>,
    pub quote_token_mint: Account<'info, Mint>,

    /// CHECK: checked by cpi
    #[account(
        seeds = [seeds::EVENT_AUTHORITY],
        bump,
        seeds::program = raydium_launchpad_program
    )]
    pub event_authority: AccountInfo<'info>,

    pub raydium_launchpad_program: Program<'info, RaydiumLaunchpad>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
