final: prev: {
  cargoArtifacts = final.callPackage ../package.nix {
    craneLibBuild = final.craneLib.buildDepsOnly;
    cargoArtifacts = null;
  };

  server = final.callPackage ../package.nix {
    craneLibBuild = final.craneLib.buildPackage;
  };

  server-docs = final.callPackage ../package.nix {
    craneLibBuild = final.craneLib.cargoDoc;
  };

  lint =
    (final.callPackage ../package.nix {
      craneLibBuild = final.craneLib.cargoClippy;
    }).overrideAttrs
      (_oldAttrs: {
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      });

  coverage-report = final.callPackage ../package.nix {
    craneLibBuild = final.craneLib.cargoTarpaulin;
  };

  server-docker-image = final.callPackage ../docker-image.nix { };
}
