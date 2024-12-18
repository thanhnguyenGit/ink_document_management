/// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
///
/// When running these you need to make sure that you:
/// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
/// - Are running a Substrate node which contains `pallet-contracts` in the background
#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    /// Imports all the definitions from the outer scope so we can use them here.
    use document_storage::document_management::{DocumentManagement, DocumentManagementRef, *};

    /// A helper function used for calling contract messages.
    use ink_e2e::{subxt::dynamic::Value, ChainBackend, ContractsBackend};

    /// The End-to-End test `Result` type.
    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// We test that we can upload and instantiate the contract using its default constructor.
    #[ink_e2e::test]
    async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Given
        let mut constructor = DocumentManagementRef::new();
        let contract = client
            .instantiate("document_management", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let document_id: u32 = 1;
        // When
        let mut call_builder = contract.call_builder::<DocumentManagement>();
        let _document_new = client
            .call(&ink_e2e::alice(), &call_builder.document_new(1))
            .submit()
            .await;
        // Then
        let get_result = client
            .call(&ink_e2e::alice(), &call_builder.verify_document_owner(1))
            .dry_run()
            .await?;
        assert!(matches!(get_result.return_value(), true));

        Ok(())
    }

    /// We test that we can read and write a value from the on-chain contract.
    #[ink_e2e::test]
    async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // // Given
        // let mut constructor = DocumentManagementRef::default();
        // let contract = client
        //     .instantiate("document_management", &ink_e2e::alice(), &mut constructor)
        //     .submit()
        //     .await
        //     .expect("instantiate failed");
        // let mut call_builder = contract.call_builder::<DocumentManagement>();
        //
        // let get = call_builder.get();
        // let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        // assert!(matches!(get_result.return_value(), false));
        //
        // // When
        // let flip = call_builder.flip();
        // let _flip_result = client
        //     .call(&ink_e2e::bob(), &flip)
        //     .submit()
        //     .await
        //     .expect("flip failed");
        //
        // // Then
        // let get = call_builder.get();
        // let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        // assert!(matches!(get_result.return_value(), true));
        //
        Ok(())
    }
}
