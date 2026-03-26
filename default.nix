{ rustPlatform
, postgresql   
, pkgconf
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "backend";
  version = "0.3.8";

  src = ./.;

  nativeBuildInputs = [ pkgconf ];
  buildInputs = [ postgresql openssl ];
  cargoLock.lockFile = ./Cargo.lock;
}
