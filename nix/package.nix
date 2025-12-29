{
  rustPlatform,
  lib,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  pname = "wally";
  version = "0.1.0";

  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
  ];

  meta = {
    description = "A wallpaper scraper and downloader";
    homepage = "https://codeberg.org/yum/wally";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [yum];
    platforms = lib.platforms.linux;
  };
}
