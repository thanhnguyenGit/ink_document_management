#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod document_management {
    use ink::env::call;
    use ink::primitives;
    use ink::scale::{Decode, Encode};
    use ink::storage::Mapping;
    use ink::{
        env::caller,
        prelude::{
            collections::{BinaryHeap, HashMap, HashSet},
            vec,
        },
    };

    // documentID represent ERC721 - non fungiable token
    pub type DocumentId = u32;

    //helper type
    pub type DocumentResult<T> = Result<T, DocumentError>;
    pub type IPFSaddr = Hash;
    #[ink(storage)]
    pub struct DocumentManagement {
        document_owner: Mapping<DocumentId, AccountId>,
        document_content: Mapping<DocumentId, Hash>,
        document_metadata: Mapping<DocumentId, Hash>,
        // a proxy to manage the document on behalf of publisher
        document_proxy: Mapping<DocumentId, AccountId>,
        // store the file on IPFS, map the document id to the ipfs addr
        document_location: Mapping<DocumentId, IPFSaddr>,
    }

    #[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum DocumentError {
        NotPublisher,
        NotAuthors,
        DocumentNotFound,
        DocumentIdAlreadyExists,
        CannotInsert,
        CannotDelete,
        NotAllow,
        DuplicationData,
        NotOwner,
        NoDataFound,
    }

    #[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum UpdateMessage {
        ContentUpdate,
        OwnverUpdate,
        DocumentDelete,
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
        action: UpdateMessage,
        #[ink(topic)]
        from: AccountId,
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
                document_location: Default::default(),
            }
        }
        //Create a new document
        #[ink(message)]
        pub fn document_new(&mut self, document_id: DocumentId) -> DocumentResult<()> {
            let caller = self.env().caller();
            self.add_document_to(&caller, document_id)?;
            self.env().emit_event(Transfer {
                from: Some(AccountId::from([0x0; 32])),
                to: Some(caller),
                id: document_id,
            });
            Ok(())
        }
        // check if document exist with an owner
        #[ink(message)]
        pub fn verify_document_owner(&self, document_id: DocumentId) -> bool {
            self.document_owner.contains(document_id)
        }
        // burn the document, only owner can do this
        #[ink(message)]
        pub fn burn_document(&mut self, document_id: DocumentId) -> DocumentResult<()> {
            let caller = self.env().caller();
            if self.check_owner_owned_document(&caller, &document_id) == false {
                return Err(DocumentError::NotOwner);
            }
            match self.document_owner.get(document_id) {
                Some(_) => {
                    self.document_owner.remove(document_id);
                    self.env().emit_event(DocumentUpdated {
                        action: UpdateMessage::DocumentDelete,
                        from: caller,
                        id: document_id,
                    });
                    Ok(())
                }
                None => Err(DocumentError::DocumentNotFound),
            }
        }
        //create a new content for the document
        #[ink(message)]
        pub fn document_content_new(
            &mut self,
            document_id: DocumentId,
            cont: Hash,
        ) -> DocumentResult<()> {
            let caller = self.env().caller();
            if !self.document_owner.contains(document_id) {
                return Err(DocumentError::DocumentNotFound);
            }

            match self.document_content.try_get(document_id) {
                Some(_) => Err(DocumentError::DocumentIdAlreadyExists),
                None => {
                    self.document_content.insert(document_id, &cont);
                    self.env().emit_event(DocumentUpdated {
                        action: UpdateMessage::OwnverUpdate,
                        from: caller,
                        id: document_id,
                    });
                    Ok(())
                }
            }
        }
        //get document content
        #[ink(message)]
        pub fn document_content_get(&self, document_id: DocumentId) -> Option<Hash> {
            self.document_content.get(&document_id).clone()
        }
        //check if document content exist
        #[ink(message)]
        pub fn verify_document_content(&self, document_id: DocumentId) -> bool {
            self.document_content.contains(&document_id)
        }
        #[ink(message)]
        pub fn remove_document_content(&mut self, document_id: DocumentId) -> DocumentResult<()> {
            let caller = self.env().caller();
            match self.check_owner_owned_document(&caller, &document_id) {
                true => Ok(self.document_content.remove(document_id)),
                false => Err(DocumentError::NotOwner),
            }
        }
        // add a new IPFS addr to the corresponding document
        #[ink(message)]
        pub fn document_addr_new(
            &mut self,
            document_id: DocumentId,
            ipfs_addr: IPFSaddr,
        ) -> DocumentResult<()> {
            let caller = self.env().caller();
            if self.check_owner_owned_document(&caller, &document_id) == false {
                return Err(DocumentError::NotOwner);
            }
            match self.document_location.try_get(document_id) {
                Some(_) => Err(DocumentError::DocumentIdAlreadyExists),
                None => {
                    let _insert = self.document_location.insert(document_id, &ipfs_addr);
                    self.env().emit_event(DocumentUpdated {
                        action: UpdateMessage::ContentUpdate,
                        from: caller,
                        id: document_id,
                    });
                    Ok(())
                }
            }
        }
        // get the ipfs addr of the document
        #[ink(message)]
        pub fn document_location_get(&self, document_id: DocumentId) -> DocumentResult<IPFSaddr> {
            match self.document_location.get(document_id) {
                Some(ipfs_addr) => Ok(ipfs_addr),
                None => Err(DocumentError::DocumentNotFound),
            }
        }
        // verify the IPFS is stored to the corresponded document id
        #[ink(message)]
        pub fn verify_document_location(&self, document_id: DocumentId) -> DocumentResult<bool> {
            match self.document_location.get(document_id) {
                Some(ipfs_addr) => Ok(true),
                None => Err(DocumentError::DocumentNotFound),
            }
        }
        //remove the ipfs location of the corresponding document
        #[ink(message)]
        pub fn document_location_delete(&mut self, document_id: DocumentId) -> DocumentResult<()> {
            let caller = self.env().caller();
            if self.check_owner_owned_document(&caller, &document_id) == false {
                return Err(DocumentError::NotOwner);
            }
            match self.document_location.get(document_id) {
                Some(_) => {
                    self.document_location.remove(document_id);
                    self.env().emit_event(DocumentUpdated {
                        action: UpdateMessage::ContentUpdate,
                        from: caller,
                        id: document_id,
                    });
                    Ok(())
                }
                None => Err(DocumentError::NoDataFound),
            }
        }

        fn add_document_to(&mut self, to: &AccountId, id: DocumentId) -> DocumentResult<()> {
            if *to == AccountId::from([0x0; 32]) {
                return Err(DocumentError::NotAllow);
            }
            match self.document_owner.try_get(id) {
                Some(_) => Err(DocumentError::DocumentIdAlreadyExists),
                None => {
                    self.document_owner.insert(id, to);
                    Ok(())
                }
            }
        }

        fn check_owner_owned_document(&self, caller: &AccountId, document_id: &DocumentId) -> bool {
            if let val = caller {
                return *val == self.document_owner.get(&document_id).expect("Val exist");
            }
            false
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
