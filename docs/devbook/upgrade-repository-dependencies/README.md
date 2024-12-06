# Upgrade the project's dependencies in the repository

## Introduction

This devbook provides step-by-step instructions to upgrade the dependencies in the repository, including Rust crates, documentation, and JavaScript packages.

## Update dependencies tool

The `update_dependencies.sh` script allows you to update dependencies performing all the steps described in the next chapter.

It requires having `cargo-edit` and `cargo-audit` installed, which can be done with the following command:

```
cargo install cargo-edit cargo-audit
```

To start the update, execute the command below from the root of the repository:

```
. ./docs/devbook/upgrade-repository-dependencies/upgrade_dependencies.sh
```

By default, Rust dependencies are updated to the latest version. If you want to only update to the latest compatible versions, add the `--incompatible` option to the command.

**Warning**: Before re-running the script, you need to revert the modified code to its original state to avoid incrementing the versions of crates and JSON packages twice.

## Steps

### Upgrade Rust outdated dependencies

We recommend using [Dependi](https://dependi.io/) VS Code extension to identify and update outdated dependencies.

To do this, verify the dependencies in the `Cargo.toml` file for each Rust crate in the repository.

- Bring up the Command Palette with `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS).
- Type `dependi` and select `Update All Dependencies to Latest Version`.

![Run dependi](./img/run-dependi.png)

After the version upgrade, upgrade the dependencies in the `Cargo.lock` file that are not directly listed in any `Cargo.toml` (ie dependencies of dependencies) by running:

```bash
cargo update
```

Create a dedicated commit, e.g.:

```bash
chore: update Rust dependencies
```

Next, ensure that upgrading the dependencies has not introduced any breaking changes in the codebase.

If breaking changes are introduced, resolve them, and then create a dedicated commit, e.g.:

```bash
fix: resolve breaking changes introduced by upgrading 'crate_name' from 'x.0.99' to 'x.1.0'
```

### Bump Rust crates versions

Increment the patch versions in the `Cargo.toml` by 1 (eg. from `x.x.1` to `x.x.2`) for each Rust crate in the repository.

Create a dedicated commit, e.g:

```bash
chore: bump crates versions
```

### Upgrade the documentation website dependencies

From the root of the repository, run:

```bash
cd docs/website
make upgrade
```

Create a dedicated commit, e.g.:

```bash
chore: upgrade doc dependencies

By running 'make upgrade' command.
```

### Upgrade client wasm `ci-test/` dependencies

From the root of the repository, run:

Upgrade the `ci-test` dependencies by running:

```bash
make -C mithril-client-wasm upgrade-ci-test-deps
```

Create a dedicated commit, e.g.:

```bash
chore: upgrade mithril client wasm 'ci-test' dependencies

By running 'make upgrade-ci-test-deps' command.
```

### Upgrade client wasm examples dependencies

From the root of the repository, run:

[!IMPORTANT]: This command must be run before upgrading client wasm examples.

```bash
make -C mithril-client-wasm build
```

Upgrade the examples dependencies by running:

```bash
make -C examples/client-wasm-nodejs upgrade
make -C examples/client-wasm-web upgrade
```

Create a dedicated commit, e.g.:

```bash
chore: upgrade mithril client wasm examples dependencies

By running 'make upgrade' command in 'examples/client-wasm-nodejs' and 'examples/client-wasm-web'.
```

### Upgrade the explorer dependencies

[!IMPORTANT]: This command must be run before upgrading `mithril-explorer`.

```bash
make -C mithril-client-wasm build
```

From the root of the repository, run:

```bash
make -C mithril-explorer upgrade
```

Create a dedicated commit, e.g.:

```bash
chore: upgrade explorer dependencies

By running 'make upgrade' command.
```

### Bump Javascript packages versions

Increment the patch versions in the `package.json` by 1 (eg. from `x.x.1` to `x.x.2`) for each Javascript package in the repository (`www`, `www-test` and `mithril-explorer`, `docs/website`).

Then, from the root of the repository, run the commands:

```bash
cd mithril-client-wasm
make www-install && make www-test-install
```

```bash
cd mithril-explorer
make install
```

```bash
cd docs/website
make install
```

Create a dedicated commit, e.g.:

```bash
chore: bump javascript packages versions

By running:
- 'make install' command in 'examples/client-wasm-web'.
- 'make install' command in 'examples/client-wasm-nodejs'.
- 'make install' command in 'mithril-explorer'.
- 'make install' command in 'docs/website'.
```

### Upgrade Nix Flake dependencies

```bash
nix flake update
```

Create a dedicated commit, e.g.:

```bash
chore: update nix flake dependencies

By running 'nix flake update' command.
```

### Run a security audit on the Rust dependencies:

```bash
cargo audit
```
