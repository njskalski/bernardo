{
  description = "Bernardo / Gladius project flake";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.package.version;
#        cleanSrc = pkgs.lib.cleanSource {
#            src = ./.;
#            checkSubmodules = true;
#        };
        cleanSrc = pkgs.lib.fileset.toSource {
          root = ./.;
          fileset = pkgs.lib.fileset.gitTrackedWith { recurseSubmodules = true; } ./.;
        };
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            rust-analyzer
            rust-bin.stable.latest.default
          ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "gladius";
          inherit version;
          src = cleanSrc;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
      }
    );
}
