# Home Manager module: installs vkc and writes its config file. The API token
# stays out of the store: point tokenFile at a runtime secret path.
self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.vkc;
in {
  options.programs.vkc = {
    enable = lib.mkEnableOption "Vikunja API CLI (vkc)";
    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "vikunja-client-cli.packages.\${system}.default";
      description = "vkc package to install.";
    };
    url = lib.mkOption {
      type = lib.types.str;
      example = "https://vikunja.example.com/api/v1";
      description = "Vikunja API base URL.";
    };
    tokenFile = lib.mkOption {
      # vkc reads the path literally, so $XDG_RUNTIME_DIR-style placeholders
      # (e.g. Home Manager agenix defaults) must be resolved by the caller.
      type = lib.types.nullOr (lib.types.strMatching "/[^$]*");
      default = null;
      example = "/run/agenix/vikunja-token";
      description = "Absolute runtime path of a file containing the API token; read on every invocation, never expanded.";
    };
    output = lib.mkOption {
      type = lib.types.enum ["json" "table"];
      default = "json";
      description = "Default output format.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [cfg.package];
    xdg.configFile."vkc/config.toml".source = (pkgs.formats.toml {}).generate "vkc-config.toml" ({
        inherit (cfg) url output;
      }
      // lib.optionalAttrs (cfg.tokenFile != null) {token_file = cfg.tokenFile;});
  };
}
