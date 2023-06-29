{
  description = "Development environment deployment flake for Waysight";

  inputs = {
    nixpkgs.url = "github:nixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in
    {
      devShells.x86_64-linux.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          libglvnd
          pkg-config
          wayland
          wayland-protocols
          libseat
          libinput
          mesa
          libxkbcommon
        ];
      };

    };
}
