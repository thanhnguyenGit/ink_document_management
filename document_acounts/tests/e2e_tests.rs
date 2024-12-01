#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    /// Imports all the definitions from the outer scope so we can use them here.
    use document_acounts::document_acounts::{DocumentAcounts, DocumentAcountsRef};

    /// A helper function used for calling contract messages.
    use ink_e2e::ContractsBackend;

    /// The End-to-End test `Result` type.
    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// We test that we can upload and instantiate the contract using its default constructor.
    #[ink_e2e::test]
    async fn doc_acc_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Given
        // let mut constructor = DocumentAcountsRef::new();
        //
        // // When
        // let contract = client
        //     .instantiate("document_acounts", &ink_e2e::alice(), &mut constructor)
        //     .submit()
        //     .await
        //     .expect("instantiate failed");
        // let call_builder = contract.call_builder::<DocumentAcounts>();
        //
        // Then

        Ok(())
    }

    /// We test that we can read and write a value from the on-chain contract.
    #[ink_e2e::test]
    async fn doc_acc_also_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Given
        // let mut constructor = DocumentAcountsRef::new();
        // let contract = client
        //     .instantiate("document_acounts", &ink_e2e::bob(), &mut constructor)
        //     .submit()
        //     .await
        //     .expect("instantiate failed");
        // let mut call_builder = contract.call_builder::<DocumentAcounts>();
        //
        // When

        // Then

        Ok(())
    }
}
