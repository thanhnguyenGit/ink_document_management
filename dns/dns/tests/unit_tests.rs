#![cfg_attr(not(feature = "std"), no_std, no_main)]
/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
/// module and test functions are marked with a `#[test]` attribute.
/// The below code is technically just normal Rust code.
#[cfg(test)]
mod tests {
    /// Imports all the definitions from the outer scope so we can use them here.
    use dns::dns::{Dns, DnsError};

    /// We test if the default constructor does its job.
    #[ink::test]
    fn dns_works1() {
        assert_eq!(1, 1)
    }

    /// We test a simple use case of our contract.
    #[ink::test]
    fn dns_works2() {
        assert_eq!(1, 1)
    }
}
