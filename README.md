# Balancer

A Rust program written to check a node's eligibility to be a web server in a high availability setup.

## Development

Development is easiest with Nix using the provided flake, simply `nix develop`. Otherwise, a Rust toolchain is required.

The program registers itself with a [Consul](https://www.hashicorp.com/en/products/consul) instance, and so a running instance is also required.

## Configuration

Examples of how to configure the program can be found in [examples](examples/) and some amount of documentation of the options is available in [examples/config.toml](examples/config.toml).
