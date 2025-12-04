{
  rustPlatform,
  lib,
}:
rustPlatform.buildRustPackage {
  pname = "ports";
  version = "0.1.0";

  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = "Obliterate processes that are hogging your ports";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "ports";
  };
}
