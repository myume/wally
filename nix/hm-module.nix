self: {
  config,
  pkgs,
  lib,
  ...
}: let
  inherit (lib.modules) mkIf;
  inherit (lib.types) package str;
  inherit (lib.options) mkOption mkEnableOption;

  cfg = config.services.wally;
in {
  options.services.wally = {
    enable = mkEnableOption "Wally, wallpaper scraper and randomizer";

    package = mkOption {
      description = "The Wally package";
      type = package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.wally;
    };

    frequency = mkOption {
      description = "The frequency of wallpaper updates";
      type = str;
      default = "daily";
    };

    config = mkOption {
      description = "The location of the config file";
      type = str;
    };
  };

  config = mkIf cfg.enable {
    home.packages = [cfg.package];

    systemd.user.timers.wally = {
      Install = {
        WantedBy = ["timers.target"];
      };
      Timer = {
        OnCalendar = cfg.frequency;
        Persistent = true;
        Unit = "wally.service";
      };
    };

    systemd.user.services.wally = {
      Service = {
        Type = "oneshot";
        ExecStart = "${cfg.package}/bin/wally --config ${cfg.config} --source konachan --evict-oldest --set-wallpaper random";
        RemainAfterExit = false;
      };
    };
  };
}
