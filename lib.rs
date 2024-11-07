#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod document_management {
    use ink::prelude::{
        collections::{BinaryHeap, HashMap, HashSet},
        vec,
    };
    use ink::primitives;
    use ink::scale::{Decode, Encode};
    use ink::storage::Mapping;

    // documentID represent ERC721 - non fungiable token
    pub type DocumentId = u32;

    #[ink(storage)]
    pub struct DocumentManagement {
        document_owner: Mapping<DocumentId, AccountId>,
        document_content: Mapping<DocumentId, Hash>,
        document_metadata: Mapping<DocumentId, Hash>,
        // a proxy to manage the document on behalf of publisher
        document_proxy: Mapping<DocumentId, AccountId>,
    }

    #[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
    pub enum DocumentError {
        NotPublisher,
        NotAuthors,
        DocumentNonFound,
        DocumentIdAlreadyExists,
        CannotInsert,
        CannotDelete,
        RestrictedAccess,
        DuplicationData,
    }

    // Emit event when document get transfer
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: DocumentId,
    }

    //Emit event when a proxy approve an document
    #[ink(event)]
    pub struct ProxyUpdated {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: DocumentId,
    }

    //Emit event when Document get update
    #[ink(event)]
    pub struct DocumentUpdated {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        id: DocumentId,
    }

    impl DocumentManagement {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                document_owner: Default::default(),
                document_content: Default::default(),
                document_proxy: Default::default(),
                document_metadata: Default::default(),
            }
        }

        //Create a new document
        #[ink(message)]
        pub fn document_new(&mut self, document_id: DocumentId) -> Result<(), DocumentError> {
            Ok(())
        }

        fn add_document_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            id: DocumentId,
        ) -> Result<(), DocumentError> {
            if *to == AccountId::from([0x0; 32]) {
                return Err(DocumentError::RestrictedAccess);
            }
            match self.document_owner.try_insert(id, to) {
                Ok(Some(_)) => self.env().emit_event(Transfer {
                    from: Some(*from),
                    to: Some(*to),
                    id,
                }),
                Err(err) => Err(DocumentError::DocumentIdAlreadyExists),
            }
            Ok(())
        }
    }

    //// Unit testing
    //#[cfg(test)]
    //mod tests {
    //    use super::*;
    //
    //    #[ink::test]
    //    fn default_works() {
    //        let document_management = DocumentManagement::default();
    //        assert_eq!(document_management.get(), false);
    //    }
    //
    //    #[ink::test]
    //    fn it_works() {
    //        let mut document_management = DocumentManagement::new(false);
    //        assert_eq!(document_management.get(), false);
    //        document_management.flip();
    //        assert_eq!(document_management.get(), true);
    //    }
    //}
    //
    ///// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    /////
    ///// When running these you need to make sure that you:
    ///// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    ///// - Are running a Substrate node which contains `pallet-contracts` in the background
    //#[cfg(all(test, feature = "e2e-tests"))]
    //mod e2e_tests {
    //    /// Imports all the definitions from the outer scope so we can use them here.
    //    use super::*;
    //
    //    /// A helper function used for calling contract messages.
    //    use ink_e2e::ContractsBackend;
    //
    //    /// The End-to-End test `Result` type.
    //    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    //
    //    /// We test that we can upload and instantiate the contract using its default constructor.
    //    #[ink_e2e::test]
    //    async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //        // Given
    //        let mut constructor = DocumentManagementRef::default();
    //
    //        // When
    //        let contract = client
    //            .instantiate("document_management", &ink_e2e::alice(), &mut constructor)
    //            .submit()
    //            .await
    //            .expect("instantiate failed");
    //        let call_builder = contract.call_builder::<DocumentManagement>();
    //
    //        // Then
    //        let get = call_builder.get();
    //        let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
    //        assert!(matches!(get_result.return_value(), false));
    //
    //        Ok(())
    //    }
    //
    //    /// We test that we can read and write a value from the on-chain contract.
    //    #[ink_e2e::test]
    //    async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //        // Given
    //        let mut constructor = DocumentManagementRef::new(false);
    //        let contract = client
    //            .instantiate("document_management", &ink_e2e::bob(), &mut constructor)
    //            .submit()
    //            .await
    //            .expect("instantiate failed");
    //        let mut call_builder = contract.call_builder::<DocumentManagement>();
    //
    //        let get = call_builder.get();
    //        let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
    //        assert!(matches!(get_result.return_value(), false));
    //
    //        // When
    //        let flip = call_builder.flip();
    //        let _flip_result = client
    //            .call(&ink_e2e::bob(), &flip)
    //            .submit()
    //            .await
    //            .expect("flip failed");
    //
    //        // Then
    //        let get = call_builder.get();
    //        let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
    //        assert!(matches!(get_result.return_value(), true));
    //
    //        Ok(())
    //    }
    //}
}
