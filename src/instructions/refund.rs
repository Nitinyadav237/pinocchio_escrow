use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_token::{
    instructions::{CloseAccount, Transfer},
    state::TokenAccount,
};

use crate::{
    AccountCheck, AccountClose, AssociatedTokenAccount, AssociatedTokenAccountCheck,
    AssociatedTokenAccountInit, Escrow, MintInterface, ProgramAccount, SignerAccount,
};

pub struct RefundAccounts<'a> {
    pub maker: &'a AccountView,
    pub maker_ata_a: &'a AccountView,
    pub mint_a: &'a AccountView,
    pub escrow_pda: &'a AccountView,
    pub vault_pda: &'a AccountView,
    pub token_program: &'a AccountView,
    pub system_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for RefundAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [maker, escrow_pda, mint_a, vault_pda, maker_ata_a, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        SignerAccount::check(maker)?;
        ProgramAccount::check(escrow_pda)?;
        MintInterface::check(mint_a)?;
        AssociatedTokenAccount::check(vault_pda, escrow_pda, mint_a, token_program)?;

        Ok(Self {
            maker,
            escrow_pda,
            mint_a,
            vault_pda,
            maker_ata_a,
            system_program,
            token_program,
        })
    }
}

pub struct Refund<'a> {
    pub accounts: RefundAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountView]> for Refund<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = RefundAccounts::try_from(accounts)?;
        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_a,
            accounts.mint_a,
            accounts.maker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a> Refund<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow_pda.try_borrow()?;
        let escrow = Escrow::load(&data)?;

        let escrow_key = Address::create_program_address(
            &[
                b"escrow",
                self.accounts.maker.address().as_array(),
                &escrow.seed.to_le_bytes(),
                &escrow.bump,
            ],
            &crate::ID,
        )?;

        if &escrow_key != self.accounts.escrow_pda.address() {
            return Err(ProgramError::InvalidAccountOwner);
        }

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
            to: self.accounts.maker_ata_a,
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
        drop(data);
        ProgramAccount::close(self.accounts.escrow_pda, self.accounts.maker)?;

        Ok(())
    }
}
