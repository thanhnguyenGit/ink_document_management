#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::hash::{Blake2x128, Blake2x256};
use ink::primitives::Hash;

pub trait Builder {
    type OutputType;
    fn add_segment<T: AsRef<[u8]>>(&mut self, input: T);
    fn build(self) -> Self::OutputType;
}

#[derive(Debug)]
pub struct HashBuilder {
    byte_segment: [u8; 32],
}
impl Builder for HashBuilder {
    type OutputType = Hash;
    fn add_segment<T: AsRef<[u8]>>(&mut self, input: T) {
        self.byte_segment = [self.byte_segment, input.into()].concat().as_ref();
    }
    fn build(self) -> Self::OutputType {
        let res = HashBuilder {
            byte_segment: [0x00; 32],
        };
        res.byte_segment.into()
    }
}
