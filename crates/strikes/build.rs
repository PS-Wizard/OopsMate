mod buildgen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    buildgen::generate()
}
