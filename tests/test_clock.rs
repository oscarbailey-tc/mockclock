// Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
#![cfg(feature = "test-sbf")]
#![cfg(test)]

use std::str::FromStr;

use solana_program::{pubkey::Pubkey, instruction::{Instruction, AccountMeta}, clock::Clock};
use solana_sdk::{signer::Signer, transaction::Transaction, signature::Keypair};

use mock_clock::processor::process_instruction;

use solana_program_test::*;

pub const ID: &str = "6o2E5vCAzGhKh3Dq6eqy5Cqxy4Eo4nPjjGHw8tou1M82";

pub fn id() -> Pubkey {
    return Pubkey::from_str(ID).unwrap();
}

pub fn get_prand(seed: &mut i64) -> i64 {
    let new_rand = ((*seed * 1103515245) + 12345) & 0x7fffffff;
    *seed = new_rand;
    return new_rand;
}

pub fn get_clock_prand(seed: &mut i64) -> Clock {
    Clock {
        epoch: get_prand(seed) as u64,
        epoch_start_timestamp: get_prand(seed),
        leader_schedule_epoch: get_prand(seed) as u64,
        slot: get_prand(seed) as u64,
        unix_timestamp: get_prand(seed)
    }
}

#[tokio::test]
async fn test_init() {
    let pc = ProgramTest::new("mock_clock", id(), processor!(process_instruction));
    let (mut banks_client, payer, _recent_blockhash) = pc.start().await;

    let acc = banks_client.get_account(get_clock_address()).await.unwrap();
    assert!(acc.is_none());

    let mut seed = 384234;
    for _ in 0..10 {
        let clock = get_clock_prand(&mut seed);
        set_clock(&payer, &mut banks_client, &clock).await;
        assert!(get_clock(&mut banks_client).await == clock);
    }
}

fn get_clock_address() -> Pubkey {
    let (state, _bump) = Pubkey::find_program_address(&[b"clock"], &id());
    return state
}

async fn get_clock(banks_client: &mut BanksClient) -> Clock {
    let acc = banks_client.get_account(get_clock_address()).await.unwrap().unwrap();
    bincode::deserialize(&acc.data).unwrap()
}

pub async fn set_clock(payer: &Keypair, banks_client: &mut BanksClient, clock: &Clock) {
    let mut instructions = vec![];
    let (clock_address, _) = Pubkey::find_program_address(&[b"clock"], &id());

    instructions.push(Instruction{
        program_id: id(),
        data: bincode::serialize(&clock).unwrap(),
        accounts: vec![
            AccountMeta::new(clock_address, false),
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
