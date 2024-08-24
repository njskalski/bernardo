{
  description = "Bernardo / Gladius project flake";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";

    subproject = {
        url = "git+file:third-party/nvim-treesitter"; # the submodule is in the ./subproject dir
        flake = false;
      };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      subproject,
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

        cleanedSource = pkgs.lib.sources.cleanSource ./.;

        combinedSource = pkgs.runCommand "combined-source" { } ''
          mkdir -p $out
          cp -r ${cleanedSource}/* $out/
          mkdir -p $out/third-party/nvim-treesitter
          cp -r ${builtins.toPath subproject}/* $out/third-party/nvim-treesitter/
        '';
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
          src = combinedSource;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
      }
    );
}
