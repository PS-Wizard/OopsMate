#![allow(dead_code)]

// Maximum moves per category based on chess theory
const MAX_PROMOTIONS: usize = 32; // 8 pawns * 4 promotion types (rare but possible)
const MAX_CAPTURES: usize = 64; // Generous estimate for captures
const MAX_QUIET: usize = 160; // Remaining moves are quiet
const MAX_TOTAL: usize = MAX_PROMOTIONS + MAX_CAPTURES + MAX_QUIET; // 256

pub struct MoveList {
    promotions: [(u8, u8, u8); MAX_PROMOTIONS], // Highest priority
    captures: [(u8, u8, u8); MAX_CAPTURES],     // Medium priority
    quiet: [(u8, u8, u8); MAX_QUIET],           // Lowest priority

    promo_count: usize,
    capture_count: usize,
    quiet_count: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            promotions: [(0, 0, 0); MAX_PROMOTIONS],
            captures: [(0, 0, 0); MAX_CAPTURES],
            quiet: [(0, 0, 0); MAX_QUIET],
            promo_count: 0,
            capture_count: 0,
            quiet_count: 0,
        }
    }

    #[inline(always)]
    pub fn push_promotion(&mut self, from: u8, to: u8, flags: u8) {
        self.promotions[self.promo_count] = (from, to, flags);
        self.promo_count += 1;
    }

    #[inline(always)]
    pub fn push_capture(&mut self, from: u8, to: u8, flags: u8) {
        self.captures[self.capture_count] = (from, to, flags);
        self.capture_count += 1;
    }

    #[inline(always)]
    pub fn push_quiet(&mut self, from: u8, to: u8, flags: u8) {
        self.quiet[self.quiet_count] = (from, to, flags);
        self.quiet_count += 1;
    }

    #[inline(always)]
    pub fn get_move(&self, index: usize) -> (u8, u8, u8) {
        if index < self.promo_count {
            self.promotions[index]
        } else if index < self.promo_count + self.capture_count {
            self.captures[index - self.promo_count]
        } else {
            self.quiet[index - self.promo_count - self.capture_count]
        }
    }

    #[inline(always)]
    pub fn total_count(&self) -> usize {
        self.promo_count + self.capture_count + self.quiet_count
    }

    // Individual counts for debugging/stats
    #[inline(always)]
    pub fn promotion_count(&self) -> usize {
        self.promo_count
    }

    #[inline(always)]
    pub fn capture_count(&self) -> usize {
        self.capture_count
    }

    #[inline(always)]
    pub fn quiet_count(&self) -> usize {
        self.quiet_count
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.promo_count = 0;
        self.capture_count = 0;
        self.quiet_count = 0;
    }

    #[inline(always)]
    pub fn print(&self) {
        println!("=== PROMOTIONS ({}) ===", self.promo_count);
        for i in 0..self.promo_count {
            let (from, to, flags) = self.promotions[i];
            println!("  Promotion {}: from={} to={} flags={}", i, from, to, flags);
        }

        println!("=== CAPTURES ({}) ===", self.capture_count);
        for i in 0..self.capture_count {
            let (from, to, flags) = self.captures[i];
            println!("  Capture {}: from={} to={} flags={}", i, from, to, flags);
        }

        println!("=== QUIET MOVES ({}) ===", self.quiet_count);
        for i in 0..self.quiet_count {
            let (from, to, flags) = self.quiet[i];
            println!("  Quiet {}: from={} to={} flags={}", i, from, to, flags);
        }

        println!("Total moves: {}", self.total_count());
    }
}
