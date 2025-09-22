pub mod algebraic;
pub mod bits;
pub mod board;

#[cfg(test)]
mod test {
    use crate::{algebraic::Algebraic, bits::EnumerateVariations, board::PrintAsBoard};

    #[test]
    fn test_enumeration() {
        let ens = 0b1010_u64.enumerate();
        let expected = vec![0b0000, 0b0010, 0b1000, 0b1010];
        assert_eq!(ens, expected);
    }

    #[test]
    fn test_place_bits() {
        let blockers = "g2,g4,g6".place();
        blockers.print();
    }

    #[test]
    fn test_notation_to_idx() {
        let blockers = "g2".idx();
        assert_eq!(blockers, 14);
    }
}
