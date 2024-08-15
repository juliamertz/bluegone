{
  description = "";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, ... }:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import inputs.rust-overlay) ];
        pkgs = import (inputs.nixpkgs) { inherit system overlays; };

        inherit (inputs.nixpkgs) lib;
        inherit (pkgs) stdenv;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rust-bin.stable.latest.minimal;
          rustc = pkgs.rust-bin.stable.latest.minimal;
        };
        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
          rustPlatform.bindgenHook
        ];
        buildInputs = with pkgs; [ openssl ];
      in {
        packages.default = rustPlatform.buildRustPackage rec {
          inherit buildInputs nativeBuildInputs;

          name = "bluegone";
          src = ./.;
          version = self.shortRev or "dev";

          cargoLock = {
            lockFile = ../Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          meta = {
            description = "";
            homepage = "";
            changelog = "";
            mainProgram = "bluegone";
            # license = lib.licenses.mit;
          };
        };

        devShell = pkgs.mkShell {
          name = "bluegone-shell";
          inherit nativeBuildInputs;

          buildInputs = buildInputs ++ (with pkgs.rust-bin; [
            (stable.latest.minimal.override {
              extensions = [ "clippy" "rust-src" ];
            })

            nightly.latest.clippy
            nightly.latest.rustfmt
            nightly.latest.rust-analyzer
          ]);
        };
      });
}

