{
  description = "Development shell for oops_mate";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
          "clippy"
          "rustfmt"
        ];
      };
    in {
      devShells.default = pkgs.mkShell {
        packages = [
          rustToolchain
          pkgs.pkg-config
        ];

        RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
      };
    });
}
