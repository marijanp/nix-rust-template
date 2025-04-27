{
  dockerTools,
  server,
  lib,
}:
dockerTools.buildLayeredImage {
  name = "ghcr.io/marijanp/nix-rust-template";
  tag = "0.1.0";
  config = {
    Cmd = lib.getExe server;
    ExposedPorts."8080/tcp" = { };
  };
}
