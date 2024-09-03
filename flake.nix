{
  description = "HTMX project for JDP: Talk is cheap, show me the code #4. If you know what a nix flake is, you are a pro dev :) (I don't know what one is)";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    jdp-htmx-chat = {
        type = "github";
        owner = "alecbass";
        repo = "jdp-htmx-chat";
    };
  };

  outputs = { self, nixpkgs, pkgs }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forEachSupportedSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (supportedSystem:
        f {
          system = supportedSystem;
          # pkgs = dream2nix.inputs.nixpkgs.legacyPackages.${supportedSystem};
        });
  in {
    # packages = forEachSupportedSystem ({pkgs, ...}: rec {
    #     
    # };
    packages = with pkgs; [
      rustup
      cargo 
    ];

    # packages.${supported}.hello = nixpkgs.legacyPackages.x86_64-linux.hello;

    # packages.x86_64-linux.default = self.packages.x86_64-linux.hello;
  };
}
