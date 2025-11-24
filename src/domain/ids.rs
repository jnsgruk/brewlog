use block_id::{Alphabet, BlockId as BlockIdGenerator};
use once_cell::sync::Lazy;
use rand::RngCore;

static ID_GENERATOR: Lazy<BlockIdGenerator<char>> =
    Lazy::new(|| BlockIdGenerator::new(Alphabet::alphanumeric(), 0xB10C_1D_u128, 4));

pub fn generate_id() -> String {
    let mut rng = rand::thread_rng();
    let value = rng.next_u64();
    ID_GENERATOR
        .encode_string(value)
        .expect("block-id encoding should succeed")
}
