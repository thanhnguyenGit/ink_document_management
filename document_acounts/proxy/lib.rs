#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod proxy_account {
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::{CallFlags, DefaultEnvironment};
    use ink::scale::{Decode, Encode};
    use ink::storage::{traits::ManualKey, Lazy, Mapping};
    use scale_info::TypeInfo;

    type ProxyResult<T> = Result<T, ProxyError>;
    type UUID = u32;
    #[ink(storage)]
    pub struct Proxy {
        developer: AccountId,
        contract_version: Lazy<u32, ManualKey<0x7672736e>>,
        identities: Mapping<AccountId, UUID, ManualKey<0x6964656e>>,
        delegatee_version: Lazy<u32, ManualKey<0x646c7673>>,
        delegate_to: Lazy<Hash>,
    }
    #[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ProxyError {
        CodeHashNotExist,
        NoDataFound,
        NotAllow,
    }
    #[ink(event)]
    pub struct ProxyTransactionEvent {
        #[ink(topic)]
        delegatee: Hash,
        #[ink(topic)]
        event: ProxyEvent,
    }
    #[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ProxyEvent {
        Updated,
        ChangedCodeHash,
    }
    impl Proxy {
        #[ink(constructor)]
        pub fn new(init_value: i32, hash: Hash) -> Self {
            let developer = Self::env().caller();
            let mut delegate_to = Lazy::new();
            delegate_to.set(&hash);
            Self::env().lock_delegate_dependency(&hash);
            Self {
                developer,
                identities: Mapping::default(),
                contract_version: Lazy::default(),
                delegatee_version: Lazy::default(),
                delegate_to,
            }
        }

        #[ink(message)]
        pub fn update_delegate_to(&mut self, new_hash: Hash) {
            if let Some(old_hash) = self.delegate_to.get() {
                self.env().unlock_delegate_dependency(&old_hash)
            }
            self.env().lock_delegate_dependency(&new_hash);
            self.delegate_to.set(&new_hash);
            self.env().emit_event(ProxyTransactionEvent {
                delegatee: new_hash,
                event: ProxyEvent::ChangedCodeHash,
            });
        }
        /// Adds entry to `identities` using delegate call.
        /// Note that we don't need `CallFlags::TAIL_CALL` flag
        /// because `Mapping` updates the storage instantly on-demand.
        #[ink(message)]
        pub fn delcall_document_account_new(&mut self) {
            let gas_before = self.env().gas_left();
            ink::env::debug_print!("Gas before delegate call {}", gas_before);
            let selector = ink::selector_bytes!("document_account_new");
            build_call::<DefaultEnvironment>()
                .delegate(self.get_delegatee_hash())
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke();
            let gas_after = self.env().gas_left();
            ink::env::debug_print!("Gas after delegate call {}", gas_after);
        }
        #[ink(message)]
        pub fn delcall_document_account_version(&mut self) {
            let gas_before = self.env().gas_left();
            ink::env::debug_print!("Gas before delegate call {}", gas_before);
            let selector = ink::selector_bytes!("document_account_version");
            build_call::<DefaultEnvironment>()
                .delegate(self.get_delegatee_hash())
                .call_flags(CallFlags::TAIL_CALL)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke();
            let gas_after = self.env().gas_left();
            ink::env::debug_print!("Gas after delegate call {}", gas_after);
        }
        #[ink(message)]
        pub fn contract_version_new(&mut self, version: u32) -> ProxyResult<()> {
            self.contract_version.set(&version);
            Ok(())
        }
        #[ink(message)]
        pub fn contract_version_get(&self) -> ProxyResult<u32> {
            match self.contract_version.get() {
                Some(value) => Ok(value),
                None => Err(ProxyError::NoDataFound),
            }
        }
        #[ink(message)]
        pub fn get_uuid(&self, account_id: AccountId) -> u32 {
            self.identities.get(account_id).unwrap()
        }
        #[ink(message)]
        pub fn get_delegatee_version(&self) -> ProxyResult<u32> {
            match self.delegatee_version.get() {
                Some(version) => Ok(version),
                None => Err(ProxyError::NoDataFound),
            }
        }
        #[ink(message)]
        pub fn get_delegatee_code_hash(&self) -> ProxyResult<Hash> {
            match self.check_is_developer() {
                true => Ok(self.get_delegatee_hash()),
                false => Err(ProxyError::NotAllow),
            }
        }
        #[ink(message)]
        pub fn get_proxy_code_hash(&self) -> ProxyResult<Hash> {
            match self.check_is_developer() {
                true => Ok(self
                    .env()
                    .own_code_hash()
                    .expect("Contract should have code hash,bruh :v")),
                false => Err(ProxyError::NotAllow),
            }
        }

        fn get_delegatee_hash(&self) -> Hash {
            self.delegate_to
                .get()
                .expect("delegate_to always has a value")
        }
        fn check_is_developer(&self) -> bool {
            let caller = self.env().caller();
            return caller == self.developer;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {}

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
        async fn e2e_encouter_mutated(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = ProxyRef::default();

            // When
            let contract = client
                .instantiate("proxy", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Proxy>();

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
            let mut constructor = ProxyRef::new(false);
            let contract = client
                .instantiate("proxy", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Proxy>();

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
