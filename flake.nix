{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in {
        devShells.default = with pkgs;
          mkShell {
            buildInputs = [
              pkgsStatic.buildPackages.rust-bin.stable.latest.default
              rust-analyzer
              taplo
              sqlx-cli
              pgformatter
              schemacrawler
            ];

            hardeningDisable = ["fortify"];

            CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          };
      }
    );
}
