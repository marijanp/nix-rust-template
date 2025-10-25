{
  lib,
  stdenv,
  openssl,
  pkg-config,
  libiconv,
  craneLibBuild,
  cargoArtifacts,
  lld
}:
craneLibBuild {
  pname = "server";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../server
    ];
  };

  inherit cargoArtifacts;

  env = lib.optionalAttrs stdenv.isLinux {
    RUSTFLAGS = "-C link-self-contained=-linker";
  };

  nativeBuildInputs = [ pkg-config lld ];
  buildInputs =
    [
      openssl.dev
    ]
    ++ lib.optionals stdenv.isDarwin [
      libiconv
    ];

  # the coverage report will run the tests
  doCheck = false;
}
