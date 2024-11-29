#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod dns {
    use ink::storage::Mapping;
    use scale::{Decode, Encode};

    type DnsResult<T> = Result<T, DnsError>;
    #[ink(storage)]
    pub struct Dns {
        name_to_address: Mapping<Hash, AccountId>,
        name_to_owner: Mapping<Hash, AccountId>,
        default_address: AccountId,
    }

    #[derive(Debug, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum DnsError {
        NameAlreadyExist,
        NotOwner,
        NonExist,
        NoDataFound,
    }
    #[ink(event)]
    pub struct Register {
        #[ink(topic)]
        name: Hash,
        #[ink(topic)]
        from: AccountId,
    }
    #[ink(event)]
    pub struct SetAddress {
        #[ink(topic)]
        name: Hash,
        from: AccountId,
        #[ink(topic)]
        old_addr: Option<AccountId>,
        #[ink(topic)]
        new_address: AccountId,
    }
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        name: Hash,
        from: AccountId,
        #[ink(topic)]
        old_addr: Option<AccountId>,
        #[ink(topic)]
        new_address: AccountId,
    }
    impl Dns {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                name_to_address: Mapping::default(),
                name_to_owner: Mapping::default(),
                default_address: AccountId::from([0x00; 32]),
            }
        }
        #[ink(message)]
        pub fn register(&mut self, name: Hash) -> DnsResult<()> {
            let caller = self.env().caller();
            match self.name_to_owner.contains(name) {
                true => Err(DnsError::NameAlreadyExist),
                false => {
                    self.name_to_owner.insert(name, &caller);
                    self.env().emit_event(Register { from: caller, name });
                    Ok(())
                }
            }
        }
        #[ink(message)]
        pub fn set_address(&mut self, name: Hash, new_addr: AccountId) -> DnsResult<()> {
            let caller = self.env().caller();
            match self.check_is_owner(&name, &caller) {
                true => {
                    let old_addr = self.name_to_address.get(&name);
                    self.name_to_address.insert(name, &new_addr);
                    self.env().emit_event(SetAddress {
                        name,
                        from: caller,
                        old_addr,
                        new_address: new_addr,
                    });
                    Ok(())
                }
                false => Err(DnsError::NotOwner),
            }
        }
        #[ink(message)]
        pub fn transfer(&mut self, name: Hash, to: AccountId) -> DnsResult<()> {
            let caller = self.env().caller();
            match self.check_is_owner(&name, &caller) {
                true => {
                    let old_owner = self.name_to_owner.get(name);
                    self.name_to_owner.insert(name, &to);
                    self.env().emit_event(Transfer {
                        name,
                        from: to,
                        old_addr: old_owner,
                        new_address: to,
                    });
                    Ok(())
                }
                false => Err(DnsError::NotOwner),
            }
        }
        #[ink(message)]
        pub fn get_address(&self, name: Hash) -> AccountId {
            self.name_to_address
                .get(name)
                .unwrap_or(self.default_address)
        }
        #[ink(message)]
        pub fn get_owner(&self, name: Hash) -> AccountId {
            self.name_to_owner.get(name).unwrap_or(self.default_address)
        }
        fn check_is_owner(&self, name: &Hash, addr: &AccountId) -> bool {
            match self.name_to_owner.get(name) {
                Some(val) => val == *addr,
                None => {
                    ink::env::debug_print!(
                        "Error: {:?} No entry for {:?}",
                        DnsError::NoDataFound,
                        name
                    );
                    false
                }
            }
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {}

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {}
    }

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
            let mut constructor = DnsRef::default();

            // When
            let contract = client
                .instantiate("dns", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Dns>();

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
            let mut constructor = DnsRef::new(false);
            let contract = client
                .instantiate("dns", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Dns>();

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
