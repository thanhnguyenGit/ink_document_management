#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod document_acounts {
    use ink::{
        env::{
            hash::{Blake2x256, CryptoHash},
            ContractEnv,
        },
        prelude::vec,
        primitives,
        storage::{traits::ManualKey, Mapping, StorageVec},
    };

    type AccountResult<T> = Result<T, AccError>;
    #[ink(storage)]
    pub struct DocumentAcounts {
        version: u32, // should be wrapped by Lazy<>
        identities: Mapping<AccountId, u32, ManualKey<0x30>>,
        bool: bool,
        permission: Mapping<AccountId, Permission, ManualKey<0x24>>,
        role: Mapping<AccountId, Role, ManualKey<0x25>>,
        domain: Mapping<AccountId, Domain, ManualKey<0x26>>,
        metadata: Mapping<AccountId, Hash, ManualKey<0x27>>,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Permission {
        Full,
        CanRead,
        CanWrite,
        CanPublish,
        CanDelete,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Role {
        Owner,
        Admin,
        Publisher,
        Speculator,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum AccError {
        DupilcatedUUID,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Domain {
        domain_id: Hash,
        domain_description: Hash,
    }

    impl DocumentAcounts {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                version: 1,
                identities: Mapping::default(),
                permission: Mapping::default(),
                bool: true,
                metadata: Mapping::default(),
                domain: Mapping::default(),
                role: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn document_account_new(&mut self) -> AccountResult<()> {
            let caller = self.env().caller();
            let id = vec![0x69, 0xB8, 0x90];
            Ok(())
        }
        #[ink(message)]
        pub fn document_account_version(&mut self) -> AccountResult<()> {
            let caller = self.env().caller();
            self.version = self.version.checked_add(1).unwrap();
            Ok(())
        }
        //
        // fn generate_hash(&self, input: impl AsRef<[u8]>) -> Hash {
        //     let mut res = Vec::new();
        //     res.extend_from_slice(input.as_ref());
        //     self.env().hash_bytes::<Blake2x256>(&res).into()
        // }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    // #[cfg(test)]
    // mod tests {
    //     /// Imports all the definitions from the outer scope so we can use them here.
    //     use super::*;
    //
    //     /// We test if the default constructor does its job.
    //     #[ink::test]
    //     fn default_works() {
    //         let document_acounts = DocumentAcounts::default();
    //         assert_eq!(document_acounts.get(), false);
    //     }
    //
    //     /// We test a simple use case of our contract.
    //     #[ink::test]
    //     fn it_works() {
    //         let mut document_acounts = DocumentAcounts::new(false);
    //         assert_eq!(document_acounts.get(), false);
    //         document_acounts.flip();
    //         assert_eq!(document_acounts.get(), true);
    //     }
    // }
    //
    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = DocumentAcountsRef::default();

            // When
            let contract = client
                .instantiate("document_acounts", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<DocumentAcounts>();

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = DocumentAcountsRef::new(false);
            let contract = client
                .instantiate("document_acounts", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<DocumentAcounts>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = call_builder.flip();
            let _flip_result = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
