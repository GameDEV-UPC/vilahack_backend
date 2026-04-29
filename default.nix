{ rustPlatform
, postgresql
, pkgconf
, openssl
}:

let
  package_name = "vilahack_backend";
  version = "0.5.1";
in
  rustPlatform.buildRustPackage {
    name = package_name;
    pname = package_name;
    version = version;

    src = ./.;

    nativeBuildInputs = [ pkgconf ];
    buildInputs = [ postgresql openssl ];
    cargoLock.lockFile = ./Cargo.lock;
  }
