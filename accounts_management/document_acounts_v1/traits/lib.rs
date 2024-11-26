#![cfg_attr(not(feature = "std"), no_std, no_main)]

use core::ops::Deref;

use ink::env::hash::{Blake2x128, Blake2x256, HashOutput, Sha2x256};
use ink::primitives::Hash;

pub trait Builder {
    type OutputType;
    fn add_segment(self, input: &[u8]) -> Self;
    fn build(self) -> Self::OutputType;
}

#[derive(Debug, Default)]
pub struct HashBuilder {
    buffer: [u8; 32],
}
impl Builder for HashBuilder {
    type OutputType = Hash;
    fn add_segment(mut self, input: &[u8]) -> Self {
        let left = &self.buffer[..16];
        let right = &input.as_ref()[16..];
        let mut res = [0u8; 32];
        res[16..].copy_from_slice(left);
        res[..16].copy_from_slice(right);
        self.buffer = res;
        self
    }
    fn build(self) -> Self::OutputType {
        let mut output_hash = <Blake2x256 as HashOutput>::Type::default();
        let input = &self.buffer;
        ink::env::hash_bytes::<Blake2x256>(&self.buffer, &mut output_hash);
        output_hash.into()
    }
}
#[cfg(test)]
mod buffer {
    use ink::primitives::AccountId;

    use crate::{Builder, HashBuilder};

    #[ink::test]
    fn build_hash() {
        let mut hash_builder = HashBuilder::default();
        let input1: AccountId = [0x17; 32].into();
        let input2: AccountId = [0x11; 32].into();
        let res_hash = hash_builder
            .add_segment(input1.as_ref())
            .add_segment(input2.as_ref())
            .build();
        assert_ne!(res_hash, [0u8; 32].into());
        println!("res_hash {:?}", res_hash);
    }
}
