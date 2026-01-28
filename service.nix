{ config, pkgs, ... }:

let
  vilahack_user_backend = pkgs.callPackage ./default.nix {};
in {
  systemd.services.vilahack_user_backend = {
    description = "VilaHack User Backend";
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart = "${vilahack_backend}/bin/vilahack_user_backend";
      Restart = "always";
      User = "user_backend";
      Group = "vilahack";

      EnvironmentFile = "${./.env}";
    };
  };
}
