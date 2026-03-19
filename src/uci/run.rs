use super::UciEngine;
use crate::{
    search::search_with_stop_signal,
    time_control::{calculate_time_allocation, clamp_search_budget},
    Position,
};
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

const UCI_SEARCH_STACK_SIZE: usize = 32 * 1024 * 1024;

impl<E: crate::eval::EvalProvider> UciEngine<E> {
    /// Starts the blocking UCI command loop on standard input and output.
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

            self.reap_finished_search();

            match parts[0] {
                "uci" => self.handle_uci(),
                "isready" => {
                    println!("readyok");
                    let _ = std::io::stdout().flush();
                }
                "setoption" => self.handle_setoption(&parts[1..]),
                "ucinewgame" => self.handle_new_game(),
                "position" => self.handle_position(&parts[1..]),
                "go" => self.handle_go(&parts[1..]),
                "quit" => {
                    self.stop_search_and_wait();
                    break;
                }
                "stop" => self.signal_stop(),
                _ => {}
            }
        }

        self.stop_search_and_wait();
    }

    fn handle_uci(&self) {
        println!("id name OopsMate");
        println!("id author Swoyam P.");
        println!("option name Hash type spin default 64 min 1 max 1024");
        println!("option name Threads type spin default 4 min 1 max 256");
        println!("uciok");
        let _ = std::io::stdout().flush();
    }

    fn handle_setoption(&mut self, parts: &[&str]) {
        if parts.len() < 4 || parts[0] != "name" {
            return;
        }

        let mut name_end = 1;
        while name_end < parts.len() && parts[name_end] != "value" {
            name_end += 1;
        }

        let name = parts[1..name_end].join(" ").to_lowercase();

        if name_end + 1 >= parts.len() {
            return;
        }

        let value = parts[name_end + 1];

        match name.as_str() {
            "hash" => {
                if let Ok(mb) = value.parse::<usize>() {
                    self.tt = std::sync::Arc::new(crate::tpt::TranspositionTable::new_mb(mb));
                }
            }
            "threads" => {
                if let Ok(t) = value.parse::<usize>() {
                    self.threads = t.clamp(1, 256);
                }
            }
            _ => {}
        }
    }

    fn handle_new_game(&mut self) {
        self.stop_search_and_wait();
        self.position = Position::new();
        self.tt.clear();
    }

    fn handle_position(&mut self, parts: &[&str]) {
        self.stop_search_and_wait();

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

        if let Some(idx) = moves_idx {
            for move_str in &parts[idx + 1..] {
                if let Some(m) = Self::parse_move(move_str, &self.position) {
                    self.position.make_move(m);
                } else {
                    eprintln!("Invalid move format: {}", move_str);
                    break;
                }
            }
        }
    }

    fn handle_go(&mut self, parts: &[&str]) {
        self.stop_search_and_wait();

        let mut wtime = None;
        let mut btime = None;
        let mut winc = 0;
        let mut binc = 0;
        let mut movestogo = None;
        let mut depth = 50;
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

        let allocated_time = if infinite {
            None
        } else if let Some(mt) = movetime {
            Some(clamp_search_budget(mt))
        } else if wtime.is_some() || btime.is_some() {
            let our_time = match self.position.side_to_move {
                crate::types::Color::White => wtime.unwrap_or(60000),
                crate::types::Color::Black => btime.unwrap_or(60000),
            };
            let our_inc = match self.position.side_to_move {
                crate::types::Color::White => winc,
                crate::types::Color::Black => binc,
            };
            Some(clamp_search_budget(calculate_time_allocation(
                our_time, our_inc, movestogo,
            )))
        } else {
            None
        };

        let pos = self.position.clone();
        let tt = self.tt.clone();
        let threads = self.threads;
        let eval = self.eval.clone();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let worker_signal = stop_signal.clone();

        let handle = thread::Builder::new()
            .stack_size(UCI_SEARCH_STACK_SIZE)
            .spawn(move || {
                let result = search_with_stop_signal(
                    &pos,
                    depth,
                    allocated_time,
                    tt,
                    threads,
                    worker_signal,
                    eval,
                );

                if let Some(info) = result {
                    println!("bestmove {}", info.best_move.to_uci());
                } else {
                    println!("bestmove 0000");
                }

                let _ = std::io::stdout().flush();
            })
            .expect("failed to spawn UCI search thread");

        self.active_search = Some(super::engine::ActiveSearch {
            stop_signal,
            handle,
        });
    }

    fn signal_stop(&self) {
        if let Some(active) = &self.active_search {
            active.stop_signal.store(true, Ordering::Relaxed);
        }
    }

    fn stop_search_and_wait(&mut self) {
        if let Some(active) = self.active_search.take() {
            active.stop_signal.store(true, Ordering::Relaxed);
            let _ = active.handle.join();
        }
    }

    fn reap_finished_search(&mut self) {
        let is_finished = self
            .active_search
            .as_ref()
            .is_some_and(|active| active.handle.is_finished());

        if is_finished {
            if let Some(active) = self.active_search.take() {
                let _ = active.handle.join();
            }
        }
    }
}
