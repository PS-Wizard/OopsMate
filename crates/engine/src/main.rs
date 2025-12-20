use evaluation::evaluation::nnue_eval::init_nnue;
use std::env;
use std::path::PathBuf;
use uci::uci_engine::UCIEngine;

fn main() {
    // Try to get NNUE path from environment variable first
    let nnue_path = match env::var("NNUE_FILE") {
        Ok(path) => {
            // eprintln!("info string Using NNUE_FILE from environment: {}", path);
            path
        }
        Err(_) => {
            eprintln!("info string NNUE_FILE not set, searching for file...");
            find_nnue_file()
        }
    };

    // eprintln!("info string Attempting to load NNUE from: {}", nnue_path);

    // Check if file exists before trying to initialize
    if !PathBuf::from(&nnue_path).exists() {
        eprintln!("info string ERROR: NNUE file not found at: {}", nnue_path);
        eprintln!("info string Set NNUE_FILE environment variable to the full path");
        eprintln!("info string Example: export NNUE_FILE=/path/to/nn-04cf2b4ed1da.nnue");
        eprintln!("info string Continuing with HCE evaluation only");
    } else {
        match init_nnue(&nnue_path) {
            Ok(_) => {
                // eprintln!("info string NNUE initialized successfully");
            }
            Err(e) => {
                eprintln!("info string Failed to initialize NNUE: {}", e);
                eprintln!("info string Continuing with HCE evaluation only");
            }
        }
    }

    // Run the UCI engine
    let mut engine = UCIEngine::new();
    engine.run();
}

/// Find the NNUE evaluation file
fn find_nnue_file() -> String {
    // Get the executable's directory
    let exe_path = env::current_exe().ok();
    let exe_dir = exe_path.as_ref().and_then(|p| p.parent());

    // Try multiple possible locations relative to different starting points
    let mut possible_paths = vec![];

    // Check environment variable first
    if let Ok(path) = env::var("NNUE_FILE") {
        possible_paths.push(path);
    }

    // Relative to current directory
    possible_paths.extend(
        [
            "nn-04cf2b4ed1da.nnue",
            "assets/nn-04cf2b4ed1da.nnue",
            "../assets/nn-04cf2b4ed1da.nnue",
            "../../assets/nn-04cf2b4ed1da.nnue",
            "./nn-04cf2b4ed1da.nnue",
            "crates/engine/nn-04cf2b4ed1da.nnue",
        ]
        .iter()
        .map(|s| s.to_string()),
    );

    // Relative to executable directory
    if let Some(dir) = exe_dir {
        possible_paths.push(
            dir.join("nn-04cf2b4ed1da.nnue")
                .to_string_lossy()
                .to_string(),
        );
        possible_paths.push(
            dir.join("../nn-04cf2b4ed1da.nnue")
                .to_string_lossy()
                .to_string(),
        );
    }

    // Try each path
    for path in &possible_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            eprintln!("info string Found NNUE file at: {}", path);
            return path.clone();
        }
    }

    // If not found, print all locations we checked
    eprintln!("info string NNUE file not found. Searched in:");
    for path in &possible_paths {
        eprintln!("info string   - {}", path);
    }
    eprintln!("info string Current directory: {:?}", env::current_dir());

    // Return first path as default (will fail, but gives clear error)
    "nn-04cf2b4ed1da.nnue".to_string()
}
