{
  description = "NSFW: Nix Subsystem for Windows - Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            cargo
            rustc

            # Development tools
            cargo-watch
            cargo-edit
            cargo-outdated

            # Build dependencies
            pkg-config
            openssl

            # For JSON parsing testing
            jq

            # Nix tools (for testing Nix operations)
            nix

            # Documentation
            mdbook
          ];

          shellHook = ''
            echo "🚀 NSFW Development Environment"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            echo "Nix: $(nix --version)"
            echo ""
            echo "📚 Commands:"
            echo "  cargo build          - Build the project"
            echo "  cargo test           - Run tests"
            echo "  cargo clippy         - Check for issues"
            echo "  cargo fmt            - Format code"
            echo "  cargo doc --open     - Generate docs"
            echo ""
            echo "📖 See docs/PHASE_1_PLAN.md for roadmap"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

            # Set up Rust backtrace for better errors
            export RUST_BACKTRACE=1
          '';

          # Rust needs these for certain crates
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        };
      }
    );
}