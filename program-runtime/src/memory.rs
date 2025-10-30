//! Memory translation utilities.
//!
//! This module provides utilities for safely translating VM memory addresses
//! to host memory references. Based on Solana's memory translation implementation.

use {
    tos_tbpf::memory_region::{AccessType, MemoryMapping},
    std::{mem::align_of, slice::from_raw_parts_mut},
};

/// Error types for memory translation operations.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MemoryTranslationError {
    /// Pointer is not properly aligned
    #[error("Unaligned pointer")]
    UnalignedPointer,
    /// Length is invalid or would cause overflow
    #[error("Invalid length")]
    InvalidLength,
}

/// Check if an address is properly aligned for type T
pub fn address_is_aligned<T>(address: u64) -> bool {
    (address as *mut T as usize)
        .checked_rem(align_of::<T>())
        .map(|rem| rem == 0)
        .expect("T to be non-zero aligned")
}

/// Internal macro for memory mapping - do not use directly
#[macro_export]
macro_rules! translate_inner {
    ($memory_mapping:expr, $map:ident, $access_type:expr, $vm_addr:expr, $len:expr $(,)?) => {
        Result::<u64, Box<dyn std::error::Error>>::from(
            $memory_mapping
                .$map($access_type, $vm_addr, $len)
                .map_err(|err| err.into()),
        )
    };
}

/// Internal macro for type translation - do not use directly
#[macro_export]
macro_rules! translate_type_inner {
    ($memory_mapping:expr, $access_type:expr, $vm_addr:expr, $T:ty, $check_aligned:expr $(,)?) => {{
        let host_addr = $crate::translate_inner!(
            $memory_mapping,
            map,
            $access_type,
            $vm_addr,
            size_of::<$T>() as u64
        )?;
        if !$check_aligned {
            Ok(unsafe { std::mem::transmute::<u64, &mut $T>(host_addr) })
        } else if !$crate::memory::address_is_aligned::<$T>(host_addr) {
            Err($crate::memory::MemoryTranslationError::UnalignedPointer.into())
        } else {
            Ok(unsafe { &mut *(host_addr as *mut $T) })
        }
    }};
}

/// Internal macro for slice translation - do not use directly
#[macro_export]
macro_rules! translate_slice_inner {
    ($memory_mapping:expr, $access_type:expr, $vm_addr:expr, $len:expr, $T:ty, $check_aligned:expr $(,)?) => {{
        if $len == 0 {
            return Ok(&mut []);
        }
        let total_size = $len.saturating_mul(size_of::<$T>() as u64);
        if isize::try_from(total_size).is_err() {
            return Err($crate::memory::MemoryTranslationError::InvalidLength.into());
        }
        let host_addr =
            $crate::translate_inner!($memory_mapping, map, $access_type, $vm_addr, total_size)?;
        if $check_aligned && !$crate::memory::address_is_aligned::<$T>(host_addr) {
            return Err($crate::memory::MemoryTranslationError::UnalignedPointer.into());
        }
        Ok(unsafe { from_raw_parts_mut(host_addr as *mut $T, $len as usize) })
    }};
}

/// Translate a VM address to a host reference of type T (immutable)
///
/// # Arguments
/// * `memory_mapping` - The memory mapping from the VM
/// * `vm_addr` - Virtual address in the VM's address space
/// * `check_aligned` - Whether to check pointer alignment
///
/// # Returns
/// An immutable reference to the data at the specified address
///
/// # Errors
/// Returns an error if:
/// - The address is not mapped
/// - The address is not properly aligned (if `check_aligned` is true)
pub fn translate_type<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    check_aligned: bool,
) -> Result<&'a T, Box<dyn std::error::Error>> {
    translate_type_inner!(memory_mapping, AccessType::Load, vm_addr, T, check_aligned)
        .map(|value| &*value)
}

/// Translate a VM address to a host slice (immutable)
///
/// # Arguments
/// * `memory_mapping` - The memory mapping from the VM
/// * `vm_addr` - Virtual address of the start of the slice
/// * `len` - Number of elements in the slice
/// * `check_aligned` - Whether to check pointer alignment
///
/// # Returns
/// An immutable slice reference to the data
///
/// # Errors
/// Returns an error if:
/// - The address is not mapped
/// - The address is not properly aligned (if `check_aligned` is true)
/// - The length would cause overflow
pub fn translate_slice<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
    check_aligned: bool,
) -> Result<&'a [T], Box<dyn std::error::Error>> {
    translate_slice_inner!(
        memory_mapping,
        AccessType::Load,
        vm_addr,
        len,
        T,
        check_aligned,
    )
    .map(|value| &*value)
}

