{
  description = "nix-rust-template";

  nixConfig = {
    extra-substituters = [
      "https://crane.cachix.org"
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "crane.cachix.org-1:8Scfpmn9w+hGdXH/Q9tTLiYAE/2dnJYRJP7kl80GuRk="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    advisory-db.url = "github:rustsec/advisory-db";
    advisory-db.flake = false;
  };

  outputs =
    inputs@{ flake-parts, treefmt-nix, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      imports = [
        treefmt-nix.flakeModule
        ./nixos
      ];
      perSystem =
        {
          self',
          inputs',
          pkgs,
          lib,
          ...
        }:
        let
          rustToolchain = inputs'.fenix.packages.stable.toolchain;
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

          commonAttrs = {
            pname = "server";

            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./server
              ];
            };

            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs =
              with pkgs;
              [
                openssl.dev
              ]
              ++ lib.optionals stdenv.isDarwin [
                libiconv
                darwin.apple_sdk.frameworks.Security
                darwin.apple_sdk.frameworks.SystemConfiguration
              ];

            # the coverage report will run the tests
            doCheck = false;
          };
        in
        {
          devShells.default = pkgs.mkShell {
            inputsFrom = [ self'.packages.default ];
          };

          packages = {
            server-deps = craneLib.buildDepsOnly commonAttrs;

            server-docs = craneLib.cargoDoc (
              commonAttrs
              // {
                cargoArtifacts = self'.packages.server-deps;
              }
            );

            server = craneLib.buildPackage (
              commonAttrs
              // {
                cargoArtifacts = self'.packages.server-deps;
                meta.mainProgram = "server";
              }
            );

            default = self'.packages.server;

            server-docker-image = pkgs.callPackage ./docker-image.nix { inherit (self'.packages) server; };
          };

          checks = {
            inherit (self'.packages) server-docs server server-docker-image;

            lint = craneLib.cargoClippy (
              commonAttrs
              // {
                cargoArtifacts = self'.packages.server-deps;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              }
            );

            coverage-report = craneLib.cargoTarpaulin (
              commonAttrs
              // {
                cargoArtifacts = self'.packages.server-deps;
              }
            );
          };

          treefmt = {
            projectRootFile = ".git/config";
            programs.nixfmt.enable = true;
            programs.rustfmt.enable = true;
            programs.rustfmt.package = craneLib.rustfmt;
            settings.formatter = { };
          };
        };
    };
}
