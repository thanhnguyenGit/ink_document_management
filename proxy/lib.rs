#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod proxy_account {
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::{CallFlags, DefaultEnvironment};
    use ink::scale::{Decode, Encode};
    use ink::storage::{traits::ManualKey, Lazy, Mapping};

    #[ink(storage)]
    pub struct Proxy {
        version: u32,
        addresses: Mapping<AccountId, i32, ManualKey<0xAB>>,
        identities: Mapping<AccountId, u32, ManualKey<0x23>>,
        delegate_to: Lazy<Hash>,
    }

    // #[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
    // #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ProxyError {
        DuplicateHash,
    }

    impl Proxy {
        #[ink(constructor)]
        pub fn new(init_value: i32, hash: Hash) -> Self {
            let mut delegate_to = Lazy::new();
            delegate_to.set(&hash);
            Self::env().lock_delegate_dependency(&hash);
            Self {
                version: 1,
                addresses: Mapping::default(),
                identities: Mapping::default(),
                delegate_to,
            }
        }

        #[ink(message)]
        pub fn update_delegate_to(&mut self, new_hash: Hash) {
            if let Some(old_hash) = self.delegate_to.get() {
                self.env().unlock_delegate_dependency(&old_hash)
            }
            self.env().lock_delegate_dependency(&new_hash);
            self.delegate_to.set(&new_hash)
        }

        #[ink(message)]
        pub fn document_account_new_delegate(&mut self) {
            let selector = ink::selector_bytes!("document_account_new");
            let _ = build_call::<DefaultEnvironment>()
                .delegate(self.delegate_to())
                .call_flags(CallFlags::TAIL_CALL)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<()>()
                .try_invoke();
        }

        #[ink(message)]
        pub fn get_uuid(&self, account_id: AccountId) -> u32 {
            self.identities.get(account_id).unwrap()
        }

        fn delegate_to(&self) -> Hash {
            self.delegate_to
                .get()
                .expect("delegate_to always has a value")
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
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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
