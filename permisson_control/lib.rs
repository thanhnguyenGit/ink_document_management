#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod permisson_control {
    use ink::prelude::vec;
    use ink::storage::Mapping;
    use ink::{
        primitives::Key,
        storage::{traits::ManualKey, Lazy, Mapping, StorageVec},
    };
    use scale::{Decode, Encode};
    use scale_info::TypeInfo;

    type TransactionId = u32;
    type MAX_ADMINS = u32;

    #[ink(storage)]
    pub struct PermissonControl {
        confirm_transactions: Mapping<(TransactionId, AccountId), ()>,
        confirm_transactions_count: Mapping<TransactionId, u32>,
        transactions: Mapping<TransactionId, Transaction>,
        transaction_list: TransactionList,
        admins_list: StorageVec<AccountId>,
        is_admins: Mapping<AccountId, ()>,
        min_admins_requirement: u32,
    }
    #[derive(Debug, Clone, Decode, Encode, PartialEq, Eq, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Transaction {
        pub callee: AccountId,
        pub selector: [u8; 4],
        pub selector_inputs: Vec<u8>,
        pub transferred_value: Balance,
        pub ref_time_limit: u64,
        pub allow_reentry: bool,
    }
    #[derive(Debug, Default, Clone, Decode, Encode, PartialEq, Eq, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct TransactionList {
        transactions: Vec<TransactionId>,
        next_id: TransactionId,
    }

    impl Default for PermissonControl {
        fn default() -> Self {
            Self {
                confirm_transactions: Mapping::default(),
                confirm_transactions_count: Mapping::default(),
                transactions: Mapping::default(),
                transaction_list: Default::default(),
                admins_list: StorageVec::default(),
                is_admins: Mapping::default(),
                min_admins_requirement: Default::default(),
            }
        }
    }
    impl PermissonControl {
        #[ink(constructor)]
        pub fn new(mut admins: Vec<AccountId>) -> Self {
            let mut _init_contract = Self::default();
            admins.sort_unstable();
            admins.dedup();
            Self::ensure_min_of_require_admins(
                &mut _init_contract,
                u32::try_from(admins.len()).expect("the admins input should not be empty"),
            );
            for admin in admins.iter() {
                _init_contract.is_admins.insert(admin, &());
            }
            _init_contract
        }

        #[ink(message)]
        pub fn flip(&mut self) {}

        #[ink(message)]
        pub fn get(&self) -> bool {
            true
        }

        fn ensure_min_of_require_admins(&mut self, admins: u32) {
            self.min_admins_requirement = admins * 2 / 3;
            assert!(0 < self.min_admins_requirement && self.min_admins_requirement < admins)
        }
    }

    // #[cfg(test)]
    //  mod tests {
    //      use super::*;
    //
    //      #[ink::test]
    //      fn default_works() {
    //          let permisson_control = PermissonControl::default();
    //          assert_eq!(permisson_control.get(), false);
    //      }
    //
    //      #[ink::test]
    //      fn it_works() {
    //          let mut permisson_control = PermissonControl::new(false);
    //          assert_eq!(permisson_control.get(), false);
    //          permisson_control.flip();
    //          assert_eq!(permisson_control.get(), true);
    //      }
    //  }
    //
    //
    // #[cfg(all(test, feature = "e2e-tests"))]
    //  mod e2e_tests {
    //      /// Imports all the definitions from the outer scope so we can use them here.
    //      use super::*;
    //
    //      /// A helper function used for calling contract messages.
    //      use ink_e2e::ContractsBackend;
    //
    //      /// The End-to-End test `Result` type.
    //      type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    //
    //      /// We test that we can upload and instantiate the contract using its default constructor.
    //      #[ink_e2e::test]
    //      async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //          // Given
    //          let mut constructor = PermissonControlRef::default();
    //
    //          // When
    //          let contract = client
    //              .instantiate("permisson_control", &ink_e2e::alice(), &mut constructor)
    //              .submit()
    //              .await
    //              .expect("instantiate failed");
    //          let call_builder = contract.call_builder::<PermissonControl>();
    //
    //          // Then
    //          let get = call_builder.get();
    //          let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
    //          assert!(matches!(get_result.return_value(), false));
    //
    //          Ok(())
    //      }
    //
    //      /// We test that we can read and write a value from the on-chain contract.
    //      #[ink_e2e::test]
    //      async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //          // Given
    //          let mut constructor = PermissonControlRef::new(false);
    //          let contract = client
    //              .instantiate("permisson_control", &ink_e2e::bob(), &mut constructor)
    //              .submit()
    //              .await
    //              .expect("instantiate failed");
    //          let mut call_builder = contract.call_builder::<PermissonControl>();
    //
    //          let get = call_builder.get();
    //          let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
    //          assert!(matches!(get_result.return_value(), false));
    //
    //          // When
    //          let flip = call_builder.flip();
    //          let _flip_result = client
    //              .call(&ink_e2e::bob(), &flip)
    //              .submit()
    //              .await
    //              .expect("flip failed");
    //
    //          // Then
    //          let get = call_builder.get();
    //          let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
    //          assert!(matches!(get_result.return_value(), true));
    //
    //          Ok(())
    //      }
    //  }
}
