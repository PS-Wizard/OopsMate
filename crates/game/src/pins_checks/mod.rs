use crate::pins_checks::{
    gen_between_attacks::generate_between, gen_ray_attacks::generate_ray_attacks,
};

mod gen_between_attacks;
mod gen_ray_attacks;
pub mod move_type;
pub mod pin_check_finder;
pub static RAY_ATTACKS: [[u64; 64]; 8] = generate_ray_attacks();
pub static BETWEEN: [[u64; 64]; 64] = generate_between();

pub mod direction_consts {
    pub const TOP: usize = 0;
    pub const TOP_RIGHT: usize = 1;
    pub const RIGHT: usize = 2;
    pub const BOTTOM_RIGHT: usize = 3;
    pub const BOTTOM: usize = 4;
    pub const BOTTOM_LEFT: usize = 5;
    pub const LEFT: usize = 6;
    pub const TOP_LEFT: usize = 7;
}
