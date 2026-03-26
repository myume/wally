self: {
  config,
  pkgs,
  lib,
  ...
}: let
  inherit (lib.modules) mkIf;
  inherit (lib.types) package str ints path bool;
  inherit (lib.options) mkOption mkEnableOption;

  cfg = config.services.wally;
  configPath = "wally/wally.kdl";

  wallhavenConfig = lib.types.submodule {
    options = {
      categories = {
        general = mkOption {
          description = "Whether to enable general wallpapers";
          type = bool;
          default = true;
          apply = lib.trivial.boolToString;
        };
        anime = mkOption {
          description = "Whether to enable anime wallpapers";
          type = bool;
          default = true;
          apply = lib.trivial.boolToString;
        };
        people = mkOption {
          description = "Whether to enable people wallpapers";
          type = bool;
          default = true;
          apply = lib.trivial.boolToString;
        };
      };
    };
  };

  konachanConfig = lib.types.submodule {
    options = {
      explicit = mkOption {
        description = "Whether to enable explicit wallpapers";
        type = bool;
        default = false;
        apply = lib.trivial.boolToString;
      };
    };
  };

  configType = lib.types.submodule {
    options = {
      outputDir = mkOption {
        description = "The location to save wallpapers to";
        type = path;
        default = "${config.home.homeDirectory}/Pictures/wally";
      };

      maxDownloaded = mkOption {
        description = "Maximum number of wallpapers to keep in the output dir";
        type = ints.positive;
        default = 10;
        apply = toString;
      };

      setCommand = mkOption {
        description = ''
          The command to run to set the wallpaper.

          Use {{path}} to substitute where the image path would be.
        '';
        type = str;
        example = "swww img {{path}}";
      };

      wallhaven = mkOption {
        description = "The wallhaven config";
        type = wallhavenConfig;
        default = {};
      };

      konachan = mkOption {
        description = "The konachan config";
        type = konachanConfig;
        default = {};
      };
    };
  };
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

    settings = mkOption {
      description = "The wally configuration";
      type = configType;
      default = {};
    };
  };

  config = mkIf cfg.enable {
    home.packages = [cfg.package];

    xdg.configFile."${configPath}" = {
      text = ''
        general {
            output_dir "${cfg.settings.outputDir}"
            set_command "${cfg.settings.setCommand}"
            max_downloaded ${cfg.settings.maxDownloaded}
        }

        wallhaven {
            categories {
                general #${cfg.settings.wallhaven.categories.general}
                anime #${cfg.settings.wallhaven.categories.anime}
                people #${cfg.settings.wallhaven.categories.people}
            }
        }

        konachan {
            explicit #${cfg.settings.konachan.explicit}
        }
      '';
    };

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
        ExecStart = "${cfg.package}/bin/wally --config ${config.xdg.configHome}/${configPath} --source konachan --evict-oldest --set-wallpaper random";
        RemainAfterExit = false;
      };
    };
  };
}
