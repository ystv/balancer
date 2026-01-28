{
  dockerTools,
  tini,
  balancer,
  cacert,
}:
dockerTools.buildLayeredImage {
  name = "balancer";
  tag = "latest";

  contents = [
    tini
    balancer
    cacert
  ];

  config = {
    Entrypoint = [
      "/bin/tini"
      "/bin/balancer"
      "--"
    ];
    WorkingDir = "/";
  };
}
