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
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.flake = false;
    fenix.url = "github:nix-community/fenix";
    fenix.flake = false;
    crane.url = "github:ipetkov/crane";
    crane.flake = false;
  };

  outputs =
    {
      self,
      nixpkgs,
      treefmt-nix,
      fenix,
      crane,
      ...
    }:
    let
      # there is also nixpkgs.lib.systems.flakeExposed
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      withPkgs =
        pkgsCallback:
        nixpkgs.lib.genAttrs allSystems (
          system:
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [
                (import "${fenix}/overlay.nix")
                (final: prev: {
                  craneLib = (import crane { pkgs = final; }).overrideToolchain final.fenix.stable.toolchain;
                  mkWrapper = (import treefmt-nix).mkWrapper final;
                })
                (import ./nix/overlays/default.nix)
                (import ./nix/overlays/dev.nix)
              ];
            };
          in
          pkgsCallback { inherit pkgs system; }
        );
    in
    {

      devShells = withPkgs (
        { pkgs, ... }:
        {
          default = pkgs.devShell;
        }
      );

      packages = withPkgs (
        { pkgs, system }:
        {
          inherit (pkgs)
            server
            server-docs
            lint
            coverage-report
            server-docker-image
            ;
          server-deps = pkgs.cargoArtifacts;
          default = self.packages.${system}.server;
        }
      );

      checks = withPkgs (
        { pkgs, ... }:
        {
          inherit (pkgs)
            server
            server-docs
            lint
            coverage-report
            ;
        }
      );
    };
}
