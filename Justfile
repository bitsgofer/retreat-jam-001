[private]
default: help-list

# List all recipes
[private]
help-list:
    just --list --unsorted

# ##############################################################################
# CONVENTION ###################################################################
# ##############################################################################

set lazy
# ------------------------------------------------------------------------------

repoRoot := invocation_directory() # OR: env("PWD", shell("pwd"))

# ##############################################################################
# dev ##########################################################################
# ##############################################################################

# Enter Nix development environment
[group("dev")]
[private]
dev:
    nix develop

# Build: game
[group("dev")]
build:
    cargo build --features dev


# Run: dev build (uses dynamic linking + asset hot-reload)
[group("dev")]
run:
    cargo run -p gamejam-001 --features dev

# ------------------------------------------------------------------------------
# fmt --------------------------------------------------------------------------
# ------------------------------------------------------------------------------

# Format: *.rs
[group("dev")]
fmt:
    cargo fmt

# Format: everything
[group("dev")]
fmt-all: fmt fmt-just fmt-nix

# Format: Justfile
[group("dev")]
[private]
fmt-just:
    just --fmt

# Format: *.nix
[group("dev")]
[private]
fmt-nix:
    find . -type f -name '*.nix' -o -name '*.nix' | xargs nixfmt

# ------------------------------------------------------------------------------
# lint -------------------------------------------------------------------------
# ------------------------------------------------------------------------------

# Lint: *.rs
[group('dev')]
lint:
    cargo fmt --check
    cargo clippy --workspace -- -D warnings

# Lint: everything
[group('dev')]
lint-all: lint lint-just lint-nix

# Lint: Justfile
[group('dev')]
[private]
lint-just:
    just --unstable --fmt --check

# Lint: *.nix
[group('dev')]
[private]
lint-nix:
    find . -type f -name '*.nix' -o -name '*.nix' | xargs nixfmt --check
