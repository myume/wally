{
  description = "Wally - wallpaper downloader";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    forAllSystems = function:
      nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (
        system: function nixpkgs.legacyPackages.${system}
      );
  in {
    packages = forAllSystems (pkgs: let
      inherit (pkgs.stdenv.hostPlatform) system;
    in {
      wally = pkgs.callPackage ./nix/package.nix {};
      default = self.packages.${system}.wally;
    });

    devShells = forAllSystems (pkgs: {
      default = pkgs.callPackage ./nix/shell.nix {};
    });

    homeManagerModules = {
      default = self.homeManagerModules.wally;
      wally = import ./nix/hm-module.nix self;
    };
  };
}
