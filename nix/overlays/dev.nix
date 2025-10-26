final: prev: {
  treefmt-formatter = final.mkWrapper {
    projectRootFile = ".git/config";
    programs.nixfmt.enable = true;
    programs.rustfmt.enable = true;
    programs.rustfmt.package = final.craneLib.rustfmt;
    settings.formatter = { };
  };
  devShell = final.mkShell {
    inputsFrom = [ final.server ];
    packages = [
      final.treefmt-formatter
    ];
  };
}
