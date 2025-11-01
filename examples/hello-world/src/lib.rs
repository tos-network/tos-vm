//! Hello World Smart Contract for TOS VM
//!
//! This is a minimal example contract that demonstrates logging.

#![no_std]
#![no_main]

use tos_vm_sdk::*;

/// Main contract entrypoint
#[no_mangle]
pub extern "C" fn entrypoint() -> u64 {
    // Log hello message
    log("Hello, TOS!");

    // Return success
    SUCCESS
}

/// Panic handler (required for no_std)
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
