{ rustPlatform
, postgresql   
, pkgconf
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "backend";
  version = "0.4.2";

  src = ./.;

  nativeBuildInputs = [ pkgconf ];
  buildInputs = [ postgresql openssl ];
  cargoLock.lockFile = ./Cargo.lock;
}
