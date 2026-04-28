self: {
  config,
  pkgs,
  lib,
  ...
}: let
  inherit (lib.modules) mkIf;
  inherit (lib.types) package str ints path bool enum nullOr listOf;
  inherit (lib.options) mkOption mkEnableOption;
  inherit (lib.trivial) boolToString;

  cfg = config.services.wally;
  configPath = "wally/wally.kdl";

  wallhavenConfig = lib.types.submodule {
    options = {
      categories = {
        general =
          mkEnableOption "general wallpapers"
          // {
            default = true;
          };
        anime =
          mkEnableOption "anime wallpapers"
          // {
            default = true;
          };
        people =
          mkEnableOption "people wallpapers"
          // {
            default = true;
          };
      };
    };
  };

  konachanConfig = lib.types.submodule {
    options = {
      explicit =
        mkEnableOption "explicit wallpapers";
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
      };

      setCommand = mkOption {
        description = ''
          The command to run to set the wallpaper.

          Use {{path}} to substitute where the image path would be.
        '';
        type = listOf str;
        example = ["swww img {{path}}"];
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

  wallpaperSources = [
    "wallhaven"
    "pixiv"
    "konachan"
  ];
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

    defaultSource = mkOption {
      description = ''
        The default source to pull wallpapers from.

        Possible sources:
        ${builtins.concatStringsSep "\n" wallpaperSources}
      '';
      type = nullOr (enum wallpaperSources);
      default = null;
    };

    evictOldest = mkOption {
      description = "Whether to evict the oldest entries from the output dir when the max number of files is reached";
      type = bool;
      default = true;
    };
  };

  config = mkIf cfg.enable {
    home.packages = [cfg.package];

    xdg.configFile."${configPath}" = {
      text = ''
        general {
            output_dir "${cfg.settings.outputDir}"
            ${builtins.concatStringsSep "\n\t" (map (command: "set_command \"${command}\"") cfg.settings.setCommand)}
            max_downloaded ${toString cfg.settings.maxDownloaded}
        }

        wallhaven {
            categories {
                general #${boolToString cfg.settings.wallhaven.categories.general}
                anime #${boolToString cfg.settings.wallhaven.categories.anime}
                people #${boolToString cfg.settings.wallhaven.categories.people}
            }
        }

        konachan {
            explicit #${boolToString cfg.settings.konachan.explicit}
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
        ExecStart = let
          additionalFlags = builtins.filter (flag: flag != "") [
            (
              if cfg.defaultSource != null
              then "--source ${cfg.defaultSource}"
              else ""
            )
            (
              if cfg.evictOldest
              then "--evict-oldest"
              else ""
            )
          ];
        in "${cfg.package}/bin/wally --config ${config.xdg.configHome}/${configPath} ${lib.concatStringsSep " " additionalFlags} --set-wallpaper random";
        RemainAfterExit = false;
      };
    };
  };
}
