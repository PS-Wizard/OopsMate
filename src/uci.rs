use crate::{
    search::search,
    time_control::{calculate_time_allocation, TimeControl},
    tpt::TranspositionTable,
    Move, Position,
};
use std::io::{self, BufRead};

pub struct UciEngine {
    position: Position,
    tt: TranspositionTable,
}

impl UciEngine {
    pub fn new() -> Self {
        UciEngine {
            position: Position::new(),
            tt: TranspositionTable::new_mb(64),
        }
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();

        while let Some(Ok(line)) = lines.next() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "uci" => self.handle_uci(),
                "isready" => println!("readyok"),
                "ucinewgame" => self.handle_new_game(),
                "position" => self.handle_position(&parts[1..]),
                "go" => self.handle_go(&parts[1..]),
                "quit" => break,
                "stop" => {} // TODO: Implement search stopping
                _ => {}
            }
        }
    }

    fn handle_uci(&self) {
        println!("id name OopsMate");
        println!("id author Swoyam P.");
        println!("option name Hash type spin default 64 min 1 max 1024");
        println!("uciok");
    }

    fn handle_new_game(&mut self) {
        self.position = Position::new();
        self.tt.clear();
    }

    fn handle_position(&mut self, parts: &[&str]) {
        if parts.is_empty() {
            return;
        }

        // position startpos moves e2e4 e7e5 ...
        // position fen <fen> moves ...

        let mut moves_idx = None;

        if parts[0] == "startpos" {
            self.position = Position::new();
            moves_idx = parts.iter().position(|&s| s == "moves");
        } else if parts[0] == "fen" {
            // Collect FEN parts (should be 6 parts after "fen")
            let fen_parts: Vec<&str> = parts
                .iter()
                .skip(1)
                .take_while(|&&s| s != "moves")
                .copied()
                .collect();

            let fen = fen_parts.join(" ");

            match Position::from_fen(&fen) {
                Ok(pos) => self.position = pos,
                Err(e) => {
                    eprintln!("Invalid FEN: {}", e);
                    return;
                }
            }

            moves_idx = parts.iter().position(|&s| s == "moves");
        }

        // Apply moves if any
        if let Some(idx) = moves_idx {
            for move_str in &parts[idx + 1..] {
                if let Some(m) = self.parse_move(move_str) {
                    self.position = self.position.make_move(&m);
                } else {
                    eprintln!("Invalid move: {}", move_str);
                    break;
                }
            }
        }
    }

    fn parse_move(&self, move_str: &str) -> Option<Move> {
        if move_str.len() < 4 {
            return None;
        }

        let from = Self::parse_square(&move_str[0..2])?;
        let to = Self::parse_square(&move_str[2..4])?;

        // Generate legal moves and find matching move
        use crate::types::MoveCollector;
        let mut collector = MoveCollector::new();
        self.position.generate_moves(&mut collector);

        for mv in collector.as_slice() {
            if mv.from() == from && mv.to() == to {
                // Handle promotions
                if move_str.len() == 5 {
                    let promo = move_str.chars().nth(4)?;
                    let is_capture = mv.is_capture();

                    let expected_type = match (promo, is_capture) {
                        ('q', false) => crate::types::MoveType::PromotionQueen,
                        ('r', false) => crate::types::MoveType::PromotionRook,
                        ('b', false) => crate::types::MoveType::PromotionBishop,
                        ('n', false) => crate::types::MoveType::PromotionKnight,
                        ('q', true) => crate::types::MoveType::CapturePromotionQueen,
                        ('r', true) => crate::types::MoveType::CapturePromotionRook,
                        ('b', true) => crate::types::MoveType::CapturePromotionBishop,
                        ('n', true) => crate::types::MoveType::CapturePromotionKnight,
                        _ => return None,
                    };

                    if mv.move_type() == expected_type {
                        return Some(*mv);
                    }
                } else {
                    return Some(*mv);
                }
            }
        }

        None
    }

    fn parse_square(s: &str) -> Option<usize> {
        if s.len() != 2 {
            return None;
        }

        let file = (s.as_bytes()[0] as char).to_digit(18)? - 10; // a=0, b=1, ...
        let rank = (s.as_bytes()[1] as char).to_digit(10)?;

        Some((rank as usize) * 8 + file as usize)
    }

    fn handle_go(&mut self, parts: &[&str]) {
        // Parse go parameters
        let mut wtime = None;
        let mut btime = None;
        let mut winc = 0;
        let mut binc = 0;
        let mut movestogo = None;
        let mut depth = 10; // Default depth
        let mut infinite = false;
        let mut movetime = None;

        let mut i = 0;
        while i < parts.len() {
            match parts[i] {
                "wtime" => {
                    if i + 1 < parts.len() {
                        wtime = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "btime" => {
                    if i + 1 < parts.len() {
                        btime = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "winc" => {
                    if i + 1 < parts.len() {
                        winc = parts[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "binc" => {
                    if i + 1 < parts.len() {
                        binc = parts[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "movestogo" => {
                    if i + 1 < parts.len() {
                        movestogo = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "depth" => {
                    if i + 1 < parts.len() {
                        depth = parts[i + 1].parse().unwrap_or(10);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "movetime" => {
                    if i + 1 < parts.len() {
                        movetime = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "infinite" => {
                    infinite = true;
                    i += 1;
                }
                _ => i += 1,
            }
        }

        // Calculate time allocation
        let _time_control = if infinite {
            TimeControl::infinite()
        } else if let Some(mt) = movetime {
            TimeControl::new(mt)
        } else {
            let our_time = match self.position.side_to_move {
                crate::types::Color::White => wtime.unwrap_or(60000),
                crate::types::Color::Black => btime.unwrap_or(60000),
            };
            let our_inc = match self.position.side_to_move {
                crate::types::Color::White => winc,
                crate::types::Color::Black => binc,
            };

            let allocated = calculate_time_allocation(our_time, our_inc, movestogo);
            TimeControl::new(allocated)
        };

        // Search for best move
        if let Some(best_move) = search(&self.position, depth, &mut self.tt) {
            println!("bestmove {}", Self::move_to_uci(&best_move));
        } else {
            println!("bestmove 0000");
        }
    }

    fn move_to_uci(m: &Move) -> String {
        let from = m.from();
        let to = m.to();

        let from_sq = format!(
            "{}{}",
            (b'a' + (from % 8) as u8) as char,
            (b'1' + (from / 8) as u8) as char
        );
        let to_sq = format!(
            "{}{}",
            (b'a' + (to % 8) as u8) as char,
            (b'1' + (to / 8) as u8) as char
        );

        // Add promotion piece if needed
        if m.is_promotion() {
            let promo = match m.move_type() {
                crate::types::MoveType::PromotionQueen
                | crate::types::MoveType::CapturePromotionQueen => 'q',
                crate::types::MoveType::PromotionRook
                | crate::types::MoveType::CapturePromotionRook => 'r',
                crate::types::MoveType::PromotionBishop
                | crate::types::MoveType::CapturePromotionBishop => 'b',
                crate::types::MoveType::PromotionKnight
                | crate::types::MoveType::CapturePromotionKnight => 'n',
                _ => unreachable!(),
            };
            format!("{}{}{}", from_sq, to_sq, promo)
        } else {
            format!("{}{}", from_sq, to_sq)
        }
    }
}
