use crate::{
    search::search, time_control::calculate_time_allocation, tpt::TranspositionTable, Move,
    Position,
};
use std::io::{self, BufRead, Write};

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
                "isready" => {
                    println!("readyok");
                    let _ = std::io::stdout().flush();
                }
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
        let _ = std::io::stdout().flush();
    }

    fn handle_new_game(&mut self) {
        self.position = Position::new();
        self.tt.clear();
    }

    fn handle_position(&mut self, parts: &[&str]) {
        if parts.is_empty() {
            return;
        }

        let mut moves_idx = None;

        if parts[0] == "startpos" {
            self.position = Position::new();
            moves_idx = parts.iter().position(|&s| s == "moves");
        } else if parts[0] == "fen" {
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

        // Apply moves directly without validation
        // The GUI/engine sending the moves is responsible for legality
        if let Some(idx) = moves_idx {
            for move_str in &parts[idx + 1..] {
                if let Some(m) = Self::parse_move_fast(move_str, &self.position) {
                    self.position = self.position.make_move(&m);
                } else {
                    eprintln!("Invalid move format: {}", move_str);
                    break;
                }
            }
        }
    }

    /// Fast move parsing without legal move generation
    /// Assumes the move is legal (as GUIs should only send legal moves)
    fn parse_move_fast(move_str: &str, pos: &Position) -> Option<Move> {
        if move_str.len() < 4 {
            return None;
        }

        let from = Self::parse_square(&move_str[0..2])?;
        let to = Self::parse_square(&move_str[2..4])?;

        let (piece, _color) = pos.piece_at(from)?;

        // Determine move type based on position and move
        use crate::types::{MoveType, Piece};

        let is_capture = pos.piece_at(to).is_some();

        // Handle promotions
        if move_str.len() == 5 {
            let promo = move_str.chars().nth(4)?;
            let move_type = match (promo, is_capture) {
                ('q', false) => MoveType::PromotionQueen,
                ('r', false) => MoveType::PromotionRook,
                ('b', false) => MoveType::PromotionBishop,
                ('n', false) => MoveType::PromotionKnight,
                ('q', true) => MoveType::CapturePromotionQueen,
                ('r', true) => MoveType::CapturePromotionRook,
                ('b', true) => MoveType::CapturePromotionBishop,
                ('n', true) => MoveType::CapturePromotionKnight,
                _ => return None,
            };
            return Some(Move::new(from, to, move_type));
        }

        // Check for castling
        if piece == Piece::King && ((from as i32 - to as i32).abs() == 2) {
            return Some(Move::new(from, to, MoveType::Castle));
        }

        // Check for en passant
        if piece == Piece::Pawn {
            if let Some(ep_sq) = pos.en_passant {
                if to == ep_sq as usize && !is_capture {
                    return Some(Move::new(from, to, MoveType::EnPassant));
                }
            }

            // Check for double push
            if (from as i32 - to as i32).abs() == 16 {
                return Some(Move::new(from, to, MoveType::DoublePush));
            }
        }

        // Regular move or capture
        let move_type = if is_capture {
            MoveType::Capture
        } else {
            MoveType::Quiet
        };

        Some(Move::new(from, to, move_type))
    }

    fn parse_square(s: &str) -> Option<usize> {
        if s.len() != 2 {
            return None;
        }

        let file = (s.as_bytes()[0] as i32 - b'a' as i32) as usize;
        let rank = (s.as_bytes()[1] as i32 - b'1' as i32) as usize;

        if file > 7 || rank > 7 {
            return None;
        }

        Some(rank * 8 + file)
    }

    fn handle_go(&mut self, parts: &[&str]) {
        let mut wtime = None;
        let mut btime = None;
        let mut winc = 0;
        let mut binc = 0;
        let mut movestogo = None;
        let mut depth = 50; // Default max depth (will be limited by time)
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
                        depth = parts[i + 1].parse().unwrap_or(50);
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
        let allocated_time = if infinite {
            None
        } else if let Some(mt) = movetime {
            Some(mt)
        } else {
            let our_time = match self.position.side_to_move {
                crate::types::Color::White => wtime.unwrap_or(60000),
                crate::types::Color::Black => btime.unwrap_or(60000),
            };
            let our_inc = match self.position.side_to_move {
                crate::types::Color::White => winc,
                crate::types::Color::Black => binc,
            };

            Some(calculate_time_allocation(our_time, our_inc, movestogo))
        };

        // Search with iterative deepening
        if let Some(info) = search(&self.position, depth, allocated_time, &mut self.tt) {
            println!("bestmove {}", Self::move_to_uci(&info.best_move));
        } else {
            println!("bestmove 0000");
        }

        // Flush stdout to ensure CuteChess receives the bestmove immediately
        let _ = std::io::stdout().flush();
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
