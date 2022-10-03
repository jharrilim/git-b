# git b

Fuzzy selector for git branches

## Install (Mac)

```sh
brew tap jharrilim/git-b
brew install git-b
```

## Configure git

```sh
git config --global alias.b '!git-b'
```

## Compiling from Mac to Linux

Due to [rust/issues/34282](https://github.com/rust-lang/rust/issues/34282), you'll need to run this
before running the `build` script:

```sh
brew tap SergioBenitez/osxct
brew install x86_64-unknown-linux-gnu
```
