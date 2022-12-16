# Mock Clock

Tiny program that can be used to mock the clock sysvar.

If you need to mock the clock sysvar, create a function that reads this account instead of the sysvar depending on build
env:

In your program:
```rust
use solana_program::{account_info::AccountInfo, clock::Clock, program_error::ProgramError, sysvar::SysvarId, msg};

/// Are we running a test?
pub fn is_test() -> bool {
    option_env!("TEST").is_some()
}

/// Get the Clock from the account passed in. Check for clock sysvar is skipped in a test
/// environment.
fn get_clock(acc: &AccountInfo) -> Result<Clock, ProgramError> {
    if !is_test() && !Clock::check_id(acc.unsigned_key()) {
        msg!("clock_sysvar account key {} is incorrect", acc.key);
        return Err(ProgramError::InvalidArgument);
    }
    bincode::deserialize(&acc.data.borrow()).map_err(|_| ProgramError::InvalidArgument)
}
```

In solana-program-test:
```rust
pub async fn set_clock(payer: &Keypair, banks_client: &mut BanksClient, clock: &Clock) {
    let mut instructions = vec![];

    instructions.push(Instruction{
        program_id: id(),
        data: bincode::serialize(&clock).unwrap(),
        accounts: vec![
            AccountMeta::new(get_clock_address(), false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    });

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer],
        banks_client.get_latest_blockhash().await.unwrap(),
    );
    banks_client.process_transaction(tx).await.unwrap();
}
```
