with import <nixpkgs> {};
  mkShell {
    packages = [
      cargo
      rustc
      rustfmt
      rust-analyzer-unwrapped
      clippy
      cargo-insta
      just
    ];

    RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
  }
