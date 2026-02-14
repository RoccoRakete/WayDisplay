{
  description = "Rust development environment and package for WayDisplay";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        # This allows 'nix build' and 'nix profile install'
        packages.default = pkgs.callPackage ./default.nix { };

        # Your development shell (nix develop)
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            pkg-config
          ];

          buildInputs = with pkgs; [
            libxkbcommon
            libGL
            wayland
            glib
            clang
          ];

          # Ensure rust-analyzer and the compiler find the libraries
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            with pkgs;
            [
              libxkbcommon
              libGL
              wayland
            ]
          );

          shellHook = ''
            echo "Ready to develop way_display!"
          '';
        };
      }
    );
}
