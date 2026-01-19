# git_copyright

Extract added/last modified times from git history and add/update copyright notes accordingly.

## Installation

The easiest way to install `git_copyright` is via `cargo` from `crates.io`:

```bash
cargo install --locked git_copyright
```

If you want to build it from source, clone the repository and then run:

```bash
cargo build --release
```

## Running

There are no required arguments,
but you probably want to set the copyright template
to something matching what your company uses,
.e.g:

```bash
git_copyright --copyright-template "Copyright {years} YourCorp. All rights reserved."
```

### Run with Docker

You can also use a pre-built image:

```bash
docker run --rm -u $(id -u) -v $(pwd):/mnt sgasse/git_copyright:0.3.2 --copyright-template "Copyright {years} YourCorp. All rights reserved."
```