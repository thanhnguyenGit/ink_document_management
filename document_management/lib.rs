#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod document_management {
    use ink::env::call;
    use ink::primitives::{self};
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
    pub type IPFSaddr = [u8; 32];
    pub type HashedValue = [u8; 32];
    #[ink(storage)]
    pub struct DocumentManagement {
        // Mapping documentId to owner accountId
        document_owner: Mapping<DocumentId, AccountId>,
        document_content: Mapping<DocumentId, HashedValue>,
        document_metadata: Mapping<DocumentId, HashedValue>,
        // store the file on IPFS, map the document id to the ipfs addr
        document_location: Mapping<DocumentId, IPFSaddr>,
        // store total document owned by this accountId
        owned_document_counter: Mapping<AccountId, u32>,
        // store the operator accounts that can manage the documents on the owner
        operator_approvals: Mapping<(AccountId, AccountId), bool>,
        // store an approved account that can only interact with that particuler documentId
        document_approvals: Mapping<DocumentId, AccountId>,
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
        AccountNotFound,
        CannotFetchValue,
    }

    #[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum UpdateMessage {
        ContentUpdate,
        OwnverUpdate,
        DocumentDelete,
        MetadataUpdate,
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

    //Emit event when Document get updated
    #[ink(event)]
    pub struct DocumentUpdated {
        #[ink(topic)]
        action: UpdateMessage,
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        id: DocumentId,
    }

    //Emit event when Role get updated
    #[ink(event)]
    pub struct RoleUpdated {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        id: AccountId,
    }

    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: DocumentId,
    }

    impl DocumentManagement {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                document_owner: Default::default(),
                document_content: Default::default(),
                document_metadata: Default::default(),
                document_location: Default::default(),
                document_approvals: Default::default(),
                owned_document_counter: Default::default(),
                operator_approvals: Default::default(),
            }
        }
        //Create a new document
        #[ink(message)]
        pub fn document_new(&mut self, document_id: DocumentId) -> DocumentResult<()> {
            let caller = self.env().caller();
            self.add_document_to(&caller, document_id)?;
            self.increase_documents_count(&caller);
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
                    self.decrease_documents_count(&caller);
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
        //get the document owner
        #[ink(message)]
        pub fn document_owner_get(&self, document_id: DocumentId) -> DocumentResult<AccountId> {
            match self.document_owner.get(document_id) {
                Some(owner) => Ok(owner),
                None => Err(DocumentError::DocumentNotFound),
            }
        }
        //create a new content for the document
        #[ink(message)]
        pub fn document_content_new(
            &mut self,
            document_id: DocumentId,
            cont: [u8; 32],
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
        pub fn document_content_get(&self, document_id: DocumentId) -> Option<HashedValue> {
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
                Some(_) => Ok(true),
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
        //create a new document metadata
        #[ink(message)]
        pub fn document_metadata_new(
            &mut self,
            document_id: DocumentId,
            metadata_hash: HashedValue,
        ) -> DocumentResult<()> {
            let caller = self.env().caller();
            if self.check_owner_owned_document(&caller, &document_id) {
                return Err(DocumentError::NotOwner);
            }
            match self.document_metadata.get(document_id) {
                Some(_) => Err(DocumentError::DocumentIdAlreadyExists),
                None => {
                    self.document_metadata.insert(document_id, &metadata_hash);
                    self.env().emit_event(DocumentUpdated {
                        action: UpdateMessage::MetadataUpdate,
                        from: caller,
                        id: document_id,
                    });
                    Ok(())
                }
            }
        }
        //get document metadata
        #[ink(message)]
        pub fn document_metadata_get(&self, document_id: DocumentId) -> DocumentResult<[u8; 32]> {
            match self.document_metadata.get(document_id) {
                Some(meta_hash) => Ok(meta_hash),
                None => Err(DocumentError::DocumentNotFound),
            }
        }
        //verify if the metadata stored is for the document
        #[ink(message)]
        pub fn verify_document_metadata(&self, document_id: DocumentId) -> DocumentResult<bool> {
            match self.document_metadata.get(document_id) {
                Some(_) => Ok(true),
                None => Err(DocumentError::DocumentNotFound),
            }
        }
        //delete the document metadata from the storage
        #[ink(message)]
        pub fn document_metadata_delete(&mut self, document_id: DocumentId) -> DocumentResult<()> {
            let caller = self.env().caller();
            if self.check_owner_owned_document(&caller, &document_id) == false {
                return Err(DocumentError::NotOwner);
            }
            match self.document_metadata.get(document_id) {
                Some(_) => {
                    self.document_metadata.remove(document_id);
                    self.env().emit_event(DocumentUpdated {
                        action: UpdateMessage::MetadataUpdate,
                        from: caller,
                        id: document_id,
                    });
                    Ok(())
                }
                None => Err(DocumentError::NoDataFound),
            }
        }
        //get the approved accountID for this documentID
        #[ink(message)]
        pub fn document_get_approve_acc(
            &self,
            document_id: DocumentId,
        ) -> DocumentResult<AccountId> {
            match self.document_approvals.get(document_id) {
                Some(aproved_acc) => Ok(aproved_acc),
                None => Err(DocumentError::AccountNotFound),
            }
        }
        //get number of owned documents of an accountId
        #[ink(message)]
        pub fn numof_owned_documents(&self, owner: AccountId) -> u32 {
            self.owned_document_counter.get(owner).unwrap_or(0)
        }
        //transfer document to another account, only owner or approved account can do it
        #[ink(message)]
        pub fn tranfer_to(&mut self, to: AccountId, document_id: DocumentId) -> DocumentResult<()> {
            let caller = self.env().caller();
            self.transfer_document_from(&caller, &to, document_id);
            Ok(())
        }
        // transfer approved for owned toke
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            document_id: DocumentId,
        ) -> DocumentResult<()> {
            self.transfer_document_from(&from, &to, document_id)?;
            Ok(())
        }
        // approves the account to transfer the specific document on behalf of the caller
        #[ink(message)]
        pub fn approve(&mut self, to: AccountId, document_id: DocumentId) -> DocumentResult<()> {
            self.approve_for(&to, document_id)?;
            Ok(())
        }
        // approve or disapprove the operator fro all documentId of the caller
        #[ink(message)]
        pub fn set_approval_for_all(
            &mut self,
            to: AccountId,
            approved: bool,
        ) -> DocumentResult<()> {
            self.approve_for_all(to, approved)?;
            Ok(())
        }
        // get the approved accountId for this documentID
        #[ink(message)]
        pub fn get_approved_account(&self, document_id: DocumentId) -> DocumentResult<AccountId> {
            match self.document_approvals.get(document_id) {
                Some(acc) => Ok(acc),
                None => Err(DocumentError::NoDataFound),
            }
        }
        // return to see if the operator is approve by the ownver
        #[ink(message)]
        pub fn is_approve_for_all(
            &self,
            owner: AccountId,
            operator: AccountId,
        ) -> DocumentResult<bool> {
            Ok(self.approved_for_all(&owner, &operator))
        }

        ///Helper function
        fn add_document_to(&mut self, to: &AccountId, id: DocumentId) -> DocumentResult<()> {
            if *to == AccountId::from([0x0; 32]) {
                return Err(DocumentError::NotAllow);
            }
            match self.document_owner.get(id) {
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

        fn check_is_proxy(&self, owner: AccountId, operator: AccountId) -> DocumentResult<bool> {
            Ok(true)
        }

        fn increase_documents_count(&mut self, owner: &AccountId) {
            let count = self
                .owned_document_counter
                .get(owner)
                .map(|count| count.checked_add(1).expect("Overflow"))
                .unwrap_or(1);
            self.owned_document_counter.insert(owner, &count);
        }

        fn decrease_documents_count(&mut self, owner: &AccountId) -> DocumentResult<()> {
            let count = self
                .owned_document_counter
                .get(owner)
                .map(|counter| counter.checked_sub(1).expect("Cannot be negative"))
                .ok_or(DocumentError::CannotFetchValue)?;
            self.owned_document_counter.insert(owner, &count);
            Ok(())
        }
        fn approve_for_all(&mut self, to: AccountId, approved: bool) -> DocumentResult<()> {
            let caller = self.env().caller();
            if to == caller {
                return Err(DocumentError::NotAllow);
            }
            self.env().emit_event(ApprovalForAll {
                owner: caller,
                operator: to,
                approved,
            });
            if approved {
                self.operator_approvals.insert((&caller, &to), &approved);
            }
            Ok(())
        }
        fn remove_approve_account(&mut self, owner: AccountId, approved_acc: AccountId) {
            if self.operator_approvals.get((owner, approved_acc)).unwrap() == false {
                self.operator_approvals.remove((owner, approved_acc));
            }
        }
        fn approved_for_all(&self, owner: &AccountId, operator: &AccountId) -> bool {
            self.operator_approvals.contains((owner, operator))
        }
        fn approve_for(&mut self, to: &AccountId, document_id: DocumentId) -> DocumentResult<()> {
            let caller = self.env().caller();
            let owner = self.document_owner_get(document_id).unwrap();
            if !(self.check_owner_owned_document(&caller, &document_id)
                || self.approved_for_all(&owner, &caller))
            {
                return Err(DocumentError::NotAllow);
            }
            if *to == AccountId::from([0x0; 32]) {
                return Err(DocumentError::NotAllow);
            }
            if self.document_approvals.contains(document_id) {
                return Err(DocumentError::CannotInsert);
            } else {
                self.document_approvals.insert(document_id, to);
            }

            self.env().emit_event(Approval {
                from: caller,
                to: *to,
                id: document_id,
            });
            Ok(())
        }
        fn clear_approval(&mut self, document_id: DocumentId) -> DocumentResult<()> {
            match self.document_approvals.get(document_id) {
                Some(_) => Ok(self.document_approvals.remove(document_id)),
                None => Err(DocumentError::NoDataFound),
            }
        }
        fn approved_or_owner(
            &self,
            from: AccountId,
            owner: AccountId,
            document_id: DocumentId,
        ) -> DocumentResult<bool> {
            if from == AccountId::from([0x0; 32]) {
                return Err(DocumentError::CannotInsert);
            }
            Ok(from == owner
                || self.document_approvals.get(document_id) == Some(from)
                || self.approved_for_all(&owner, &from))
        }
        fn transfer_document_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            document_id: DocumentId,
        ) -> DocumentResult<()> {
            let caller = self.env().caller();
            let owner = self.document_owner_get(document_id).unwrap();
            if !self.approved_or_owner(caller, owner, document_id).unwrap() {
                return Err(DocumentError::NotAllow);
            }
            if owner != *from {
                return Err(DocumentError::NotOwner);
            }
            self.clear_approval(document_id);
            self.burn_document(document_id)?;
            self.add_document_to(to, document_id);
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                id: document_id,
            });
            Ok(())
        }
    }

    //Unit testing
    #[cfg(test)]
    mod tests {

        use super::*;

        pub type IPFSaddr = [u8; 32];

        #[ink::test]
        fn mint_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // create a new document
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
            // create a new document
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
            // create a new document
            let mut document = DocumentManagement::new();
            // try delete an non-existing document
            assert_eq!(
                document.burn_document(1),
                Err(DocumentError::DocumentNotFound)
            );
            // instantinate a document
        }
        #[ink::test]
        fn document_content_work() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut document = DocumentManagement::new();
            let content_hash: HashedValue = [12; 32];
            // ensure that document is created
            assert_eq!(document.document_new(1), Ok(()));
            // ensure that new document content is created, ensure that is the [u8;32]
            assert_eq!(document.document_content_new(1, content_hash), Ok(()));
            // ensure that new content hash belong to the corresponded documentId
            assert_eq!(document.document_content_get(1), Some(content_hash));
        }
    }

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
