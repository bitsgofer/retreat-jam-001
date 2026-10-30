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
    cargo build


# Run: dev build (uses dynamic linking + asset hot-reload)
[group("dev")]
run:
    cargo run -p gamejam-001 --features dev
