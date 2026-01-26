{
  dockerTools,
  tini,
  balancer,
}:
dockerTools.buildLayeredImage {
  name = "balancer";
  tag = "latest";

  contents = [
    tini
    balancer
  ];

  config = {
    Cmd = [
      "/bin/tini"
      "/bin/balancer"
    ];
    WorkingDir = "/";
  };
}