/// Translate a VM address to a mutable host reference of type T
///
/// This version is for mutable access (Store operations).
///
/// # Arguments
/// * `memory_mapping` - The memory mapping from the VM
/// * `vm_addr` - Virtual address in the VM's address space
/// * `check_aligned` - Whether to check pointer alignment
///
/// # Returns
/// A mutable reference to the data at the specified address
///
/// # Errors
/// Returns an error if:
/// - The address is not mapped
/// - The address is not properly aligned (if `check_aligned` is true)
pub fn translate_type_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    check_aligned: bool,
) -> Result<&'a mut T, Box<dyn std::error::Error>> {
    translate_type_inner!(memory_mapping, AccessType::Store, vm_addr, T, check_aligned)
}

/// Translate a VM address to a mutable host slice
///
/// This version is for mutable access (Store operations).
///
/// # Arguments
/// * `memory_mapping` - The memory mapping from the VM
/// * `vm_addr` - Virtual address of the start of the slice
/// * `len` - Number of elements in the slice
/// * `check_aligned` - Whether to check pointer alignment
///
/// # Returns
/// A mutable slice reference to the data
///
/// # Errors
/// Returns an error if:
/// - The address is not mapped
/// - The address is not properly aligned (if `check_aligned` is true)
/// - The length would cause overflow
pub fn translate_slice_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
    check_aligned: bool,
) -> Result<&'a mut [T], Box<dyn std::error::Error>> {
    translate_slice_inner!(
        memory_mapping,
        AccessType::Store,
        vm_addr,
        len,
        T,
        check_aligned,
    )
}

/// Translate a VM address to a string slice
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
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let bytes = translate_slice::<u8>(memory_mapping, vm_addr, len, false)?;
    std::str::from_utf8(bytes)
        .map_err(|_| MemoryTranslationError::InvalidLength.into())
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

        let result: Result<&u32, _> = translate_type(&mapping, 0x100000000, true);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(*value, 0x78563412); // Little-endian
    }

    #[test]
    fn test_translate_slice() {
        let mut data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let mapping = create_test_mapping(&mut data);

        let result = translate_slice::<u8>(&mapping, 0x100000000, 4, false);
        assert!(result.is_ok());
        let slice = result.unwrap();
        assert_eq!(slice, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_translate_empty_slice() {
        let mut data = vec![1u8, 2, 3, 4];
        let mapping = create_test_mapping(&mut data);

        let result = translate_slice::<u8>(&mapping, 0x100000000, 0, false);
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

    #[test]
    fn test_address_alignment() {
        assert!(address_is_aligned::<u32>(0x100000000));
        assert!(address_is_aligned::<u32>(0x100000004));
        assert!(!address_is_aligned::<u32>(0x100000001));
        assert!(!address_is_aligned::<u32>(0x100000002));
    }

    #[test]
    fn test_translate_type_mut() {
        let mut data = vec![0u32, 0u32];
        let mapping = create_test_mapping(unsafe {
            std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u8, data.len() * 4)
        });

        let result: Result<&mut u32, _> = translate_type_mut(&mapping, 0x100000000, true);
        assert!(result.is_ok());
        let value = result.unwrap();
        *value = 0x12345678;
        assert_eq!(data[0], 0x12345678);
    }

    #[test]
    fn test_translate_slice_mut() {
        let mut data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let mapping = create_test_mapping(&mut data);

        let result = translate_slice_mut::<u8>(&mapping, 0x100000000, 4, false);
        assert!(result.is_ok());
        let slice = result.unwrap();
        slice[0] = 99;
        slice[1] = 88;
        assert_eq!(data[0], 99);
        assert_eq!(data[1], 88);
    }

    #[test]
    fn test_unaligned_access_with_check() {
        let mut data = vec![0u8; 16];
        let mapping = create_test_mapping(&mut data);

        // Unaligned u32 access should fail with check_aligned=true
        let result: Result<&u32, _> = translate_type(&mapping, 0x100000001, true);
        assert!(result.is_err());

        // But should succeed with check_aligned=false
        let result: Result<&u32, _> = translate_type(&mapping, 0x100000001, false);
        assert!(result.is_ok());
    }
}
