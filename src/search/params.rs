// Represents an "infinity" , just something big that represents an infinitely good / bad in the
// search algorithm
pub const INFINITY: i32 = 50_000;
pub const MATE_VALUE: i32 = 49_000;

// Max number of legal moves expected in any position
pub const MAX_MOVES: usize = 256;

// Max depth search will reach
pub const MAX_DEPTH: usize = 128;

// Min depth at which aspiration windows are used
pub const ASPIRATION_DEPTH: u8 = 8;

// Minimum depth to trigger IID when no hash move is found
pub const IID_MIN_DEPTH: u8 = 4;
