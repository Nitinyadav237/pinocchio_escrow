# Pinocchio v0.9 → v0.10 Migration Guide

Quick reference for migrating the Blueshift Escrow program from Pinocchio v0.9 to v0.10.

---

## ⚠️ Important Dependency

To use `Address::find_program_address()` and `Address::create_program_address()`, you **must** add this to your `Cargo.toml`:

```toml
solana-address = { version = "2.0.0", features = ["curve25519"] }
```

Without this dependency, these methods won't be available via `Address::`.

---

## 🎯 Why These Changes?

Pinocchio v0.10 aligns with Solana SDK types for better interoperability:
- **AccountView** replaces custom `AccountInfo` (eliminates name clashes)
- **Address** replaces `Pubkey` alias (matches Solana SDK's new `Address` type)
- **pinocchio-pubkey crate removed** (functionality now in `solana-address`)
- **ProgramError** from SDK (no more duplication)

**Result:** Same performance, better compatibility, less confusion.

---

## 📋 Quick Reference

| What | v0.9 | v0.10 |
|------|------|-------|
| **Types** |
| Account type | `AccountInfo` | `AccountView` |
| Pubkey type | `Pubkey` | `Address` |
| **Methods** |
| Get account key | `.key()` | `.address()` |
| Key as bytes | `.key()` | `.address().as_array()` |
| Key as slice | `.key().as_ref()` | `.address().as_ref()` |
| Borrow data | `.try_borrow_data()` | `.try_borrow()` |
| Borrow data (mut) | `.try_borrow_mut_data()` | `.try_borrow_mut()` |
| Find PDA | `find_program_address(...)` | `Address::find_program_address(...)` |
| Create PDA | `create_program_address(...)` | `Address::create_program_address(...)` |
| Load token account | `TokenAccount::from_account_info(...)` | `TokenAccount::from_account_view(...)` |
| **Imports** |
| Seed, Signer | `instruction::{Seed, Signer}` | `cpi::{Seed, Signer}` |
| ProgramError | `program_error::ProgramError` | `error::ProgramError` |
| Pubkey crate | `pinocchio_pubkey::*` | ❌ Removed |

---

## 🔄 Migration Examples

### Imports
```rust
// v0.9
use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError};
use pinocchio_pubkey::{find_program_address, create_program_address, Pubkey};

// v0.10
use pinocchio::{cpi::{Seed, Signer}, error::ProgramError, AccountView, Address};
```

### Struct Definition
```rust
// v0.9
pub struct MakeAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
}

// v0.10
pub struct MakeAccounts<'a> {
    pub maker: &'a AccountView,
    pub escrow_pda: &'a AccountView,
}
```

### PDA Operations
```rust
// v0.9
let (pda, bump) = find_program_address(&[b"escrow", maker.key(), &seed], &id);
let key = create_program_address(&[b"escrow", maker.key(), &seed, &bump], &id)?;

// v0.10
let (pda, bump) = Address::find_program_address(&[b"escrow", maker.address().as_array(), &seed], &id);
let key = Address::create_program_address(&[b"escrow", maker.address().as_array(), &seed, &bump], &id)?;
```

### Account Data
```rust
// v0.9
let data = account.try_borrow_data()?;
let mut data = account.try_borrow_mut_data()?;
let amount = TokenAccount::from_account_info(vault)?.amount();

// v0.10
let data = account.try_borrow()?;
let mut data = account.try_borrow_mut()?;
let amount = TokenAccount::from_account_view(vault_pda)?.amount();
```

### Seeds for CPI
```rust
// v0.9
let seeds = [
    Seed::from(b"escrow"),
    Seed::from(maker.key().as_ref()),
    Seed::from(&seed_bytes),
];

// v0.10
let seeds = [
    Seed::from(b"escrow"),
    Seed::from(maker.address().as_ref()),
    Seed::from(&seed_bytes),
];
```

---

## ✅ Migration Checklist

- [ ] Add `solana-address = { version = "2.0.0", features = ["curve25519"] }` to `Cargo.toml`
- [ ] Remove `pinocchio-pubkey` from `Cargo.toml`
- [ ] Replace all `AccountInfo` → `AccountView`
- [ ] Replace all `Pubkey` → `Address`
- [ ] Update imports:
  - [ ] `instruction::` → `cpi::`
  - [ ] `program_error::` → `error::`
  - [ ] Remove `use pinocchio_pubkey::*`
- [ ] Update method calls:
  - [ ] `.key()` → `.address()`
  - [ ] `.try_borrow_data()` → `.try_borrow()`
  - [ ] `.try_borrow_mut_data()` → `.try_borrow_mut()`
  - [ ] `find_program_address()` → `Address::find_program_address()`
  - [ ] `create_program_address()` → `Address::create_program_address()`
  - [ ] `from_account_info()` → `from_account_view()`
- [ ] Update PDA seeds: `.key()` → `.address().as_array()` or `.address().as_ref()`

---

## 🎉 Done!

All changes are API-level only. Your program logic stays the same, with better SDK compatibility and zero performance impact.