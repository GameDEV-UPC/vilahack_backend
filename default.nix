{ rustPlatform
, postgresql
, pkgconf
, openssl
, nix
, bash
}:

let
  package_name = "vilahack_backend";
  version = "0.6.1";
in
  rustPlatform.buildRustPackage {
    name = package_name;
    pname = package_name;
    version = version;

    src = ./.;

    nativeBuildInputs = [ pkgconf ];
    buildInputs = [ postgresql openssl nix bash ];
    cargoLock.lockFile = ./Cargo.lock;
  }
