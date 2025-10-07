use board::Position;
use evaluation::negamax::Searcher;
use std::io::{self, BufRead};

pub struct UCIEngine {
    position: Position,
}

impl UCIEngine {
    pub fn new() -> Self {
        Self {
            position: Position::new(),
        }
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut line = String::new();

        loop {
            line.clear();
            if reader.read_line(&mut line).is_err() {
                break;
            }

            let command = line.trim();
            if command.is_empty() {
                continue;
            }

            if !self.handle_command(command) {
                break; // quit command
            }
        }
    }

    fn handle_command(&mut self, command: &str) -> bool {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return true;
        }

        match parts[0] {
            "uci" => self.cmd_uci(),
            "isready" => self.cmd_isready(),
            "ucinewgame" => self.cmd_ucinewgame(),
            "position" => self.cmd_position(&parts[1..]),
            "go" => self.cmd_go(&parts[1..]),
            "quit" => return false,
            "d" | "display" => self.cmd_display(),
            _ => println!("Unknown command: {}", parts[0]),
        }

        true
    }

    fn cmd_uci(&self) {
        println!("id name Oops!Mate");
        println!("id author Wizard");
        println!("uciok");
    }

    fn cmd_isready(&self) {
        println!("readyok");
    }

    fn cmd_ucinewgame(&mut self) {
        self.position = Position::new();
    }

    fn cmd_position(&mut self, parts: &[&str]) {
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "startpos" => {
                self.position = Position::new();
                // Handle moves if present
                if parts.len() > 1 && parts[1] == "moves" {
                    self.apply_moves(&parts[2..]);
                }
            }
            "fen" => {
                let moves_idx = parts.iter().position(|&s| s == "moves");

                let fen_end = moves_idx.unwrap_or(parts.len());
                let fen = parts[1..fen_end].join(" ");

                match Position::from_fen(&fen) {
                    Ok(pos) => {
                        self.position = pos;
                        if let Some(idx) = moves_idx {
                            self.apply_moves(&parts[idx + 1..]);
                        }
                    }
                    Err(e) => println!("Invalid FEN: {}", e),
                }
            }
            _ => println!("Invalid position command"),
        }
    }

    fn apply_moves(&mut self, move_strs: &[&str]) {
        use types::moves::MoveCollector;

        for move_str in move_strs {
            let mut collector = MoveCollector::new();
            self.position.generate_moves(&mut collector);

            // Find matching move (UCI format: e2e4, e7e5, etc.)
            let mut found = false;
            for i in 0..collector.len() {
                let m = collector[i];
                let from = m.from();
                let to = m.to();

                // Convert to algebraic notation
                let from_sq = format!("{}{}", (b'a' + (from % 8) as u8) as char, (from / 8) + 1);
                let to_sq = format!("{}{}", (b'a' + (to % 8) as u8) as char, (to / 8) + 1);

                let move_string = format!("{}{}", from_sq, to_sq);

                // Check for promotion
                let expected_move = if move_str.len() > 4 {
                    format!("{}{}", move_string, &move_str[4..5])
                } else {
                    move_string.clone()
                };

                if *move_str == move_string || *move_str == expected_move {
                    self.position = self.position.make_move(m);
                    found = true;
                    break;
                }
            }

            if !found {
                println!("info string Illegal or invalid move: {}", move_str);
                break;
            }
        }
    }

    fn cmd_go(&mut self, parts: &[&str]) {
        let mut depth: Option<u8> = None;
        let mut movetime: Option<u64> = None;
        let mut wtime: Option<u64> = None;
        let mut btime: Option<u64> = None;
        let mut _winc: u64 = 0;
        let mut _binc: u64 = 0;

        let mut i = 0;
        while i < parts.len() {
            match parts[i] {
                "depth" => {
                    if i + 1 < parts.len() {
                        depth = parts[i + 1].parse().ok();
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
                        _winc = parts[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "binc" => {
                    if i + 1 < parts.len() {
                        _binc = parts[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "infinite" => {
                    depth = Some(10);
                    i += 1;
                }
                _ => i += 1,
            }
        }

        // Calculate depth based on time if not specified
        let search_depth = if let Some(d) = depth {
            d
        } else if let Some(mt) = movetime {
            // Simple heuristic: more time = more depth
            if mt > 10000 {
                6
            } else if mt > 5000 {
                5
            } else if mt > 1000 {
                4
            } else {
                3
            }
        } else {
            // Use remaining time
            use types::others::Color;
            let our_time = match self.position.side_to_move {
                Color::White => wtime.unwrap_or(30000),
                Color::Black => btime.unwrap_or(30000),
            };

            // Simple time management: use ~1/30th of remaining time
            let time_for_move = our_time / 30;

            if time_for_move > 10000 {
                6
            } else if time_for_move > 5000 {
                5
            } else if time_for_move > 1000 {
                4
            } else {
                3
            }
        };

        // Search
        let (best_move, score) = self.position.search(search_depth);

        if let Some(m) = best_move {
            println!("info depth {} score cp {}", search_depth, score as i32);
            println!("bestmove {}", m);
        } else {
            println!("bestmove 0000");
        }
    }

    fn cmd_display(&self) {
        // Debug display of current position
        println!("Current position:");
        println!("{:?}", self.position);
    }
}

// For creating a binary
pub fn main() {
    let mut engine = UCIEngine::new();
    engine.run();
}
