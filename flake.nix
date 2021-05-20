{
  description = "Extracting information from flakes";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils, mach-nix }: flake-utils.lib.eachDefaultSystem (
    system:
      let
        pkgs = import nixpkgs { inherit system; };
        doorman = (import ./Cargo.nix { inherit pkgs; rootFeatures = []; }).rootCrate.build;
        doorman-discord = (import ./Cargo.nix { inherit pkgs; rootFeatures = ["discord"]; }).rootCrate.build;
        doorman-discord-bt = (import ./Cargo.nix { inherit pkgs; rootFeatures = ["discord" "bluetooth"]; }).rootCrate.build;
        doorman-bt = import ./Cargo.nix { inherit pkgs; rootFeatures = ["bluetooth"]; };


      in
        rec {
          packages = { inherit doorman doorman-discord-bt; };
          defaultPackage = doorman;
          defaultApp = apps.doorman-discord-bt;
          apps = {
            doorman-discord-bt = flake-utils.lib.mkApp { drv = packages.doorman-discord-bt; };
          };
        }
  );
}
