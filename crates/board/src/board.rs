#![allow(dead_code)]

use std::{ops::BitOr};

#[derive(Clone, Copy)]
pub struct Board(pub u64);

impl Board {
    pub fn set_bit(&mut self, pos: usize) -> Result<(), &str> {
        if pos >= 64 {
            return Err("Idx Greater Than 64");
        }

        self.0 |= 1u64 << pos;
        Ok(())
    }

    pub fn has_bit(&self, pos: usize) -> Option<bool> {
        if pos >= 64 {
            None
        } else {
            Some((self.0 & (1 << pos)) != 0)
        }
    }

    pub fn remove_bit(&mut self, pos: usize) -> Result<(), &str> {
        if pos >= 64 {
            return Err("Index out of bounds");
        }
        self.0 &= !(1u64 << pos);
        Ok(())
    }

    pub fn print_board(&self) {
        for rank in (0..8).rev() {
            // ranks 8..1
            print!("{} ", rank + 1);
            for file in 0..8 {
                // files a..h
                let sq = rank * 8 + file;
                if (self.0 >> sq) & 1 != 0 {
                    print!("X ");
                } else {
                    print!(". ");
                }
            }
            println!();
        }
        println!("  a b c d e f g h");
    }
}

impl BitOr for Board {
    type Output = Board;

    fn bitor(self, rhs: Self) -> Self::Output {
        Board(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod board_test {
    use super::*;

    #[test]
    fn test_set_and_has_bit() {
        let mut b = Board(0);
        assert_eq!(b.has_bit(0), Some(false));
        b.set_bit(0).unwrap();
        assert_eq!(b.has_bit(0), Some(true));

        b.set_bit(63).unwrap();
        assert_eq!(b.has_bit(63), Some(true));

        assert!(b.set_bit(64).is_err()); // out-of-bounds
    }

    #[test]
    fn test_remove_bit() {
        let mut b = Board(0);
        b.set_bit(10).unwrap();
        assert_eq!(b.has_bit(10), Some(true));

        b.remove_bit(10).unwrap();
        assert_eq!(b.has_bit(10), Some(false));

        assert!(b.remove_bit(64).is_err()); // out-of-bounds
    }

    #[test]
    fn test_bit_or() {
        let mut b1 = Board(0);
        let mut b2 = Board(0);

        b1.set_bit(0).unwrap();
        b2.set_bit(1).unwrap();

        let combined = b1 | b2;
        combined.print_board();
        assert_eq!(combined.has_bit(0), Some(true));
        assert_eq!(combined.has_bit(1), Some(true));
        assert_eq!(combined.has_bit(2), Some(false));
    }
}
