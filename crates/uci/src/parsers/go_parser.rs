use crate::time::TimeControl;

/// Struct to prase the go command
pub struct GoParser;

impl GoParser {
    /// Take in parts of the UCI command and parses the one for the "go"
    pub fn parse(parts: &[&str]) -> TimeControl {
        let mut tc = TimeControl::new();
        let mut i = 0;

        while i < parts.len() {
            match parts[i] {
                "depth" => {
                    if let Some(val) = Self::parse_next_u8(parts, i) {
                        tc.depth = Some(val);
                    }
                    i += 2;
                }
                "movetime" => {
                    if let Some(val) = Self::parse_next_u64(parts, i) {
                        tc.movetime = Some(val);
                    }
                    i += 2;
                }
                "wtime" => {
                    if let Some(val) = Self::parse_next_u64(parts, i) {
                        tc.wtime = Some(val);
                    }
                    i += 2;
                }
                "btime" => {
                    if let Some(val) = Self::parse_next_u64(parts, i) {
                        tc.btime = Some(val);
                    }
                    i += 2;
                }
                "winc" => {
                    if let Some(val) = Self::parse_next_u64(parts, i) {
                        tc.winc = val;
                    }
                    i += 2;
                }
                "binc" => {
                    if let Some(val) = Self::parse_next_u64(parts, i) {
                        tc.binc = val;
                    }
                    i += 2;
                }
                "movestogo" => {
                    if let Some(val) = Self::parse_next_u32(parts, i) {
                        tc.movestogo = Some(val);
                    }
                    i += 2;
                }
                "infinite" => {
                    tc.infinite = true;
                    i += 1;
                }
                _ => i += 1,
            }
        }

        tc
    }
    
    fn parse_next_u8(parts: &[&str], i: usize) -> Option<u8> {
        parts.get(i + 1).and_then(|s| s.parse().ok())
    }

    fn parse_next_u32(parts: &[&str], i: usize) -> Option<u32> {
        parts.get(i + 1).and_then(|s| s.parse().ok())
    }

    fn parse_next_u64(parts: &[&str], i: usize) -> Option<u64> {
        parts.get(i + 1).and_then(|s| s.parse().ok())
    }
}
