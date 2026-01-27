use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_token::{
    instructions::{CloseAccount, Transfer},
    state::TokenAccount,
};

use crate::{
    AccountCheck, AccountClose, AssociatedTokenAccount, AssociatedTokenAccountCheck,
    AssociatedTokenAccountInit, Escrow, MintInterface, ProgramAccount, SignerAccount,
};

pub struct TakeAccounts<'a> {
    pub taker: &'a AccountView,
    pub maker: &'a AccountView,
    pub escrow_pda: &'a AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub vault_pda: &'a AccountView,
    pub taker_ata_a: &'a AccountView,
    pub taker_ata_b: &'a AccountView,
    pub maker_ata_b: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for TakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [taker, maker, escrow_pda, mint_a, mint_b, vault_pda, taker_ata_a, taker_ata_b, maker_ata_b, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // account checks
        SignerAccount::check(taker)?;
        ProgramAccount::check(escrow_pda)?;
        MintInterface::check(mint_a)?;
        MintInterface::check(mint_b)?;
        AssociatedTokenAccount::check(taker_ata_b, taker, mint_b, token_program)?;
        AssociatedTokenAccount::check(vault_pda, escrow_pda, mint_a, token_program)?;

        Ok(Self {
            taker,
            maker,
            escrow_pda,
            mint_a,
            mint_b,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            vault_pda,
            system_program,
            token_program,
        })
    }
}

pub struct Take<'a> {
    pub accounts: TakeAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountView]> for Take<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = TakeAccounts::try_from(accounts)?;

        AssociatedTokenAccount::init_if_needed(
            accounts.taker_ata_a,
            accounts.mint_a,
            accounts.taker,
            accounts.taker,
            accounts.system_program,
            accounts.token_program,
        )?;
        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_b,
            accounts.mint_b,
            accounts.taker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a> Take<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow_pda.try_borrow()?;

        let escrow = Escrow::load(&data)?;
        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;

        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];

        let signer = Signer::from(&escrow_seeds);
        let amount = TokenAccount::from_account_view(self.accounts.vault_pda)?.amount();

        Transfer {
            from: self.accounts.vault_pda,
            to: self.accounts.taker_ata_a,
            authority: self.accounts.escrow_pda,
            amount,
        }
        .invoke_signed(&[signer.clone()])?;

        CloseAccount {
            account: self.accounts.vault_pda,
            destination: self.accounts.maker,
            authority: self.accounts.escrow_pda,
        }
        .invoke_signed(&[signer.clone()])?;

        Transfer {
            from: self.accounts.taker_ata_b,
            to: self.accounts.maker_ata_b,
            authority: self.accounts.taker,
            amount: escrow.receive,
        }
        .invoke()?;

        drop(data);
        ProgramAccount::close(self.accounts.escrow_pda, self.accounts.taker)?;
        Ok(())
    }
}
