let
  pkgs = import <nixpkgs> {
    crossSystem = { config = "aarch64-unknown-linux-gnu"; };
  };
in pkgs.callPackage ({ mkShell, pkg-config, zlib }:
  mkShell {
    nativeBuildInputs = [ pkg-config ]; # you build dependencies here
    buildInputs = with pkgs; [
      udev
      alsa-lib
      vulkan-loader
      xorg.libX11
      xorg.libXcursor
      xorg.libXi
      xorg.libXrandr
      libxkbcommon
      wayland
    ];
  }) { }
