{ pkgs }:

let
  # Definition of the desktop entry
  desktopItem = pkgs.makeDesktopItem {
    name = "way-display";
    exec = "way-display";
    icon = "way-display";
    comment = "A Wayland display utility built with egui";
    desktopName = "WayDisplay";
    genericName = "Display Utility";
    categories = [ "Utility" ];
    terminal = false;
  };
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = "way-display";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
    rustc
    cargo
    makeWrapper
    copyDesktopItems
  ];

  buildInputs = with pkgs; [
    libxkbcommon
    libGL
    wayland
    libX11
    libXcursor
    libXi
    libXrandr

    glib
    openssl
    sqlite
    dbus
  ];

  desktopItems = [ desktopItem ];

  postInstall = ''
    wrapProgram $out/bin/way-display \
      --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath buildInputs}"

    mkdir -p $out/share/icons/hicolor/256x256/apps

    cp assets/monitor.jpg $out/share/icons/hicolor/256x256/apps/way-display.png
  '';

  meta = with pkgs.lib; {
    description = "A Wayland display utility built with egui";
    homepage = "https://github.com/RoccoRakete/WayDisplay";
    maintainers = [ ];
    platforms = platforms.linux;
  };
}
