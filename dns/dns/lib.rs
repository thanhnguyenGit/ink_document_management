#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod dns {
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
}
