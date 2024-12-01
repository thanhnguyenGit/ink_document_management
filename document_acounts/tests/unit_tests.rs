#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[cfg(test)]
mod tests {
    use document_acounts::document_acounts::*;

    #[ink::test]
    fn doc_acc_works1() {
        assert_eq!(1, 1)
    }

    #[ink::test]
    fn doc_acc_works() {
        assert_eq!(1, 1)
    }
}
