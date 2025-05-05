{
  lib,
  stdenv,
  openssl,
  pkg-config,
  libiconv,
  craneLibBuild,
  cargoArtifacts,
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

  nativeBuildInputs = [ pkg-config ];
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
