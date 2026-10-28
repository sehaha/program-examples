pub mod create_collection;
pub mod mint_nft;
pub mod verify_collection;

pub use create_collection::*;
pub use mint_nft::*;
pub use verify_collection::*;

/// Seed prefix for the program's mint-authority PDA (`[b"authority"]`). The PDA
/// is never initialized — it exists only to sign the Metaplex CPIs.
pub const AUTHORITY_SEED: &[u8] = b"authority";
