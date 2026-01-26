{
  dockerTools,
  tini,
  balancer,
}:
dockerTools.buildLayeredImage {
  name = "ashleah-rest";
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
