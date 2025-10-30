//! Storage provider trait and implementations
//!
//! This module defines the storage abstraction layer that allows TOS-VM
//! to be integrated with different storage backends without modification.

use tos_tbpf::error::EbpfError;

/// Storage provider trait
///
/// This trait must be implemented by the TOS blockchain to provide
/// storage access to contracts. The VM itself does not implement storage;
/// it only defines the interface.
///
/// # Example Implementation
///
/// ```rust,ignore
/// use tos_program_runtime::storage::StorageProvider;
///
/// struct TosChainStorage {
///     // Your storage implementation
/// }
///
/// impl StorageProvider for TosChainStorage {
///     fn get(&self, contract_hash: &[u8; 32], key: &[u8]) -> Result<Option<Vec<u8>>, EbpfError> {
///         // Read from your storage backend
///         todo!()
///     }
///
///     fn set(&mut self, contract_hash: &[u8; 32], key: &[u8], value: &[u8]) -> Result<(), EbpfError> {
///         // Write to your storage backend
///         todo!()
///     }
///
///     fn delete(&mut self, contract_hash: &[u8; 32], key: &[u8]) -> Result<bool, EbpfError> {
///         // Delete from your storage backend
///         todo!()
///     }
/// }
/// ```
pub trait StorageProvider {
    /// Read a value from contract storage
    ///
    /// # Arguments
    /// * `contract_hash` - Hash of the contract (for isolation)
    /// * `key` - Storage key
    ///
    /// # Returns
    /// * `Some(value)` if the key exists
    /// * `None` if the key doesn't exist
    fn get(&self, contract_hash: &[u8; 32], key: &[u8]) -> Result<Option<Vec<u8>>, EbpfError>;

    /// Write a value to contract storage
    ///
    /// # Arguments
    /// * `contract_hash` - Hash of the contract (for isolation)
    /// * `key` - Storage key
    /// * `value` - Value to store
    fn set(&mut self, contract_hash: &[u8; 32], key: &[u8], value: &[u8]) -> Result<(), EbpfError>;

    /// Delete a key from contract storage
    ///
    /// # Arguments
    /// * `contract_hash` - Hash of the contract (for isolation)
    /// * `key` - Storage key
    ///
    /// # Returns
    /// * `true` if the key existed and was deleted
    /// * `false` if the key didn't exist
    fn delete(&mut self, contract_hash: &[u8; 32], key: &[u8]) -> Result<bool, EbpfError>;
}

/// Account provider trait
///
/// This trait must be implemented by the TOS blockchain to provide
/// balance queries and transfers.
pub trait AccountProvider {
    /// Get the balance of an account
    ///
    /// # Arguments
    /// * `address` - Account address
    ///
    /// # Returns
    /// The account balance in smallest units
    fn get_balance(&self, address: &[u8; 32]) -> Result<u64, EbpfError>;

    /// Transfer tokens from one account to another
    ///
    /// # Arguments
    /// * `from` - Source account (contract)
    /// * `to` - Destination account
    /// * `amount` - Amount to transfer
    ///
    /// # Returns
    /// * `Ok(())` if transfer succeeded
    /// * `Err(...)` if insufficient balance or other error
    fn transfer(&mut self, from: &[u8; 32], to: &[u8; 32], amount: u64) -> Result<(), EbpfError>;
}

/// No-op storage provider for testing
///
/// This implementation does nothing and is useful for:
/// - Unit tests that don't need storage
/// - Benchmarking VM performance
/// - Development/debugging
pub struct NoOpStorage;

impl StorageProvider for NoOpStorage {
    fn get(&self, _contract_hash: &[u8; 32], _key: &[u8]) -> Result<Option<Vec<u8>>, EbpfError> {
        Ok(None)
    }

    fn set(&mut self, _contract_hash: &[u8; 32], _key: &[u8], _value: &[u8]) -> Result<(), EbpfError> {
        Ok(())
    }

    fn delete(&mut self, _contract_hash: &[u8; 32], _key: &[u8]) -> Result<bool, EbpfError> {
        Ok(false)
    }
}

/// No-op account provider for testing
pub struct NoOpAccounts;

impl AccountProvider for NoOpAccounts {
    fn get_balance(&self, _address: &[u8; 32]) -> Result<u64, EbpfError> {
        Ok(0)
    }

    fn transfer(&mut self, _from: &[u8; 32], _to: &[u8; 32], _amount: u64) -> Result<(), EbpfError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_storage() {
        let mut storage = NoOpStorage;
        let contract = [1u8; 32];
        let key = b"test_key";
        let value = b"test_value";

        // Read non-existent key
        assert_eq!(storage.get(&contract, key).unwrap(), None);

        // Write
        assert!(storage.set(&contract, key, value).is_ok());

        // Delete
        assert_eq!(storage.delete(&contract, key).unwrap(), false);
    }

    #[test]
    fn test_noop_accounts() {
        let mut accounts = NoOpAccounts;
        let addr1 = [1u8; 32];
        let addr2 = [2u8; 32];

        // Balance is always 0
        assert_eq!(accounts.get_balance(&addr1).unwrap(), 0);

        // Transfer always succeeds
        assert!(accounts.transfer(&addr1, &addr2, 1000).is_ok());
    }
}
