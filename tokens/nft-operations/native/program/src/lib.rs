pub mod instructions;
pub mod mpl_util;
pub mod processor;

use {processor::process_instruction, solana_program::entrypoint};

entrypoint!(process_instruction);
