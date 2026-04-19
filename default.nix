{ rustPlatform
, postgresql   
, pkgconf
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "backend";
  version = "0.5.0";

  src = ./.;

  nativeBuildInputs = [ pkgconf ];
  buildInputs = [ postgresql openssl ];
  cargoLock.lockFile = ./Cargo.lock;
}
