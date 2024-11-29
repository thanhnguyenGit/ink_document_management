#[cfg(test)]
mod tests {
    use super::*;
    use document_storage::document_management::*;

    #[ink::test]
    fn mint_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        // create a new document document contract instance
        let mut document = DocumentManagement::new();
        // assert that document is not yet exist
        assert_eq!(
            document.document_owner_get(1),
            Err(DocumentError::DocumentNotFound)
        );
        // check the number of owned document of Alice before minting a new document
        assert_eq!(document.numof_owned_documents(accounts.alice), 0);
        //
        assert_eq!(document.document_new(1), Ok(()));
        assert_eq!(document.numof_owned_documents(accounts.alice), 1);
    }
    #[ink::test]
    fn mint_an_existence_document() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        // create a new document contract instance
        let mut document = DocumentManagement::new();
        // instantiate value for the document
        assert_eq!(document.document_new(1), Ok(()));
        // ensure one event were emitted due from the instantiate transaction
        assert_eq!(ink::env::test::recorded_events().count(), 1);
        // ensure alice own the document instantiated
        assert_eq!(document.numof_owned_documents(accounts.alice), 1);
        // ensure that document with id: 1 is owned by alice
        assert_eq!(document.document_owner_get(1), Ok(accounts.alice));
        // ensure that a document id 1 is unique, cannot be mint again
        assert_eq!(
            document.document_new(1),
            Err(DocumentError::DocumentIdAlreadyExists)
        );
    }
    #[ink::test]
    fn burn_a_document() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        // create a new document contract instance
        let mut document = DocumentManagement::new();
        // Alice create a new document with id 1
        assert_eq!(document.document_new(1), Ok(()));
        assert_eq!(document.numof_owned_documents(accounts.alice), 1);
        assert_eq!(document.document_owner_get(1), Ok(accounts.alice));
        // delete the document with id: 1
        assert_eq!(document.burn_document(1), Ok(()));
        // Alice now do not own the document with id: 1
        assert_eq!(document.numof_owned_documents(accounts.alice), 0);
    }
    #[ink::test]
    fn burn_a_non_exist_document() {
        // instantinate the contract
        let mut document = DocumentManagement::new();
        // try delete an non-existing document
        assert_eq!(
            document.burn_document(2),
            Err(DocumentError::DocumentNotFound)
        );
    }
    #[ink::test]
    fn document_content_work() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let mut document = DocumentManagement::new();
        let content_hash: [u8; 32] = [0x00; 32];
        // ensure that document is created
        assert_eq!(document.document_new(1), Ok(()));
        // ensure that new document content is created, ensure that is the [u8;32]
        assert_eq!(
            document.document_content_new(1, content_hash.into()),
            Ok(())
        );
        // ensure that new content hash belong to the corresponded documentId
        assert_eq!(document.document_content_get(1), Some(content_hash.into()));
    }
}
