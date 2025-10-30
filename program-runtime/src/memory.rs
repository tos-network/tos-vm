//! Memory translation utilities for TBPF programs
//!
//! This module provides helper functions to safely translate VM memory addresses
//! to host memory references. These utilities are used by syscalls to access
//! data passed from contracts.
//!
//! Based on Solana's memory translation patterns in program-runtime/src/memory.rs

use tos_tbpf::{
    error::{EbpfError, ProgramResult},
    memory_region::{AccessType, MemoryMapping},
};

/// Translates a VM address to a host reference of type T
///
/// # Safety
/// This function performs bounds checking and alignment validation.
///
/// # Arguments
/// * `memory_mapping` - The memory mapping from the VM
/// * `vm_addr` - Virtual address in the VM's address space
///
/// # Returns
/// A reference to the data at the specified address
///
/// # Errors
/// Returns an error if:
/// - The address is not mapped
/// - The address is not properly aligned for type T
/// - The address + size would overflow the mapped region
pub fn translate_type<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
) -> Result<&'a T, EbpfError> {
    let size = std::mem::size_of::<T>() as u64;

    // Check alignment
    let align = std::mem::align_of::<T>() as u64;
    if vm_addr % align != 0 {
        return Err(EbpfError::InvalidMemoryRegion(vm_addr as usize));
    }

    // Translate the address
    let host_addr = match memory_mapping.map(AccessType::Load, vm_addr, size) {
        ProgramResult::Ok(addr) => addr,
        ProgramResult::Err(e) => return Err(e),
    };

    // Safety: We've verified alignment and bounds
    unsafe {
        Ok(&*(host_addr as *const T))
    }
}

/// Translates a VM address to a mutable host reference of type T
///
/// # Safety
/// This function performs bounds checking and alignment validation.
///
/// # Arguments
/// * `memory_mapping` - The memory mapping from the VM
/// * `vm_addr` - Virtual address in the VM's address space
///
/// # Returns
/// A mutable reference to the data at the specified address
///
/// # Errors
/// Returns an error if:
/// - The address is not mapped
/// - The address is not properly aligned for type T
/// - The address + size would overflow the mapped region
pub fn translate_type_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
) -> Result<&'a mut T, EbpfError> {
    let size = std::mem::size_of::<T>() as u64;

    // Check alignment
    let align = std::mem::align_of::<T>() as u64;
    if vm_addr % align != 0 {
        return Err(EbpfError::InvalidMemoryRegion(vm_addr as usize));
    }

    // Translate the address
    let host_addr = match memory_mapping.map(AccessType::Store, vm_addr, size) {
        ProgramResult::Ok(addr) => addr,
        ProgramResult::Err(e) => return Err(e),
    };

    // Safety: We've verified alignment and bounds
    unsafe {
        Ok(&mut *(host_addr as *mut T))
    }
}

/// Translates a VM address and length to a host slice
///
/// # Arguments
/// * `memory_mapping` - The memory mapping from the VM
/// * `vm_addr` - Virtual address of the start of the slice
/// * `len` - Number of elements in the slice
///
/// # Returns
/// A slice reference to the data
///
/// # Errors
/// Returns an error if:
/// - The address is not mapped
/// - The address is not properly aligned for type T
/// - The address + (len * size_of::<T>()) would overflow the mapped region
/// - len * size_of::<T>() would overflow u64
pub fn translate_slice<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
) -> Result<&'a [T], EbpfError> {
    if len == 0 {
        return Ok(&[]);
    }

    let size_of_t = std::mem::size_of::<T>() as u64;

    // Check for overflow in size calculation
    let total_size = len.checked_mul(size_of_t)
        .ok_or(EbpfError::InvalidMemoryRegion(vm_addr as usize))?;

    // Check alignment
    let align = std::mem::align_of::<T>() as u64;
    if vm_addr % align != 0 {
        return Err(EbpfError::InvalidMemoryRegion(vm_addr as usize));
    }

    // Translate the address
    let host_addr = match memory_mapping.map(AccessType::Load, vm_addr, total_size) {
        ProgramResult::Ok(addr) => addr,
        ProgramResult::Err(e) => return Err(e),
    };

    // Safety: We've verified alignment, bounds, and overflow
    unsafe {
        Ok(std::slice::from_raw_parts(host_addr as *const T, len as usize))
    }
}

