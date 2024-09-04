![Nix flake check](https://github.com/marijanp/nix-rust-template/actions/workflows/check.yaml/badge.svg)

A Rust project template packaged with Nix.

## Technologies/ Dependencies
- Rust
    - axum-server
    - clap
    - tokio
    - tracing
- Nix
    - [flake-parts](https://flake.parts/)
    - [crane](https://crane.dev/)
    - [fenix](https://github.com/nix-community/fenix)

## Development

The following sections explain how to obtain a development shell, what tools to use during development, and lastly what to run before committing.

### Development Shell

There are two to obtain a development shell: you can configure [direnv](https://direnv.net/) to be dropped in a development shell automatically when you enter the directory (recommended) or do it manually.

#### Automatic Development Shell using `direnv`

First, you will have to [install direnv](https://direnv.net/docs/installation.html), by adding it to your Nix/NixOS configuration or using your package manager.

Afterward, add a `.envrc` file to the root of the project:

```sh
touch .envrc
echo "use flake" >> .envrc
```

Next, enable direnv for this project:

```sh
direnv allow
```

#### Obtaining a Development Shell Manually

Run:

```sh
nix develop
```

### Inside a Development Shell

Inside the development shell, you can use `cargo` as usual during development.

### Before you Commit

Because Nix gives us gives us a high degree of reproducibility, by building our project and running the checks locally and making them succeed, we can be very certain it will pass the pipeline too.

#### Build

You can explore the buildable outputs of any flake project by running:

```sh
nix flake show
```

To build e.g. `server` you can then run:

```sh
nix build .#server
```

#### Run the Checks

To run all the "checks" of this project, like formatting, lint, audit, etc. checks, run:

```sh
nix flake check
```

To run a single check e.g. the format check, run:

```sh
nix build .#checks.<system>.treefmt
```

### Format

Code for the whole project tree can be formatted by running `nix fmt` from the project's root or anywhere in the tree, but be warned that it will only format code inside the sub-tree.

The `nix fmt` command currently formats all the `Rust` and `Nix` code in the tree. To add support for more languages you'll have to adjust the `treefmt` attribute-set in the `flake.nix` accordingly. A list of already supported formatters can be found [here](https://numtide.github.io/treefmt/formatters/).
