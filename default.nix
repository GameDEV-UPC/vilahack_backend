{ rustPlatform
, postgresql   
, pkgconf
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "vilahack_backend";
  version = "0.1.1";

  src = ./.;

  nativeBuildInputs = [ pkgconf ];
  buildInputs = [ postgresql, openssl ];
  cargoLock.lockFile = ./Cargo.lock;
}
