{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs @ {
    nixpkgs,
    flake-parts,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];

      perSystem = {
        pkgs,
        system,
        config,
        ...
      }: {
        _module.args.pkgs = import nixpkgs {
          inherit system;
          overlays = [inputs.rust-overlay.overlays.default];
        };

        packages.toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;

        devShells.default = with pkgs;
          mkShell {
            packages = [
              config.packages.toolchain
              cargo-insta
              just
              wasm-pack
              cargo-generate
              nodejs
              wasm-bindgen-cli
              binaryen
            ];

            # RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
          };
      };
    };
}
