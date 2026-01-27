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
    Cmd = [
      "/bin/tini"
      "/bin/balancer"
    ];
    WorkingDir = "/";
  };
}
