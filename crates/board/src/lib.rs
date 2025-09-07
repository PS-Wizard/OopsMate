mod board;
mod game;
mod piece;
mod utils;

#[cfg(test)]
mod tests {
    use crate::game::Game;

    #[test]
    fn board_state_test() {
        let game = Game::new();
        game.print_board(); 
    }
}
