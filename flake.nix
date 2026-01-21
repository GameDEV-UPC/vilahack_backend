{
  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url  = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
            cargo = rustVersion;
            rustc = rustVersion;
        };

        rustBuild = pkgs.callPackage ./default.nix {};
      in {
        defaultPackage = rustBuild;

        devShells.default = with pkgs; mkShell {
          buildInputs = [
            diesel-cli
            diesel-cli-ext
            
            postgresql
            pkgconf
            openssl
            
            [ (rustVersion.override { extensions = [ "rust-src" "rust-analyzer" ]; }) ]
          ];
        };
      }
    );
}
