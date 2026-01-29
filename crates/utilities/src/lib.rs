pub mod algebraic;
pub mod board;

#[cfg(test)]
#[cfg(debug_assertions)]
mod test {
    use crate::{algebraic::Algebraic, board::PrintAsBoard};
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
