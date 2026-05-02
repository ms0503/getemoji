{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
  ];
  perSystem =
    {
      inputs',
      lib,
      pkgs,
      ...
    }:
    {
      treefmt.programs = {
        mdformat = {
          enable = true;
          plugins =
            ps: with ps; [
              mdformat-gfm
            ];
          settings = {
            end-of-line = "lf";
            number = true;
            wrap = 80;
          };
        };
        nixfmt = {
          enable = true;
          excludes = [
            "_sources/generated.nix"
          ];
        };
        rustfmt = {
          edition = "2024";
          enable = true;
          package = inputs'.fenix.packages.latest.rustfmt;
        };
        taplo.enable = true;
      };
    };
}
