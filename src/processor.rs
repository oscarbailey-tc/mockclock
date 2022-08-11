use solana_program::{
    entrypoint::ProgramResult,
    account_info::AccountInfo,
    pubkey::Pubkey,
    msg,
    program::invoke_signed,
    system_instruction,
    sysvar::Sysvar,
};

/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let clock_acc = &accounts[0];
    let rent_payer = &accounts[1];
    let system_program = &accounts[2];

    let (expected_addr, bump) = Pubkey::find_program_address(&[b"clock"], program_id);
    assert!(*clock_acc.key == expected_addr);

    let new_clock_bytes: [u8; 8] = instruction_data.try_into().expect("Invalid instruction data");
    let new_clock = u64::from_le_bytes(new_clock_bytes);
    msg!("Setting new clock to {}...", new_clock);

    if clock_acc.data_len() == 0 {
        // Create clock account
        let rent = solana_program::rent::Rent::get()?;
        let lamports = rent.minimum_balance(8);
        invoke_signed(
            &system_instruction::create_account(
                rent_payer.key,
                clock_acc.key,
                lamports,
                8,
                program_id,
            ),
            &[
                clock_acc.clone(),
                rent_payer.clone(),
                system_program.clone(),
            ],
            &[&[b"clock", &[bump]]],
        )?;
    }

    clock_acc.try_borrow_mut_data()?.copy_from_slice(&new_clock_bytes);

    Ok(())
}
