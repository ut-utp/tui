{
  description = "UTP TUI Flake";

  inputs = {
    nixpkgs.url      = github:NixOS/nixpkgs/nixos-21.11;
    rust-overlay.url = github:oxalica/rust-overlay;
    flake-utils.url  = github:numtide/flake-utils;
  };

  # TODO: cargo extensions
  # TODO: wasm things (node, etc.)
  # TODO: CI setup? (garnix)
  # TODO: expose app targets, etc.

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
          ] ++ lib.optionals stdenv.isLinux [
            libudev pkg-config openssl
          ];
          shellHook = ''
          '';
        };
      }
    );
}
