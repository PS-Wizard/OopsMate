use board::Position;
use evaluation::search::negamax::Searcher;
use std::io::{self, BufRead};
use types::others::Color;

use crate::parsers::{go_parser::GoParser, move_parser::MoveParser};

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
                break;
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
                if parts.len() > 1 && parts[1] == "moves" {
                    if let Err(e) = MoveParser::apply_moves(&mut self.position, &parts[2..]) {
                        println!("info string {}", e);
                    }
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
                            if let Err(e) =
                                MoveParser::apply_moves(&mut self.position, &parts[idx + 1..])
                            {
                                println!("info string {}", e);
                            }
                        }
                    }
                    Err(e) => println!("info string Invalid FEN: {}", e),
                }
            }
            _ => println!("info string Invalid position command"),
        }
    }

    // TODO: Iterative Deepening Here
    fn cmd_go(&mut self, parts: &[&str]) {
        let time_control = GoParser::parse(parts);

        let is_white = matches!(self.position.side_to_move, Color::White);
        let search_depth = time_control.calculate_depth(is_white);

        let (best_move, score) = self.position.search(search_depth);

        if let Some(m) = best_move {
            println!("info depth {} score cp {}", search_depth, score as i32);
            println!("bestmove {}", m);
        } else {
            println!("bestmove 0000");
        }
    }

    fn cmd_display(&self) {
        println!("Current position:");
        println!("{:?}", self.position);
    }
}