/// Translates a VM address and length to a mutable host slice
///
/// # Arguments
/// * `memory_mapping` - The memory mapping from the VM
/// * `vm_addr` - Virtual address of the start of the slice
/// * `len` - Number of elements in the slice
///
/// # Returns
/// A mutable slice reference to the data
///
/// # Errors
/// Returns an error if:
/// - The address is not mapped
/// - The address is not properly aligned for type T
/// - The address + (len * size_of::<T>()) would overflow the mapped region
/// - len * size_of::<T>() would overflow u64
pub fn translate_slice_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
) -> Result<&'a mut [T], EbpfError> {
    if len == 0 {
        return Ok(&mut []);
    }

    let size_of_t = std::mem::size_of::<T>() as u64;

    // Check for overflow in size calculation
    let total_size = len.checked_mul(size_of_t)
        .ok_or(EbpfError::InvalidMemoryRegion(vm_addr as usize))?;

    // Check alignment
    let align = std::mem::align_of::<T>() as u64;
    if vm_addr % align != 0 {
        return Err(EbpfError::InvalidMemoryRegion(vm_addr as usize));
    }

    // Translate the address
    let host_addr = match memory_mapping.map(AccessType::Store, vm_addr, total_size) {
        ProgramResult::Ok(addr) => addr,
        ProgramResult::Err(e) => return Err(e),
    };

    // Safety: We've verified alignment, bounds, and overflow
    unsafe {
        Ok(std::slice::from_raw_parts_mut(host_addr as *mut T, len as usize))
    }
}

/// Translates a VM address to a string slice
///
/// This is a convenience function for translating byte slices and
/// validating them as UTF-8 strings.
///
/// # Arguments
/// * `memory_mapping` - The memory mapping from the VM
/// * `vm_addr` - Virtual address of the start of the string
/// * `len` - Length of the string in bytes
///
/// # Returns
/// A string slice reference to the data
///
/// # Errors
/// Returns an error if:
/// - Memory translation fails
/// - The bytes are not valid UTF-8
pub fn translate_string<'a>(
    memory_mapping: &'a MemoryMapping,
    vm_addr: u64,
    len: u64,
) -> Result<&'a str, EbpfError> {
    let bytes = translate_slice::<u8>(memory_mapping, vm_addr, len)?;
    std::str::from_utf8(bytes)
        .map_err(|_| EbpfError::InvalidMemoryRegion(vm_addr as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tos_tbpf::{
        memory_region::{MemoryRegion, MemoryMapping},
        program::TBPFVersion,
    };

    fn create_test_mapping(data: &mut [u8]) -> MemoryMapping {
        // Leak config so it lives for 'static - this is fine for tests
        let config: &'static tos_tbpf::vm::Config = Box::leak(Box::new(tos_tbpf::vm::Config::default()));
        let region = MemoryRegion::new_writable(data, 0x100000000);
        MemoryMapping::new(
            vec![region],
            config,
            TBPFVersion::V3,
        ).unwrap()
    }

    #[test]
    fn test_translate_type() {
        let mut data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mapping = create_test_mapping(&mut data);

        let result: Result<&u32, _> = translate_type(&mapping, 0x100000000);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(*value, 0x78563412); // Little-endian
    }

    #[test]
    fn test_translate_slice() {
        let mut data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let mapping = create_test_mapping(&mut data);

        let result = translate_slice::<u8>(&mapping, 0x100000000, 4);
        assert!(result.is_ok());
        let slice = result.unwrap();
        assert_eq!(slice, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_translate_empty_slice() {
        let mut data = vec![1u8, 2, 3, 4];
        let mapping = create_test_mapping(&mut data);

        let result = translate_slice::<u8>(&mapping, 0x100000000, 0);
        assert!(result.is_ok());
        let slice = result.unwrap();
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn test_translate_string() {
        let mut data = b"Hello, TOS!".to_vec();
        let len = data.len() as u64;
        let mapping = create_test_mapping(&mut data);

        let result = translate_string(&mapping, 0x100000000, len);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, TOS!");
    }

    #[test]
    fn test_translate_invalid_utf8() {
        let mut data = vec![0xFF, 0xFE, 0xFD];
        let mapping = create_test_mapping(&mut data);

        let result = translate_string(&mapping, 0x100000000, 3);
        assert!(result.is_err());
    }
}
