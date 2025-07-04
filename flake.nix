{
  description = "Rust development flake";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils = { url = "github:numtide/flake-utils"; };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            rust-analyzer
            openssl
            pkg-config
            cargo
            just
	    zellij
            binaryen
          ];
        };
      }
    );
}
