{
  lib,
  stdenv,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "nitora";
  version = (lib.importTOML ../Cargo.toml).package.version;

  src = lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;

  # All Apple frameworks are provided by the default SDK in stdenv on Darwin;
  # we only need to tell the linker which ones to pull in.
  env = lib.optionalAttrs stdenv.hostPlatform.isDarwin {
    NIX_LDFLAGS = toString [
      "-framework" "AppKit"
      "-framework" "CoreFoundation"
      "-framework" "CoreGraphics"
      "-framework" "Foundation"
      "-framework" "IOKit"
      "-framework" "Metal"
      "-framework" "QuartzCore"
    ];
  };

  # Hardware-dependent tests cannot run in the Nix sandbox
  doCheck = false;

  meta = {
    description = "macOS XDR/EDR brightness control CLI";
    license = lib.licenses.mit;
    mainProgram = "nitora";
    platforms = lib.platforms.darwin;
  };
}
