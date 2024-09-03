{
  description = "HTMX project for JDP: Talk is cheap, show me the code #4. If you know what a nix flake is, you are a pro dev :) (I don't know what one is)";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    # jdp-htmx-chat = {
    #     type = "github";
    #     owner = "alecbass";
    #     repo = "jdp-htmx-chat";
    # };
  };

  outputs = { self, nixpkgs }: let
    # Copied from https://github.com/iggy-rs/iggy/pull/997/files#diff-206b9ce276ab5971a2489d75eb1b12999d4bf3843b7988cbe8d687cfde61dea0
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

    # Function that loops over each supported system and allows you set configuration for it
    forEachSupportedSystem = callback:
      nixpkgs.lib.genAttrs supportedSystems (supportedSystem:
        callback {
          system = supportedSystem;
          pkgs = nixpkgs.legacyPackages.${supportedSystem};
          # pkgs = dream2nix.inputs.nixpkgs.legacyPackages.${supportedSystem};
        });
  in {
    packages = forEachSupportedSystem ({system, pkgs}: {
      default = pkgs.cargo;
    });

    mkDerivation = forEachSupportedSystem ({system, pkgs}: {
      buildPhase = ''
        cargo build
      '';

      shellHook = ''
        echo hi
        echo $(which cargo)
      '';

      nativeBuildInputs = [
        pkgs.pkg-config
      ];

      buildInputs =
        [
          pkgs.openssl
        ]
        ++ nixpkgs.lib.optionals (pkgs.stdenv.isDarwin) [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
      ];
    });

    devShells = forEachSupportedSystem ({system, pkgs}: {
      default = pkgs.mkShell {
        inputsFrom = [
            # self.packages.${system}.default.devShell
        ];    

        packages = with pkgs; [cargo rustc rustup];
        
        env = {
          OPENSSL_NO_VENDOR = 1;
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        buildInputs =
          [
            pkgs.openssl
          ]
          ++ nixpkgs.lib.optionals (pkgs.stdenv.isDarwin) [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];
      };
    });
  };
}
