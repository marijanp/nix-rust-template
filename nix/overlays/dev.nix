final: prev: {
  devShell = final.mkShell {
    inputsFrom = [ final.server ];
    nativeBuildInputs = [
      (final.mkWrapper {
        projectRootFile = ".git/config";
        programs.nixfmt.enable = true;
        programs.rustfmt.enable = true;
        programs.rustfmt.package = final.craneLib.rustfmt;
        settings.formatter = { };
      })
    ];
  };
}
