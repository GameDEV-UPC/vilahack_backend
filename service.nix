{ config, pkgs, ... }:

let
  vilahack_backend = pkgs.callPackage ./default.nix {};
in {
  systemd.services.vilahack_backend = {
    description = "VilaHack Backend";
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart = "${vilahack_backend}/bin/vilahack_backend";
      Restart = "always";
      User = "backend";
      Group = "vilahack";

      EnvironmentFile = "${./.env}";
    };
  };
}
