use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instructions::AUTHORITY_SEED, mpl_util::verify_collection_instruction};

/// Verifies an NFT as a member of its collection via the Metaplex `Verify`
/// instruction (`VerificationArgs::CollectionV1`), signed by the collection's
/// update authority — the program's `[b"authority"]` PDA.
pub fn verify_collection(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;
    let metadata = next_account_info(accounts_iter)?;
    let collection_mint = next_account_info(accounts_iter)?;
    let collection_metadata = next_account_info(accounts_iter)?;
    let collection_master_edition = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let sysvar_instructions = next_account_info(accounts_iter)?;
    let token_metadata_program = next_account_info(accounts_iter)?;

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda, bump) = Pubkey::find_program_address(&[AUTHORITY_SEED], program_id);
    if mint_authority.key != &pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let bump_bytes = [bump];
    let signer_seeds: &[&[u8]] = &[AUTHORITY_SEED, &bump_bytes];

    msg!("Verifying collection");
    let ix = verify_collection_instruction(
        mint_authority.key,
        token_metadata_program.key,
        metadata.key,
        collection_mint.key,
        collection_metadata.key,
        collection_master_edition.key,
        system_program.key,
        sysvar_instructions.key,
    );

    invoke_signed(
        &ix,
        &[
            mint_authority.clone(),
            token_metadata_program.clone(),
            metadata.clone(),
            collection_mint.clone(),
            collection_metadata.clone(),
            collection_master_edition.clone(),
            system_program.clone(),
            sysvar_instructions.clone(),
        ],
        &[signer_seeds],
    )?;

    msg!("Collection verified successfully");
    Ok(())
}
