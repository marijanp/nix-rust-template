final: prev:
let
  treefmtEval = final.treefmt-nix.evalModule final {
    projectRootFile = ".git/config";
    programs.nixfmt.enable = true;
    programs.rustfmt.enable = true;
    programs.rustfmt.package = final.craneLib.rustfmt;
    settings.formatter = { };
  };
in
{
  treefmt-formatter = treefmtEval.config.build.wrapper;

  devShell = final.mkShell {
    inputsFrom = [ final.server ];
    packages = [
      final.treefmt-formatter
    ];
  };

  formatting-check = treefmtEval.config.build.check ../../.;
}
