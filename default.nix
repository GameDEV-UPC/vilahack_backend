{ rustPlatform
, postgresql   
, pkgconf
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "backend";
  version = "0.1.4";

  src = ./.;

  nativeBuildInputs = [ pkgconf ];
  buildInputs = [ postgresql, openssl ];
  cargoLock.lockFile = ./Cargo.lock;
}
