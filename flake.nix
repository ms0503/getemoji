{
  inputs = {
    fenix = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:nix-community/fenix";
    };
    flake-compat = {
      flake = false;
      url = "github:NixOS/flake-compat";
    };
    flake-parts = {
      inputs.nixpkgs-lib.follows = "nixpkgs";
      url = "github:hercules-ci/flake-parts";
    };
    git-hooks = {
      inputs = {
        flake-compat.follows = "";
        nixpkgs.follows = "nixpkgs";
      };
      url = "github:cachix/git-hooks.nix";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems = {
      flake = false;
      url = "github:nix-systems/default";
    };
    treefmt-nix = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:numtide/treefmt-nix";
    };
  };
  outputs =
    inputs@{ flake-parts, systems, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        ./nix/treefmt.nix
        ./nix/git-hooks.nix
      ];
      perSystem =
        {
          config,
          inputs',
          lib,
          pkgs,
          ...
        }:
        {
          devShells.shell = pkgs.mkShell {
            packages =
              config.pre-commit.settings.enabledPackages ++ lib.attrValues config.treefmt.build.programs;
            shellHook = ''
              ${config.pre-commit.shellHook}
            '';
          };
          packages.default =
            let
              inherit (pkgs) callPackage makeRustPlatform;
              rustPlatform = makeRustPlatform {
                cargo = inputs'.fenix.packages.latest.toolchain;
                rustc = inputs'.fenix.packages.latest.toolchain;
              };
            in
            callPackage ./nix/package.nix {
              inherit rustPlatform;
            };
        };
      systems = import systems;
    };
}
