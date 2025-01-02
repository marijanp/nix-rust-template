{ lib, config, ... }:
let
  inherit (lib)
    types
    mkOption
    mkIf
    mkEnableOption
    escapeShellArgs
    optionals
    ;
  cfg = config.services.server;
in
{
  options.services.server = {

    enable = mkEnableOption "server";

    package = mkOption {
      type = types.package;
    };

    address = mkOption {
      type = types.str;
      default = "0.0.0.0";
      example = "0.0.0.0";
      description = ''
        Address to listen on.
      '';
    };

    port = mkOption {
      type = types.port;
      default = 8080;
      example = 8080;
      description = ''
        Port to listen on.
      '';
    };

    metrics = {
      enable = lib.mkEnableOption "Prometheus metrics server";

      address = mkOption {
        type = types.str;
        default = "0.0.0.0";
        example = "0.0.0.0";
        description = ''
          Listen address of the metrics server.
        '';
      };

      port = mkOption {
        type = types.port;
        default = 8081;
        description = ''
          Listen port of the metrics service.
        '';
      };
    };

    logLevel = mkOption {
      type = types.str;
      default = "info";
      description = ''
        The log-level that should be used.
      '';
    };
  };

  config = mkIf cfg.enable {

    systemd.services.server =
      let
        args = escapeShellArgs (
          [
            "--listen-address"
            "${cfg.address}:${toString cfg.port}"
          ]
          ++ optionals cfg.metrics.enable [
            "--metrics-listen-address"
            "${cfg.metrics.address}:${toString cfg.metrics.port}"
          ]
        );
      in
      {
        description = "server";
        documentation = [ "" ];
        wantedBy = [ "multi-user.target" ];
        after = [ "network-online.target" ];
        requires = [ "network-online.target" ];
        environment = {
          RUST_LOG = cfg.logLevel;
        };
        serviceConfig = {
          ExecStart = "${lib.getExe cfg.package} ${args}";
          Restart = "always";
          DynamicUser = true;
        };
      };
  };
}
