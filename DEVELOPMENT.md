# Development

## Man page

The man page is generated at build time with [clap_mangen](https://crates.io/crates/clap_mangen) and written to `man/git-b.1`.

CLI flags and help live in `cli/src/lib.rs` (`git-b-cli` crate). Both the binary and `build.rs` use `git_b_cli::Args::command()` so the man page stays in sync. Bump `version` in the root `[workspace.package]` section of `Cargo.toml` when releasing.

Regenerate after CLI changes:

```sh
cargo build
```

## Releasing a new version

Use these steps when cutting a release for GitHub and Homebrew.

### 1. Bump the version

Update the version in `Cargo.toml`:

```toml
version = "1.3.0"
```

Commit that change on `main` (or your release branch) before tagging.

### 2. Build release artifacts

From the repo root, run the `build` script with the new version:

```sh
unset CARGO_TARGET_DIR   # if set (e.g. by Cursor), so artifacts land in ./target
./build 1.3.0
```

On macOS without the Linux cross-linker installed, build macOS tarballs only:

```sh
./build 1.3.0 --skip-linux
```

This script:

- Cross-compiles release binaries for macOS (Intel and Apple Silicon) and Linux (x86_64)
- Creates tarballs in the repo root (each contains `git-b` and `share/man/man1/git-b.1`):
  - `git-b-v<version>-x86_64-apple-darwin.tar.gz`
  - `git-b-v<version>-aarch64-apple-darwin.tar.gz`
  - `git-b-v<version>-x86_64-unknown-linux-gnu.tar.gz`
- Updates `Formula/git-b.rb` with the new `VERSION` and macOS `sha256` checksums

Release archives use this layout so Homebrew can run `man1.install "share/man/man1/git-b.1"`.

**Prerequisites**

- Rust toolchain via [rustup](https://rustup.rs/)
- Cross-compilation targets:

  ```sh
  rustup target add x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu
  ```

- Linux cross-linker on macOS (see [README.md](README.md#compiling-from-mac-to-linux)):

  ```sh
  brew tap SergioBenitez/osxct
  brew install x86_64-unknown-linux-gnu
  ```

If the Linux build fails with `x86_64-linux-gnu-gcc` not found, use `./build <version> --skip-linux`, install the cross toolchain from [README.md](README.md#compiling-from-mac-to-linux) and re-run without the flag, or build the Linux tarball on a Linux machine / in CI.

If you only need macOS artifacts locally, you can build individual targets:

```sh
cargo build --target x86_64-apple-darwin --release
cargo build --target aarch64-apple-darwin --release
```

Then create tarballs manually from `target/<triple>/release/git-b`.

### 3. Tag the release in git

After the version bump is committed:

```sh
git tag 1.3.0
git push origin 1.3.0
```

Tag names must match the `Cargo.toml` version exactly (no `v` prefix). This matches existing tags (`1.0.0`, `1.1.0`, `1.2.0`) and the Homebrew formula download URLs (`releases/download/<version>/...`).

### 4. Publish GitHub release

1. Open [GitHub Releases](https://github.com/jharrilim/git-b/releases) and create a release for tag `1.3.0`.
2. Upload the three tarballs produced by `./build`.
3. Optionally attach release notes (dependency upgrades, `-b` flag, direct checkout, etc.).

### 5. Update Homebrew

The main repo’s `Formula/git-b.rb` is updated by `./build` (version + macOS checksums). The tap users install from is a separate repo:

```sh
cd Formula   # submodule → homebrew-git-b
git add git-b.rb
git commit -m "git-b 1.3.0"
git push
```

Homebrew users refresh with:

```sh
brew update
brew upgrade git-b
```

### Quick checklist

- [ ] `Cargo.toml` version bumped
- [ ] `./build <version>` succeeded; tarballs in repo root
- [ ] `Formula/git-b.rb` version and SHA256 values updated
- [ ] Changes committed on `main`
- [ ] Git tag `<version>` created and pushed (no `v` prefix)
- [ ] GitHub release created with tarball assets
- [ ] `homebrew-git-b` tap updated and pushed
