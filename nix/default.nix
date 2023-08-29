{
  pkgs,
  system,
  crane,
}: let
  craneLib = crane.lib.${system};

  testDataFilter = path: type: builtins.match ".*/test_data.*" path != null;
  cargoOrTestFiles = path: type:
    (craneLib.filterCargoSources path type) || (testDataFilter path type);

  # Common derivation arguments used for all builds
  commonArgs = {
    src = pkgs.lib.cleanSourceWith {
      src = ../.;
      filter = cargoOrTestFiles;
    };

    nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.darwin.apple_sdk.frameworks.Security
      pkgs.libiconv
    ];
  };

  # Build *just* the cargo dependencies, so we can reuse
  # all of that work (e.g. via cachix) when running in CI
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  # Run clippy (and deny all warnings) on the crate source,
  # resuing the dependency artifacts (e.g. from build scripts or
  # proc-macros) from above.
  #
  # Note that this is done as a separate derivation so it
  # does not impact building just the crate by itself.
  myCrateClippy = craneLib.cargoClippy (commonArgs
    // {
      # Again we apply some extra arguments only to this derivation
      # and not every where else. In this case we add some clippy flags
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "-- --deny warnings";
    });

  # Next, we want to run the tests and collect code-coverage, _but only if
  # the clippy checks pass_ so we do not waste any extra cycles.
  myCrateCoverage = craneLib.cargoNextest (commonArgs
    // {
      cargoArtifacts = myCrateClippy;
    });

  # Build the actual crate itself, reusing the dependency
  # artifacts from above.
  myCrate = craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;
    });
in {
  inherit
    commonArgs
    myCrate
    myCrateClippy
    myCrateCoverage
    ;
}
