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
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    launchd.user.agents.nitora = {
      serviceConfig = {
        Label = "dev.nitora.daemon";
        ProgramArguments = [
          "${cfg.package}/bin/nitora"
          "serve"
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
