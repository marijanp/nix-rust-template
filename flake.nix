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
    inputs@{
      self,
      flake-parts,
      treefmt-nix,
      ...
    }:
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
              ];

            # the coverage report will run the tests
            doCheck = false;
          };
        in
        {
          devShells.default =
            let
              DATABASE_FILE = "dev-database.db";
            in
            pkgs.mkShell {
              inputsFrom = [ self'.packages.default ];
              packages = [
                pkgs.sqlx-cli
                pkgs.sqlite
                self'.packages.run-migrations
              ];
              inherit DATABASE_FILE;
              DATABASE_URL = "sqlite:${DATABASE_FILE}";
            };

          packages = {
            run-stack = pkgs.writeShellApplication {
              name = "run-stack";
              runtimeInputs = [
                pkgs.caddy
                pkgs.simple-http-server
                self'.packages.run-migrations
              ];
              text =
                let
                  caddyConfig = pkgs.writeTextFile {
                    name = "Caddyfile";
                    text = ''
                      :8000 {
                        reverse_proxy /api/* http://0.0.0.0:8002
                        reverse_proxy /auth/* http://0.0.0.0:8002
                        reverse_proxy /* http://0.0.0.0:8001
                      }
                    '';
                  };
                in
                ''
                  set -euo pipefail

                  # Create database file if it does NOT exist
                  if [ ! -f "$DATABASE_FILE" ]; then
                      DATABASE_FILE=$(touch "$DATABASE_FILE")
                      echo "Created new database file: $DATABASE_FILE"
                  fi

                  run-migrations server/migrations "$DATABASE_FILE"


                  simple-http-server --index --port 8001 frontend &
                  PID_FRONTEND=$!

                  cargo run -- --listen-address 0.0.0.0:8002 \
                               --host-url http://localhost:8000 \
                               --database-url "sqlite://$DATABASE_FILE" \
                               --openid-provider-url https://marijan.eu.auth0.com/ \
                               --openid-client-id "$OPENID_CLIENT_ID" \
                               --openid-client-secret "$OPENID_CLIENT_SECRET" \
                               &
                  PID_BACKEND=$!

                  caddy run --config ${caddyConfig} --adapter caddyfile &
                  PID_PROXY=$!

                  cleanup() {
                    kill $PID_FRONTEND $PID_BACKEND $PID_PROXY
                    wait $PID_FRONTEND $PID_BACKEND $PID_PROXY
                    exit 0
                  }

                  trap cleanup SIGINT

                  wait $PID_FRONTEND $PID_BACKEND $PID_PROXY
                '';
            };

            run-migrations = pkgs.writeShellApplication {
              name = "run-migrations";
              runtimeInputs = [ pkgs.sqlite ];
              text = ''
                >&2 echo "Applying migrations"
                for migration_file in "$1"/*.sql; do
                  >&2 echo "Applying migration: $migration_file"
                  sqlite3 "$2" < "$migration_file"
                done
              '';
            };

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
                passthru = {
                  migrations = ./server/migrations;
                  inherit (self'.packages) run-migrations;
                };
              }
            );

            default = self'.packages.server;
          };

          checks = {
            inherit (self'.packages) server-docs server;

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
      flake =
        let
          system = "x86_64-linux";
          pkgs = inputs.nixpkgs.legacyPackages.${system};
        in
        {
          packages.${system}.server-docker-image = pkgs.callPackage ./docker-image.nix {
            inherit (self.packages.${system}) server;
          };
        };
    };
}
