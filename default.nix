{ rustPlatform
, postgresql   
, pkgconf
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "vilahack_backend";
  version = "0.1.0";

  src = ./.;

  nativeBuildInputs = [ pkgconf ];
  buildInputs = [ postgresql, openssl ];
  cargoLock.lockFile = ./Cargo.lock;
}
