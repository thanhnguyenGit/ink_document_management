#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod document_acounts {
    use ink::{
        env::{
            hash::{Blake2x256, CryptoHash},
            ContractEnv,
        },
        prelude::vec,
        primitives::{self, Key},
        storage::{traits::ManualKey, Lazy, Mapping, StorageVec},
    };
    use traits::{Builder, HashBuilder};

    // using random hex generator :)
    const VERSION_KEY: Key = 0x9e245fad;
    const IDENTITIES_KEY: Key = 0x53cd0567;
    const PERMISSON_KEY: Key = 0x71e6551d;
    const ROLE_KEY: Key = 0xa5c4ec7f;
    const DOMAIN_KEY: Key = 0x2acfa878;
    const METADATA_KEY: Key = 0x12896c2e;

    type AccountResult<T> = Result<T, AccountError>;
    #[ink(storage)]
    pub struct DocumentAcounts {
        version: Lazy<u32, ManualKey<VERSION_KEY>>,
        identities: Mapping<AccountId, Hash, ManualKey<IDENTITIES_KEY>>,
        permission: Mapping<Role, Vec<Permission>, ManualKey<PERMISSON_KEY>>,
        role: Mapping<AccountId, Role, ManualKey<ROLE_KEY>>,
        domain: Mapping<AccountId, Domain, ManualKey<DOMAIN_KEY>>,
        metadata: Mapping<AccountId, Hash, ManualKey<METADATA_KEY>>,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Permission {
        CanRead,
        CanWrite,
        CanDelete,
        CanBurn,
        CanMint,
        CanGrant,
        CanRevoke,
        CanTransfer,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Role {
        Admin,
        Speculator,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum AccountError {
        DupilcatedUUID,
        AccountAlreadyHaveUUID,
        Unauthorized,
        NoDataFound,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Domain {
        domain_id: Hash,
        domain_description: Hash,
    }

    #[ink(event)]
    pub struct Event {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        event_type: EventType,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum EventType {
        NewAccount,
        VersionUpdated { old_version: u32, new_version: u32 },
        RoleGranted { role: Role },
        RoleRevoke { previous_role: Role },
    }

    impl DocumentAcounts {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut _init_permissions = Mapping::default();
            _init_permissions.insert(
                Role::Admin,
                &vec![
                    Permission::CanMint,
                    Permission::CanBurn,
                    Permission::CanRevoke,
                    Permission::CanDelete,
                    Permission::CanRead,
                    Permission::CanWrite,
                ],
            );
            _init_permissions.insert(
                Role::Speculator,
                &vec![
                    Permission::CanRead,
                    Permission::CanWrite,
                    Permission::CanTransfer,
                ],
            );

            Self {
                version: Lazy::new(),
                identities: Mapping::default(),
                permission: _init_permissions,
                metadata: Mapping::default(),
                domain: Mapping::default(),
                role: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn account_new(&mut self) -> AccountResult<()> {
            let caller = self.env().caller();
            if self.account_has_uuid(&caller) {
                return Err(AccountError::AccountAlreadyHaveUUID);
            }

            let uuid = self.generate_uuid(caller.as_ref());
            self.identities.insert(caller, &uuid);
            self.env().emit_event(Event {
                from: Some(caller),
                to: None,
                event_type: EventType::NewAccount,
            });
            Ok(())
        }
        #[ink(message)]
        pub fn account_version_new(&mut self, init_version: u32) -> AccountResult<()> {
            let caller = self.env().caller();
            let old_version = self
                .version
                .get()
                .expect("Version had to be init first via constructor");
            if !self.caller_is_developer(&caller) {
                ink::env::debug_println!("Unathorized access: {:?}", AccountError::Unauthorized);
                return Err(AccountError::Unauthorized);
            }
            self.version.set(&init_version);
            self.env().emit_event(Event {
                from: Some(caller),
                to: None,
                event_type: EventType::VersionUpdated {
                    old_version: old_version,
                    new_version: init_version,
                },
            });
            Ok(())
        }
        #[ink(message)]
        pub fn grant_role(&mut self, to: AccountId, role: Role) -> AccountResult<()> {
            let caller = self.env().caller();
            if caller == to {
                ink::env::debug_println!(
                    "Input account '{:?}' is the same as caller account '{:?}'",
                    to,
                    caller
                );
                return Err(AccountError::Unauthorized);
            }
            if let Some(caller_role) = self.role.get(&caller) {
                match caller_role {
                    Role::Admin => {
                        self.role.insert(to, &role);
                        self.env().emit_event(Event {
                            from: Some(caller),
                            to: Some(to),
                            event_type: EventType::RoleGranted { role: role },
                        });
                    }
                    _ => return Err(AccountError::Unauthorized),
                }
            }
            Ok(())
        }
        #[ink(message)]
        pub fn revoke_role(&mut self, account_id: AccountId) -> AccountResult<()> {
            let caller = self.env().caller();
            if caller == account_id {
                ink::env::debug_println!(
                    "Cannot self revoke role, input account: '{:?}', caller account: '{:?}'",
                    account_id,
                    caller
                );
                return Err(AccountError::Unauthorized);
            }
            if let Some(caller_role) = self.role.get(&caller) {
                match caller_role {
                    Role::Admin => {
                        let previous_role = self
                            .role
                            .get(account_id)
                            .expect("Account must had Role to be removed");
                        self.role.remove(account_id);
                        self.env().emit_event(Event {
                            from: Some(caller),
                            to: Some(account_id),
                            event_type: EventType::RoleGranted {
                                role: previous_role,
                            },
                        });
                    }
                    _ => return Err(AccountError::Unauthorized),
                }
            }
            Ok(())
        }
        #[ink(message)]
        pub fn get_role(&self, account_id: AccountId) -> AccountResult<Role> {
            match self.role.get(account_id) {
                Some(role) => {
                    ink::env::debug_println!("Role of account input is '{:?}'", role);
                    return Ok(role);
                }
                None => {
                    ink::env::debug_println!("No Role exist for this account");
                    return Err(AccountError::NoDataFound);
                }
            }
        }

        fn account_has_uuid(&self, account_id: &AccountId) -> bool {
            match self.identities.get(account_id) {
                Some(_) => {
                    ink::env::debug_print!(
                        "Accound Error:UUID for this account already exist {:?}",
                        AccountError::AccountAlreadyHaveUUID
                    );
                    return true;
                }
                None => return false,
            }
        }
        fn generate_uuid(&self, input: &[u8]) -> Hash {
            let mut uuid_builder = HashBuilder::default();
            let block_height = &[self.env().block_number() as u8];
            let time_stamp = &[self.env().block_timestamp() as u8];
            uuid_builder
                .add_segment(input)
                .add_segment(block_height)
                .add_segment(time_stamp)
                .build()
        }
        fn caller_is_developer(&self, caller: &AccountId) -> bool {
            if self
                .role
                .get(caller)
                .expect("This caller must had a role initilized")
                != Role::Admin
            {
                return false;
            }
            true
        }
        fn get_caller_role(&self, caller: &AccountId) -> Option<Role> {
            match self.role.get(caller) {
                Some(role) => {
                    ink::env::debug_println!("caller '{:?}' has role '{:?}'", caller, role);
                    return Some(role);
                }
                None => {
                    ink::env::debug_println!("caller '{:?}' has No role", caller);
                    return None;
                }
            }
        }
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
