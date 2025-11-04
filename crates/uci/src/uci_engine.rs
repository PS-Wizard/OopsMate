use board::Position;
use evaluation::search::iterative_deepening::IterativeSearcher;
use std::io::{self, BufRead};
use types::others::Color;

use crate::parsers::{go_parser::GoParser, move_parser::MoveParser};

/// Struct to handle UCI communication
pub struct UCIEngine {
    position: Position,
}

impl UCIEngine {
    /// Returns a new UCIEngine with default values
    pub fn new() -> Self {
        Self {
            position: Position::new(),
        }
    }

    /// Handles the UCI Loop
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

    /// Handles parsing the UCI command
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

    /// Responds to the initial "uci" command
    fn cmd_uci(&self) {
        println!("id name Oops!Mate");
        println!("id author Wizard");
        println!("uciok");
    }

    /// Responds to the initial "isready" command
    fn cmd_isready(&self) {
        println!("readyok");
    }

    /// Creates a new default Position as a result of the `ucinewgame` command
    fn cmd_ucinewgame(&mut self) {
        self.position = Position::new();
    }

    /// Handles parsing custom fen position
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

    /// Handles search with iterative deepening
    fn cmd_go(&mut self, parts: &[&str]) {
        let time_control = GoParser::parse(parts);
        let is_white = matches!(self.position.side_to_move, Color::White);

        // Convert time control to search limits
        let limits = time_control.to_search_limits(is_white);

        // Run iterative deepening search
        let result = self.position.search_iterative(limits);

        if let Some(m) = result.best_move {
            println!("bestmove {}", m);
        } else {
            println!("bestmove 0000");
        }
    }

    /// Handles the "d" or "display" command
    fn cmd_display(&self) {
        println!("Current position:");
        println!("{:?}", self.position);
    }
}
