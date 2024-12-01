#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[ink::contract]
pub mod document_acounts {
    use ink::{
        primitives::Key,
        storage::{traits::ManualKey, Lazy, Mapping, StorageVec},
    };

    use docs_support::{Builder, HashBuilder};

    // using random hex generator :)
    const VERSION_KEY: Key = 0x9e245fad;
    const IDENTITIES_KEY: Key = 0x53cd0567;
    const ROLE_KEY: Key = 0xa5c4ec7f;
    const DOMAIN_KEY: Key = 0x2acfa878;
    const METADATA_KEY: Key = 0x12896c2e;

    type AccountResult<T> = Result<T, AccountError>;
    #[ink(storage)]
    #[derive(Default)]
    pub struct DocumentAcounts {
        version: Lazy<u32, ManualKey<VERSION_KEY>>,
        pub identities: Mapping<AccountId, Hash, ManualKey<IDENTITIES_KEY>>,
        pub role: Mapping<AccountId, Role, ManualKey<ROLE_KEY>>,
        domain: Mapping<AccountId, Domain, ManualKey<DOMAIN_KEY>>,
        metadata: Mapping<AccountId, Hash, ManualKey<METADATA_KEY>>,
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
        #[must_use]
        pub fn new() -> Self {
            Self::default()
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
                    old_version,
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
            if let Some(caller_role) = self.role.get(caller) {
                match caller_role {
                    Role::Admin => {
                        self.role.insert(to, &role);
                        self.env().emit_event(Event {
                            from: Some(caller),
                            to: Some(to),
                            event_type: EventType::RoleGranted { role },
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
            if let Some(caller_role) = self.role.get(caller) {
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
            if let Some(role) = self.role.get(account_id) {
                ink::env::debug_println!("Role of account input is '{:?}'", role);
                Ok(role)
            } else {
                ink::env::debug_println!("No Role exist for this account");
                Err(AccountError::NoDataFound)
            }
        }

        fn account_has_uuid(&self, account_id: &AccountId) -> bool {
            match self.identities.get(account_id) {
                Some(_) => {
                    ink::env::debug_print!(
                        "Accound Error:UUID for this account already exist {:?}",
                        AccountError::AccountAlreadyHaveUUID
                    );
                    true
                }
                None => false,
            }
        }
        fn generate_uuid(&self, input: &[u8]) -> Hash {
            let uuid_builder = HashBuilder::default();
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
            if let Some(role) = self.role.get(caller) {
                ink::env::debug_println!("caller '{:?}' has role '{:?}'", caller, role);
                Some(role)
            } else {
                ink::env::debug_println!("caller '{:?}' has No role", caller);
                None
            }
        }
    }
}
