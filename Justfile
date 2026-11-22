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
program := "gamejam-001"

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
    cargo run -p {{program}} --features dev

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

# ##############################################################################
# release ######################################################################
# ##############################################################################

# Build: release program (optimized, no dev features)
[group("release")]
build-release:
    cargo build --release -p {{program}}

# Package: Linux build
[group("release")]
package: build-release
    rm -rf dist/{{program}}-linux
    mkdir -p dist/{{program}}-linux
    cp target/release/{{program}} dist/{{program}}-linux/
    # Update the ELF program, using system paths instead of Nix-based ones:
    # - Use system `ld` as ELF interpreter
    # - Use system libs (wayland, alsa, udev libgcc_s, libc, etc)
    #
    # NOTE: Vulkan/Mesa have their libs called in dlopen() at runtime instead.
    # => unaffected by this.
    patchelf \
        --set-interpreter /lib64/ld-linux-x86-64.so.2 \
         --remove-rpath \
        dist/{{program}}-linux/{{program}}
    # Copy asssets
    cp -r assets dist/{{program}}-linux/
    rm -f dist/{{program}}-linux/assets/*.aseprite
    # Zip
    cd dist/{{program}}-linux && zip -rq ../{{program}}-linux.zip .

# Run: packaged Linux build from a temp dir, in a clean (non-Nix) environment
[group("release")]
[script("bash")]
run-release: package
    set -euo pipefail
    set -x
    dir=$(mktemp -d /tmp/{{program}}-linux.XXXXXX)
    # Extract release to tmp dir
    unzip -q dist/{{program}}-linux.zip -d "$dir"
    # Wipe the environment (no Nix PATH, LD_LIBRARY_PATH, CARGO_MANIFEST_DIR, ...),
    # keeping only what's needed to reach the display and audio servers.
    env -i \
        HOME="$HOME" \
        USER="$USER" \
        PATH=/usr/local/bin:/usr/bin:/bin \
        bash --noprofile --norc -c '
            cd "$1"
            echo "--- missing shared libs (ldd) ---"
            ldd ./{{program}} | grep "not found" || echo "(none)"
            echo "--- running ---"
            ./{{program}}
        ' _ "$dir"

# Build: wasm32 release + JS bindings for the browser
[group("release")]
build-web-release:
    cargo build --release --target wasm32-unknown-unknown -p {{program}}
    rm -rf dist/{{program}}-web
    mkdir -p dist/{{program}}-web
    wasm-bindgen \
        --no-typescript \
        --target web \
        --out-dir dist/{{program}}-web \
        --out-name game \
        target/wasm32-unknown-unknown/release/{{program}}.wasm
    # Optimize game size for itch.io's upload
    wasm-opt --all-features -Oz -o dist/{{program}}-web/game_bg.wasm dist/{{program}}-web/game_bg.wasm

# Package: browser build for itch.io
[group("release")]
package-web: build-web-release
    cp web/index.html dist/{{program}}-web/
    cp -r assets dist/{{program}}-web/
    rm -f dist/{{program}}-web/assets/*.aseprite
    cd dist/{{program}}-web && zip -rq ../{{program}}-web.zip .

# Serve: browser build at http://localhost:<port>
[group("release")]
run-web-release port="8080": package-web
    miniserve --port {{port}} dist/{{program}}-web
