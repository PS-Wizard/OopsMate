mod gen_between_attacks;
mod gen_ray_attacks;
mod move_generator;
mod move_type;
mod pin_check_finder;

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
