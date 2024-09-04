{ self, inputs, ... }:
{
  flake = {
    checks."x86_64-linux" =
      let
        pkgs = inputs.nixpkgs.legacyPackages."x86_64-linux";
      in
      {
        verify-server-service = pkgs.callPackage ./tests/verify-host-configuration.nix {
          hostConfiguration = {
            imports = [
              self.nixosModules.server
            ];
            services.server.enable = true;
          };
          verifyServices = [ "server.service" ];
        };
      };
    nixosModules = {
      server =
        { pkgs, ... }:
        {
          imports = [ ./modules/server.nix ];
          services.server.package = self.packages.${pkgs.system}.server;
        };
    };
  };
}
