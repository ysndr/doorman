{
  description = "Extracting information from flakes";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils, mach-nix }: flake-utils.lib.eachDefaultSystem (
    system:
      let
        pkgs = import nixpkgs { inherit system; };

        customBuildRustCrateForPkgs = pkgs: pkgs.buildRustCrate.override {
          defaultCrateOverrides = pkgs.defaultCrateOverrides // {
            hyper = attrs: {
              buildInputs =
                pkgs.lib.optionals
                  pkgs.stdenv.isDarwin
                  [ pkgs.darwin.apple_sdk.frameworks.Security pkgs.libiconv ];
            };
            doorman = attrs: {
              buildInputs =
                pkgs.lib.optionals
                  pkgs.stdenv.isDarwin
                  [ pkgs.darwin.apple_sdk.frameworks.Security pkgs.libiconv ];
            };
          };
        };
        base = (
          import ./Cargo.nix {
            inherit pkgs;
            buildRustCrateForPkgs = customBuildRustCrateForPkgs;
          }
        );

        doorman = base.rootCrate.build;
        doorman-discord = doorman.override { features = [ "discord" ]; };
        doorman-discord-bt = doorman.override { features = [ "discord" "bluetooth" ]; };
      in
        rec {
          packages = { inherit doorman doorman-discord-bt doorman-discord; };
          defaultPackage = doorman;
          defaultApp = apps.doorman-discord-bt;
          apps = {
            doorman-discord-bt = flake-utils.lib.mkApp { drv = packages.doorman-discord-bt; };
          };
        }
  );
}
