{
  mkShell,
  rust-analyzer,
  rustfmt,
  clippy,
  cargo,
  rustc,
  rustPlatform,
  openssl,
  pkg-config,
}:
mkShell {
  nativeBuildInputs = [
    cargo
    rustc
    rustfmt
    rust-analyzer
    clippy
    pkg-config
  ];

  buildInputs = [
    openssl
  ];

  env.RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
