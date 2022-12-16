use bincode::Options;
use solana_program::{
    entrypoint::ProgramResult,
    account_info::AccountInfo,
    pubkey::Pubkey,
    msg,
    program::invoke_signed,
    system_instruction,
    sysvar::Sysvar,
    clock::Clock, program_error::ProgramError,
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

    // Don't allow trailing bytes - instruction_data len must == Clock struct length
    let clock_ins: Clock = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .deserialize(&instruction_data)
        .map_err(|_| ProgramError::InvalidArgument)?;
    let clock_length = instruction_data.len();

    msg!("New Clock - slot: {}, epoch_start_timestamp: {}, epoch: {}, leader_schedule_epoch: {}, unix_timestamp: {}", 
         clock_ins.slot,
         clock_ins.epoch_start_timestamp,
         clock_ins.epoch,
         clock_ins.leader_schedule_epoch,
         clock_ins.unix_timestamp,
    );

    if clock_acc.data_len() == 0 {
        // Create clock account
        let rent = solana_program::rent::Rent::get()?;
        let lamports = rent.minimum_balance(instruction_data.len());
        invoke_signed(
            &system_instruction::create_account(
                rent_payer.key,
                clock_acc.key,
                lamports,
                clock_length.try_into().unwrap(),
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

    clock_acc.try_borrow_mut_data()?.copy_from_slice(instruction_data);

    Ok(())
}
