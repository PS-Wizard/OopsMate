// Pseudo Random Number Generator: https://rosettacode.org/wiki/Pseudo-random_numbers/Splitmix64
const fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

const fn gen_zobrist_array<const N: usize>(seed: u64) -> [u64; N] {
    let mut arr = [0u64; N];
    let mut state = seed;
    let mut i = 0;
    while i < N {
        state = splitmix64(state);
        arr[i] = state;
        i += 1;
    }
    arr
}

// Pieces: [Color][Piece][Square]
// 2 colors * 6 pieces * 64 squares = 768
pub const PIECE_KEYS: [[[u64; 64]; 6]; 2] = {
    let flat = gen_zobrist_array::<768>(0x123456789ABCDEF0);
    let mut keys = [[[0u64; 64]; 6]; 2];
    let mut i = 0;
    while i < 2 {
        let mut j = 0;
        while j < 6 {
            let mut k = 0;
            while k < 64 {
                keys[i][j][k] = flat[i * 384 + j * 64 + k];
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
    keys
};

pub const CASTLE_KEYS: [u64; 16] = gen_zobrist_array(0xFEDCBA9876543210);
pub const EP_KEYS: [u64; 8] = gen_zobrist_array(0x1111111111111111);
pub const SIDE_KEY: u64 = 0x1234567890ABCDEF;

