{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.nitora;
in
{
  options.programs.nitora = {
    enable = lib.mkEnableOption "Nitora brightness control daemon";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ../package.nix { };
      defaultText = lib.literalExpression "pkgs.callPackage ../package.nix { }";
      description = "The Nitora package to use.";
    };

    socketPath = lib.mkOption {
      type = lib.types.str;
      default = "/tmp/nitora.sock";
      description = "Path to the Unix socket Nitora listens on (NITORA_SOCKET).";
    };

    autoEnable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Automatically enable XDR brightness when the service starts.";
    };

    brightness = lib.mkOption {
      type = lib.types.ints.between 0 100;
      default = 100;
      description = "Initial brightness level (0-100) applied when auto-enabled.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    launchd.user.agents.nitora = {
      serviceConfig = {
        Label = "dev.nitora.daemon";
        ProgramArguments =
          [
            "${cfg.package}/bin/nitora"
            "serve"
          ]
          ++ lib.optionals cfg.autoEnable [
            "--auto-enable"
          ]
          ++ lib.optionals (cfg.brightness != 100) [
            "--brightness"
            (toString cfg.brightness)
          ];
        EnvironmentVariables = {
          NITORA_SOCKET = cfg.socketPath;
        };
        KeepAlive = true;
        RunAtLoad = true;
        ProcessType = "Interactive";
        StandardOutPath = "/tmp/nitora.stdout.log";
        StandardErrorPath = "/tmp/nitora.stderr.log";
      };
    };
  };
}
