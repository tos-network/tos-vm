//! Test runner for u128-test contract

use std::sync::Arc;
use tos_program_runtime::InvokeContext;
use tos_tbpf::{
    aligned_memory::AlignedMemory,
    ebpf,
    elf::Executable,
    memory_region::{MemoryMapping, MemoryRegion},
    program::BuiltinProgram,
    vm::{Config, ContextObject, EbpfVm},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize simple logger with custom format
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "   {}", record.args())
        })
        .init();

    println!("=== TOS VM - U128 Test Runner ===\n");

    // 1. Create loader with syscalls
    println!("1. Creating TBPF loader and registering syscalls...");
    let config = Config::default();
    let mut loader = BuiltinProgram::<InvokeContext>::new_loader(config.clone());

    // Register syscalls
    tos_syscalls::register_syscalls(&mut loader)?;
    let loader = Arc::new(loader);
    println!("   ✓ Loader created with syscalls registered\n");

    // 2. Load u128-test executable
    println!("2. Loading u128-test.so...");
    let elf_path = "examples/u128-test/target/tbpf-tos-tos/release/u128_test.so";
    let elf_bytes = std::fs::read(elf_path)
        .map_err(|e| format!("Failed to read {}: {}", elf_path, e))?;

    let executable = Executable::load(&elf_bytes, loader.clone())?;
    println!("   ✓ Executable loaded ({} bytes)\n", elf_bytes.len());

    // 3. Create invoke context with NoOp providers
    println!("3. Creating execution context...");
    let mut storage = tos_program_runtime::NoOpStorage;
    let mut accounts = tos_program_runtime::NoOpAccounts;

    let mut invoke_context = InvokeContext::new_with_state(
        500_000,     // compute budget (more for u128 operations)
        [4u8; 32],   // contract hash
        [1u8; 32],   // block hash
        12345,       // block height
        [2u8; 32],   // tx hash
        [3u8; 32],   // tx sender
        &mut storage,
        &mut accounts,
    );
    invoke_context.enable_debug();
    println!("   ✓ Context created with 500,000 compute units\n");

    // 4. Create memory mapping
    let mut stack = AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(config.stack_size());
    let stack_len = stack.len();
    let regions: Vec<MemoryRegion> = vec![
        executable.get_ro_region(),
        MemoryRegion::new_writable(stack.as_slice_mut(), ebpf::MM_STACK_START),
    ];
    let memory_mapping = MemoryMapping::new(regions, &config, executable.get_tbpf_version())?;

    // 5. Create VM and execute
    println!("4. Executing u128 test contract...");
    println!("   --- Contract Output ---\n");

    let mut vm = EbpfVm::new(
        executable.get_loader().clone(),
        executable.get_tbpf_version(),
        &mut invoke_context,
        memory_mapping,
        stack_len,
    );

    let (instruction_count, result) = vm.execute_program(&executable, true);

    println!("\n   --- End of Output ---\n");

    // 6. Display results
    println!("5. Execution Results:");
    println!("   Instructions executed: {}", instruction_count);
    println!("   Return value: {:?}", result);
    println!("   Compute units used: {}", 500_000 - invoke_context.get_remaining());

    match result {
        tos_tbpf::error::ProgramResult::Ok(0) => {
            println!("\n✅ U128 tests executed successfully! Return value: SUCCESS (0)")
        }
        tos_tbpf::error::ProgramResult::Ok(val) => {
            println!("\n⚠️  Contract returned: {}", val)
        }
        tos_tbpf::error::ProgramResult::Err(e) => {
            println!("\n❌ Contract failed: {:?}", e)
        }
    }

    Ok(())
}
