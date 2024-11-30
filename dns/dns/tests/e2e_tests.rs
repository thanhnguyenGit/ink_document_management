#![cfg_attr(not(feature = "std"), no_std, no_main)]
/// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
///
/// When running these you need to make sure that you:
/// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
/// - Are running a Substrate node which contains `pallet-contracts` in the background
#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    /// Imports all the definitions from the outer scope so we can use them here.
    use dns::dns::*;

    /// A helper function used for calling contract messages.
    use ink_e2e::ContractsBackend;

    /// The End-to-End test `Result` type.
    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// We test that we can upload and instantiate the contract using its default constructor.
    #[ink_e2e::test]
    async fn dns_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Given
        let mut constructor = DnsRef::new();

        // When
        let contract = client
            .instantiate("dns", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let call_builder = contract.call_builder::<Dns>();

        // Then

        Ok(())
    }

    /// We test that we can read and write a value from the on-chain contract.
    #[ink_e2e::test]
    async fn dns_also_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Given
        let mut constructor = DnsRef::new();
        let contract = client
            .instantiate("dns", &ink_e2e::bob(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = contract.call_builder::<Dns>();

        // When

        // Then

        Ok(())
    }
}
