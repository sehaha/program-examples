use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

/// The Metaplex Token Metadata program
/// (`metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`).
pub const TOKEN_METADATA_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    11, 112, 101, 177, 227, 209, 124, 69, 56, 157, 82, 127, 107, 4, 195, 205, 88, 184, 108, 115, 26, 160, 253, 181, 73,
    182, 209, 188, 3, 248, 41, 70,
]);

/// Discriminator of the Metaplex `CreateMetadataAccountV3` instruction.
const CREATE_METADATA_ACCOUNT_V3: u8 = 33;
/// Discriminator of the Metaplex `CreateMasterEditionV3` instruction.
const CREATE_MASTER_EDITION_V3: u8 = 17;
/// Discriminator of the Metaplex `Verify` instruction.
const VERIFY: u8 = 52;
/// `VerificationArgs::CollectionV1` variant index, the argument to `Verify`.
const VERIFY_COLLECTION_V1: u8 = 1;

/// Appends a Borsh `string` (4-byte little-endian length prefix + UTF-8 bytes).
fn push_borsh_string(buffer: &mut Vec<u8>, value: &[u8]) {
    buffer.extend_from_slice(&(value.len() as u32).to_le_bytes());
    buffer.extend_from_slice(value);
}

/// Serializes the data for a Metaplex `CreateMetadataAccountV3` instruction.
///
/// Layout: `[33] DataV2 is_mutable:bool collection_details:Option`, where
/// `DataV2` is `name:string symbol:string uri:string seller_fee:u16
/// creators:Option collection:Option uses:Option`.
///
/// The single creator is the mint-authority PDA, marked `verified` (it signs the
/// CPI). `collection` links a member NFT to its collection; `collection_details`
/// marks the metadata itself as a (sized) collection. The two are mutually
/// exclusive in this example, matching the Anchor sources.
pub fn build_metadata_data(
    name: &str,
    symbol: &str,
    uri: &str,
    creator: &Pubkey,
    collection: Option<&Pubkey>,
    is_collection: bool,
) -> Vec<u8> {
    let mut data = Vec::new();
    data.push(CREATE_METADATA_ACCOUNT_V3);

    // DataV2
    push_borsh_string(&mut data, name.as_bytes());
    push_borsh_string(&mut data, symbol.as_bytes());
    push_borsh_string(&mut data, uri.as_bytes());
    data.extend_from_slice(&0u16.to_le_bytes()); // seller_fee_basis_points

    // creators: Some(vec![Creator { address, verified: true, share: 100 }])
    data.push(1); // Option: Some
    data.extend_from_slice(&1u32.to_le_bytes()); // vec length
    data.extend_from_slice(creator.as_ref()); // address
    data.push(1); // verified: true
    data.push(100); // share

    // collection: Option<Collection { verified: bool, key: Pubkey }>
    match collection {
        Some(key) => {
            data.push(1); // Option: Some
            data.push(0); // verified: false (set true by verify_collection)
            data.extend_from_slice(key.as_ref());
        }
        None => data.push(0), // Option: None
    }

    data.push(0); // uses: None

    data.push(1); // is_mutable: true

    // collection_details: Option<CollectionDetails::V1 { size: u64 }>
    if is_collection {
        data.push(1); // Option: Some
        data.push(0); // CollectionDetails::V1
        data.extend_from_slice(&0u64.to_le_bytes()); // size
    } else {
        data.push(0); // Option: None
    }

    data
}

/// Serializes the data for a Metaplex `CreateMasterEditionV3` instruction.
///
/// Layout: `[17] max_supply:Option<u64>`. A `max_supply` of `Some(0)` allows no
/// printed editions (a one-of-one), matching the Anchor sources.
pub fn build_master_edition_data() -> Vec<u8> {
    let mut data = Vec::new();
    data.push(CREATE_MASTER_EDITION_V3);
    data.push(1); // max_supply: Some
    data.extend_from_slice(&0u64.to_le_bytes()); // max_supply value
    data
}

/// Serializes the data for a Metaplex `Verify` instruction with
/// `VerificationArgs::CollectionV1`. Layout: `[52, 1]`.
pub fn build_verify_collection_data() -> Vec<u8> {
    vec![VERIFY, VERIFY_COLLECTION_V1]
}

/// Creates a `CreateMetadataAccountV3` instruction.
#[allow(clippy::too_many_arguments)]
pub fn create_metadata_account_v3(
    metadata: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    update_authority: &Pubkey,
    system_program: &Pubkey,
    name: &str,
    symbol: &str,
    uri: &str,
    collection: Option<&Pubkey>,
    is_collection: bool,
) -> Instruction {
    let data = build_metadata_data(name, symbol, uri, mint_authority, collection, is_collection);
    Instruction {
        program_id: TOKEN_METADATA_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*metadata, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*mint_authority, true),
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*update_authority, false),
            AccountMeta::new_readonly(*system_program, false),
        ],
        data,
    }
}

/// Creates a `CreateMasterEditionV3` instruction.
#[allow(clippy::too_many_arguments)]
pub fn create_master_edition_v3(
    edition: &Pubkey,
    mint: &Pubkey,
    update_authority: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    metadata: &Pubkey,
    token_program: &Pubkey,
    system_program: &Pubkey,
) -> Instruction {
    let data = build_master_edition_data();
    Instruction {
        program_id: TOKEN_METADATA_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*edition, false),
            AccountMeta::new(*mint, false),
            AccountMeta::new_readonly(*update_authority, true),
            AccountMeta::new_readonly(*mint_authority, true),
            AccountMeta::new(*payer, true),
            AccountMeta::new(*metadata, false),
            AccountMeta::new_readonly(*token_program, false),
            AccountMeta::new_readonly(*system_program, false),
        ],
        data,
    }
}

/// Creates a `Verify` instruction with `VerificationArgs::CollectionV1`.
#[allow(clippy::too_many_arguments)]
pub fn verify_collection_instruction(
    mint_authority: &Pubkey,
    token_metadata_program: &Pubkey,
    metadata: &Pubkey,
    collection_mint: &Pubkey,
    collection_metadata: &Pubkey,
    collection_master_edition: &Pubkey,
    system_program: &Pubkey,
    sysvar_instructions: &Pubkey,
) -> Instruction {
    let data = build_verify_collection_data();
    Instruction {
        program_id: TOKEN_METADATA_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*mint_authority, true),
            AccountMeta::new_readonly(*token_metadata_program, false),
            AccountMeta::new(*metadata, false),
            AccountMeta::new_readonly(*collection_mint, false),
            AccountMeta::new(*collection_metadata, false),
            AccountMeta::new_readonly(*collection_master_edition, false),
            AccountMeta::new_readonly(*system_program, false),
            AccountMeta::new_readonly(*sysvar_instructions, false),
        ],
        data,
    }
}
