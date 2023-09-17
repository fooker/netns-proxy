{ rustPlatform, lib, ... }:

rustPlatform.buildRustPackage rec {
  name = "netns-proxy";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
}
