#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod permisson_control {
    use ink::prelude::vec;
    use ink::{
        primitives::Key,
        storage::{traits::ManualKey, Lazy, Mapping, StorageVec},
    };
    use scale::{Decode, Encode};
    use scale_info::TypeInfo;

    type TransactionId = u32;
    type MAX_ADMINS = u32;

    const MAX_ADMINS: u32 = 50;

    #[ink(storage)]
    pub struct PermissonControl {
        confirm_transactions: Mapping<(TransactionId, AccountId), ()>,
        // redudant information to speed up check if the transaction is confirmed
        confirm_transactions_index: Mapping<TransactionId, u32>,
        admins: Mapping<AccountId, ()>,
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

    #[derive(Debug, Clone, Decode, Encode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum PermissionError {
        NotAdmin,
        InvalidOperation,
        NoDataFound,
    }

    type PermissonResult<T> = Result<T, PermissionError>;

    #[ink(event)]
    pub struct Event {}
    impl Default for PermissonControl {
        fn default() -> Self {
            Self {
                confirm_transactions: Mapping::default(),
                confirm_transactions_index: Mapping::default(),
                admins: Mapping::default(),
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
                let _ = _init_contract.admins.try_insert(admin, &());
            }
            _init_contract
        }

        #[ink(message)]
        pub fn add_admins(&mut self, new_admins: AccountId) -> PermissonResult<()> {
            self.ensure_from_wallet();
            self.ensure_no_admin(&new_admins);
            let _ = self.admins.try_insert(new_admins, &());
            self.env().emit_event(Event {});
            Ok(())
        }

        #[ink(message)]
        pub fn remove_admins(&mut self, removed_admins: AccountId) -> PermissonResult<()> {
            self.ensure_from_wallet();
            self.ensure_is_admin(&removed_admins);
            match self.admins.get(removed_admins) {
                Some(val) => self.admins.remove(removed_admins),
                None => return Err(PermissionError::NoDataFound),
            }
            Ok(())
        }
        #[ink(message)]
        pub fn replace_admins(&mut self, from: AccountId, to: AccountId) -> PermissonResult<()> {
            self.ensure_from_wallet();
            self.ensure_is_admin(&from);
            self.ensure_no_admin(&to);
            match self.admins.get(from) {
                Some(val) => {
                    self.admins.remove(from);
                    self.admins.insert(to, &());
                }
                None => return Err(PermissionError::InvalidOperation),
            }
            Ok(())
        }
        fn ensure_min_of_require_admins(&mut self, admins: u32) {
            self.min_admins_requirement = admins * 2 / 3;
            assert!(2 < self.min_admins_requirement && self.min_admins_requirement < admins)
        }
        fn ensure_from_wallet(&self) {
            assert_eq!(self.env().caller(), self.env().account_id())
        }
        fn ensure_no_admin(&self, account_id: &AccountId) {
            assert!(!self.admins.contains(account_id))
        }
        fn ensure_is_admin(&self, account_id: &AccountId) {
            assert!(self.admins.contains(account_id))
        }
    }
}
