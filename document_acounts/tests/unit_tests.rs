// / Unit tests in Rust are normally defined within such a `#[cfg(test)]`
// / module and test functions are marked with a `#[test]` attribute.
// / The below code is technically just normal Rust code.
#[cfg(test)]
mod tests {
    /// Imports all the definitions from the outer scope so we can use them here.
    use document_acounts::document_acounts::*;

    /// We test if the default constructor does its job.
    #[ink::test]
    fn default_works() {}

    /// We test a simple use case of our contract.
    #[ink::test]
    fn it_works() {}
}
