// Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
#![cfg(feature = "test-bpf")]
#![cfg(test)]

use std::str::FromStr;

use solana_program::{pubkey::Pubkey, instruction::{Instruction, AccountMeta}};
use solana_sdk::{signer::Signer, transaction::Transaction, signature::Keypair};

use mock_clock::processor::process_instruction;

use solana_program_test::*;

pub const ID: &str = "6o2E5vCAzGhKh3Dq6eqy5Cqxy4Eo4nPjjGHw8tou1M82";

pub fn id() -> Pubkey {
    return Pubkey::from_str(ID).unwrap();
}

#[tokio::test]
async fn test_init() {
    let pc = ProgramTest::new("mock_clock", id(), processor!(process_instruction));
    let (mut banks_client, payer, _recent_blockhash) = pc.start().await;

    let acc = banks_client.get_account(get_clock_address()).await.unwrap();
    assert!(acc.is_none());

    set_clock(&payer, &mut banks_client, 100).await;

    assert!(get_clock(&mut banks_client).await == 100);

    set_clock(&payer, &mut banks_client, 28234982).await;

    assert!(get_clock(&mut banks_client).await == 28234982);
}

fn get_clock_address() -> Pubkey {
    let (state, _bump) = Pubkey::find_program_address(&[b"clock"], &id());
    return state
}

async fn get_clock(banks_client: &mut BanksClient) -> u64 {
    let acc = banks_client.get_account(get_clock_address()).await.unwrap().unwrap();

    let clock_bytes: [u8; 8] = acc.data.try_into().unwrap();
    let clock_val = u64::from_le_bytes(clock_bytes);

    return clock_val;
}

pub async fn set_clock(payer: &Keypair, banks_client: &mut BanksClient, time: u64) {
    let mut instructions = vec![];

    println!("Payer: {}", payer.pubkey());

    instructions.push(Instruction{
        program_id: id(),
        data: time.to_le_bytes().to_vec(),
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
