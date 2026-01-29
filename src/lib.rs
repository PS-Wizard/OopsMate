pub mod movegen;
pub mod play;
pub mod position;
pub mod types;
mod random_game;

pub use position::Position;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position() {
        let pos = Position::new();
        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);
        assert_eq!(collector.len(), 20);
    }

    #[test]
    fn test_perft_depth_1() {
        let pos = Position::new();
        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);
        assert_eq!(collector.len(), 20);
    }

    #[test]
    fn test_perft_depth_2() {
        let pos = Position::new();
        let mut total = 0;
        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);

        for i in 0..collector.len() {
            let m = collector.get(i);
            let new_pos = pos.make_move(m);
            let mut collector2 = MoveCollector::new();
            new_pos.generate_moves(&mut collector2);
            total += collector2.len();
        }

        assert_eq!(total, 400);
    }

    #[test]
    fn test_kiwipete_depth_1() {
        let pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);
        assert_eq!(collector.len(), 48);
    }

    #[test]
    fn test_make_move() {
        let pos = Position::new();
        let mut collector = MoveCollector::new();
        pos.generate_moves(&mut collector);

        // Make e2e4
        let m = collector.get(0);
        let new_pos = pos.make_move(m);

        // assert_ne!(pos, new_pos);
        assert_eq!(new_pos.side_to_move, Color::Black);
    }
}
