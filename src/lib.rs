use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use std::str::FromStr;

fn system_program_id() -> Pubkey {
    Pubkey::from_str("11111111111111111111111111111111").unwrap()
}

entrypoint!(process_instruction);

#[derive(BorshSerialize, BorshDeserialize)]
#[allow(dead_code)]

struct VaultAccount {
    version: u8,
    depositor: Pubkey,
    receiver: Pubkey,
    amount: u64,
}

impl VaultAccount {
    pub const LEN: usize = 1 + 32 + 32 + 8;
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let accounts_array = &mut accounts.iter();

    let vault_pda_account = next_account_info(accounts_array)?;
    let user_wallet = next_account_info(accounts_array)?;
    let system_program_input = next_account_info(accounts_array)?;

    if !user_wallet.is_signer {
        return Err(solana_program::program_error::ProgramError::InvalidArgument);
    }

    if system_program_input.key != &system_program_id() {
        return Err(solana_program::program_error::ProgramError::IncorrectProgramId);
    }

    msg!("Vault program reached");

    let (len1, rest) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    let seed1 = &rest[..*len1 as usize];
    let rest = &rest[*len1 as usize..];

    let (len2, rest) = rest
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    let seed2 = &rest[..*len2 as usize];
    let client_bump = &rest[*len2 as usize];

    let (pda_found, derived_bump) = Pubkey::find_program_address(&[seed1, seed2], program_id);

    if *client_bump != derived_bump {
        return Err(ProgramError::InvalidArgument);
    }

    if pda_found != *vault_pda_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    msg!("PDA verified");

    let seeds: &[&[u8]] = &[seed1, seed2, &[*client_bump]];

    if vault_pda_account.lamports() == 0 || vault_pda_account.data_len() == 0 {
        msg!("Creating a PDA in vault");
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(VaultAccount::LEN);

        let tx = system_instruction::create_account(
            &user_wallet.key,
            &pda_found,
            lamports,
            VaultAccount::LEN as u64,
            &program_id,
        );

        invoke_signed(
            &tx,
            &[
                vault_pda_account.clone(),
                user_wallet.clone(),
                system_program_input.clone(),
            ],
            &[seeds],
        )?;

        msg!("Vault account created");

        if vault_pda_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let mut data = vault_pda_account.try_borrow_mut_data()?;

        VaultAccount {
            version: 1,
            depositor: *user_wallet.key,
            receiver: *user_wallet.key,
            amount: 0,
        }
        .serialize(&mut &mut data[..])?;

        msg!("Vault initialized");
    } else {
        msg!("Vault already Exists");
    }

    Ok(())
}
