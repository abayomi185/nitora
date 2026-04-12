{
  description = "nitora — macOS XDR/EDR brightness control CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      darwinSystems = [
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      forDarwin = nixpkgs.lib.genAttrs darwinSystems;
    in
    {
      packages = forDarwin (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          nitora = pkgs.callPackage ./nix/package.nix { };
        in
        {
          default = nitora;
          inherit nitora;
        }
      );

      apps = forDarwin (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.nitora}/bin/nitora";
        };
      });

      devShells = forDarwin (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            name = "nitora-dev";

            packages = [
              pkgs.rustc
              pkgs.cargo
              pkgs.rustfmt
              pkgs.clippy
              pkgs.rust-analyzer
            ];

            # The SDK in stdenv provides all Apple frameworks; tell the linker
            # which ones to pull in for local `cargo build` invocations.
            NIX_LDFLAGS = pkgs.lib.concatStringsSep " " [
              "-framework" "AppKit"
              "-framework" "CoreFoundation"
              "-framework" "CoreGraphics"
              "-framework" "Foundation"
              "-framework" "IOKit"
              "-framework" "Metal"
              "-framework" "QuartzCore"
            ];

            RUST_BACKTRACE = "1";
          };
        }
      );

      overlays.default = _final: prev: {
        nitora = prev.callPackage ./nix/package.nix { };
      };

      darwinModules = {
        default = import ./nix/modules/darwin.nix;
        nitora = import ./nix/modules/darwin.nix;
      };
    };
}
