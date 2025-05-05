let
  getInput =
    input:
    let
      flakeLock = builtins.fromJSON (builtins.readFile ./flake.lock);
    in
    fetchTarball {
      url =
        flakeLock.nodes.${input}.locked.url
          or "https://github.com/${flakeLock.nodes.${input}.locked.owner}/${
            flakeLock.nodes.${input}.locked.repo
          }/archive/${flakeLock.nodes.nixpkgs.locked.rev}.tar.gz";
      sha256 = flakeLock.nodes.${input}.locked.narHash;
    };
  fenixSrc = getInput "fenix";
  craneSrc = getInput "crane";
  pkgsSrc = getInput "nixpkgs";
  treefmt-nix = getInput "treefmt-nix";

  pkgs = import pkgsSrc {
    overlays = [
      (import "${fenixSrc}/overlay.nix")
      (final: prev: {
        craneLib = (import craneSrc { pkgs = final; }).overrideToolchain final.fenix.stable.toolchain;
        mkWrapper = (import treefmt-nix).mkWrapper final;
      })
      (import ./nix/overlays/default.nix)
      (import ./nix/overlays/dev.nix)
    ];
  };
in
{
  inherit (pkgs)
    server
    server-docs
    lint
    coverage-report
    server-docker-image
    devShell
    ;
  server-deps = pkgs.cargoArtifacts;
}
