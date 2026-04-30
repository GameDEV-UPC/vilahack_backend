{ config, pkgs, ... }:

let
  vilahack_backend = pkgs.callPackage ./default.nix {};
in {
  systemd.services.vilahack_backend = {
    description = "VilaHack Backend";

    wantedBy = [ 
      "multi-user.target"
      "nginx.service"
    ];

    requires = [
      "network.target"
      "opentelemetry-collector.service"
      "loki.service"
      "tempo.service"
      "prometheus.service"
    ];
    
    path = with pkgs; [ nix bash ];

    serviceConfig = {
      ExecStart = "${vilahack_backend}/bin/backend";
      Restart = "always";
      User = "backend";
      Group = "vilahack";

      StateDirectory = "vilahack_backend";
      Environment = "HOME=/var/lib/vilahack_backend";
    };
  };
}
