{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    devenv.url = "github:cachix/devenv";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      rust-overlay,
      devenv,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            (
              { pkgs, ... }:
              {
                packages = with pkgs; [
                  pkgsStatic.buildPackages.rust-bin.stable.latest.default
                  rust-analyzer
                  taplo
                  sqlx-cli
                  pgformatter
                  schemacrawler
                ];

                services.postgres.enable = true;

                hardeningDisable = [ "fortify" ];

                env = {
                  CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";

                  DATABASE_URL = "postgresql:///stardb";
                  API_KEY = "fd177b2d-a824-4e5f-a73b-ddc796f8fcf6";
                };
              }
            )
          ];
        };

        packages = {
          devenv-up = self.devShells.${system}.default.config.procfileScript;
          devenv-test = self.devShells.${system}.default.config.test;
        };
      }
    );
}
