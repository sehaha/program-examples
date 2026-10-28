use {
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    },
    solana_system_interface::instruction as system_instruction,
    spl_associated_token_account_interface::instruction as associated_token_account_instruction,
    spl_token_interface::{instruction as token_instruction, state::Mint},
};

use crate::{
    instructions::AUTHORITY_SEED,
    mpl_util::{create_master_edition_v3, create_metadata_account_v3},
};

/// Creates a collection NFT: a 0-decimal mint whose authority is the program's
/// `[b"authority"]` PDA, with Metaplex metadata (marked as a sized collection)
/// and a master edition. The single token is minted to the user's ATA.
pub fn create_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let user = next_account_info(accounts_iter)?;
    let mint = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;
    let metadata = next_account_info(accounts_iter)?;
    let master_edition = next_account_info(accounts_iter)?;
    let destination = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;
    let token_metadata_program = next_account_info(accounts_iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda, bump) = Pubkey::find_program_address(&[AUTHORITY_SEED], program_id);
    if mint_authority.key != &pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(Mint::LEN);

    msg!("Creating mint account");
    invoke(
        &system_instruction::create_account(user.key, mint.key, lamports, Mint::LEN as u64, token_program.key),
        &[user.clone(), mint.clone(), system_program.clone(), token_program.clone()],
    )?;

    msg!("Initializing mint account");
    invoke(
        &token_instruction::initialize_mint2(
            token_program.key,
            mint.key,
            mint_authority.key,
            Some(mint_authority.key),
            0,
        )?,
        std::slice::from_ref(mint),
    )?;

    let bump_bytes = [bump];
    let signer_seeds: &[&[u8]] = &[AUTHORITY_SEED, &bump_bytes];

    msg!("Creating destination token account");
    invoke(
        &associated_token_account_instruction::create_associated_token_account_idempotent(
            user.key,
            user.key,
            mint.key,
            token_program.key,
        ),
        &[
            user.clone(),
            destination.clone(),
            user.clone(),
            mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    msg!("Minting collection NFT");
    invoke_signed(
        &token_instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            mint_authority.key,
            &[mint_authority.key],
            1,
        )?,
        &[mint.clone(), destination.clone(), mint_authority.clone(), token_program.clone()],
        &[signer_seeds],
    )?;

    msg!("Creating metadata account");
    let create_metadata_ix = create_metadata_account_v3(
        metadata.key,
        mint.key,
        mint_authority.key,
        user.key,
        mint_authority.key,
        system_program.key,
        "DummyCollection",
        "DC",
        "",
        None,
        true,
    );
    invoke_signed(
        &create_metadata_ix,
        &[
            metadata.clone(),
            mint.clone(),
            mint_authority.clone(),
            user.clone(),
            mint_authority.clone(),
            system_program.clone(),
            token_metadata_program.clone(),
        ],
        &[signer_seeds],
    )?;

    msg!("Creating master edition account");
    let create_edition_ix = create_master_edition_v3(
        master_edition.key,
        mint.key,
        mint_authority.key,
        mint_authority.key,
        user.key,
        metadata.key,
        token_program.key,
        system_program.key,
    );
    invoke_signed(
        &create_edition_ix,
        &[
            master_edition.clone(),
            mint.clone(),
            mint_authority.clone(),
            mint_authority.clone(),
            user.clone(),
            metadata.clone(),
            token_program.clone(),
            system_program.clone(),
            token_metadata_program.clone(),
        ],
        &[signer_seeds],
    )?;

    msg!("Collection NFT created successfully");
    Ok(())
}
