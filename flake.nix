{
  description = "UTP TUI Flake";

  inputs = {
    nixpkgs.url      = github:NixOS/nixpkgs/nixos-21.11;
    rust-overlay.url = github:oxalica/rust-overlay;
    flake-utils.url  = github:numtide/flake-utils;
    nur.url          = github:nix-community/NUR;
  };

  # TODO: cargo extensions
  # TODO: wasm things (node, etc.)
  # TODO: CI setup? (garnix)
  # TODO: expose app targets, etc.
  # TODO: gdb-dashboard, gdb
  #   - TODO: expose lldb instead on macOS (or in addition; it has better support for attaching to a running process..)


  # TODO: RUST_BACKTRACE=full
  # TODO: RUST_LOG=...
  # TODO: debug task in vscode that lets us attach!
  outputs = { self, nixpkgs, rust-overlay, flake-utils, nur }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) nur.overlay ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # `gdb` is broken on ARM macOS so we'll fallback to using x86_64 GDB
        # there (assuming Rosetta is installed: https://github.com/NixOS/nix/pull/4310).
        #
        # See: https://github.com/NixOS/nixpkgs/issues/147953
        gdbPkgs = let
          pkgs' = if pkgs.stdenv.isDarwin && pkgs.stdenv.isAarch64 then
            (import nixpkgs { system = "x86_64-darwin"; inherit overlays; })
          else
            pkgs;
        in
          [ pkgs'.gdb pkgs'.nur.repos.mic92.gdb-dashboard ]
        ;
      in
      with pkgs;
      {
        devShells.default = mkShell {
          nativeBuildInputs = [
            pkg-config
          ] ++ gdbPkgs;

          buildInputs = [
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
          ] ++ lib.optionals stdenv.isLinux [
            libudev openssl
          ];
          shellHook = ''
          '';
        };
      }
    );
}
