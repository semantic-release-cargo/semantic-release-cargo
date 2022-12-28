{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.flake-utils.follows = "flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.flake-utils.follows = "flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    pre-commit-hooks,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      rust-components = with pkgs; [
        rustc
        cargo
        clippy
        rust-analyzer
        rustfmt
      ];

      craneLib = crane.lib.${system};

      # Common derivation arguments used for all builds
      commonArgs = {
        src = craneLib.cleanCargoSource ./.;

        # Add extra inputs here or any other derivation settings
        # doCheck = true;
        buildInputs = [];

        nativeBuildInputs = [];
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

      pre-commit-check = pre-commit-hooks.lib.${system}.run {
        src = ./.;
        hooks = {
          actionlint.enable = true;
          alejandra.enable = true;
          prettier.enable = true;
        };
      };
    in {
      packages.default = myCrate;
      checks = {
        inherit
          myCrate
          myCrateClippy
          myCrateCoverage
          pre-commit-check
          ;
      };
      devShells = {
        default = nixpkgs.legacyPackages.${system}.mkShell {
          buildInputs = commonArgs.buildInputs;
          nativeBuildInputs =
            commonArgs.nativeBuildInputs
            ++ rust-components
            ++ [
              pkgs.nodejs
            ];

          inherit (self.checks.${system}.pre-commit-check) shellHook;
        };
      };
    });
}
