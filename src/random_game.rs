#![allow(dead_code)]
use crate::{Color, Position};

fn rand_game() {
    println!("Playing a random game...\n");

    let mut pos = Position::new();
    let mut move_count = 0;

    loop {
        // Generate and display current position stats
        let is_check = pos.is_in_check();

        println!(
            "Move {}: {} to move{}",
            pos.fullmove,
            if pos.side_to_move == Color::White {
                "White"
            } else {
                "Black"
            },
            if is_check { " (in check)" } else { "" }
        );

        // Try to make a random move
        match pos.random_move() {
            Some(m) => {
                println!(
                    "  Playing: {}{}",
                    square_name(m.from()),
                    square_name(m.to())
                );
                pos = pos.make_move(m);
                move_count += 1;
            }
            None => {
                // No legal moves
                if is_check {
                    println!(
                        "\nCheckmate! {} wins!",
                        if pos.side_to_move == Color::White {
                            "Black"
                        } else {
                            "White"
                        }
                    );
                } else {
                    println!("\nStalemate!");
                }
                break;
            }
        }

        if move_count >= 50 {
            println!("\nGame ended after 100 moves (draw by move limit)");
            break;
        }
    }

    println!("\nTotal moves played: {}", move_count);
}

fn square_name(sq: usize) -> String {
    let file = (b'a' + (sq % 8) as u8) as char;
    let rank = (sq / 8) + 1;
    format!("{}{}", file, rank)
}

#[cfg(test)]
mod random_game_test {
    use crate::random_game::rand_game;

    #[test]
    fn test_random_game() {
        rand_game();
    }
}
