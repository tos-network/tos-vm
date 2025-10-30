//! Hello World Smart Contract for TOS VM
//!
//! This is a minimal example contract that demonstrates:
//! - Logging messages
//! - Accessing blockchain state
//! - Reading transaction information
//!
//! Entry point 0: Logs "Hello, TOS!" and blockchain information

#![no_std]
#![no_main]

use tos_vm_sdk::*;

/// Entry point 0: Main contract function
///
/// This function demonstrates basic syscalls:
/// - log(): Output debug messages
/// - get_block_height(): Read blockchain state
/// - get_tx_sender(): Read transaction information
/// - get_contract_hash(): Get contract's own address
#[no_mangle]
pub extern "C" fn entry_0() -> u64 {
    // Log hello message
    log("Hello, TOS!");

    // Get and log block height
    let block_height = get_block_height();
    log("Contract executing at block height...");

    // Get transaction sender
    let sender = get_tx_sender();
    log("Transaction sent by user");

    // Get contract's own address
    let contract = get_contract_hash();
    log("Contract executing with address...");

    // Log completion
    log("Hello World contract completed successfully!");

    // Return success
    SUCCESS
}

/// Panic handler (required for no_std)
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
