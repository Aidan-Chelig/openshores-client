set shell := ["bash", "-euo", "pipefail", "-c"]

# List available development tasks.
default:
    @just --list

# Inspect assets without extraction; installation files are never executed.
assets installation="reference/hazeron": build
    nix develop .#server-test --command target/debug/openshores-client inspect-mount {{quote(installation)}}

# Extract/convert user assets into a cache outside the supplied installation.
prepare-assets installation cache: build
    nix develop .#server-test --command target/debug/openshores-client prepare-assets {{quote(installation)}} {{quote(cache)}}

# Build the headless tools.
build:
    nix develop .#server-test --command cargo build --offline

# Build the graphical client.
build-client:
    nix develop .#server-test --command cargo build --offline --features client --bin openshores-client-slice

# Disposable server + client, automatic test-account login and avatar selection.
quick-dev: build-client
    nix develop .#server-test --command python tests/local_server.py quick-dev

# Disposable server + client, manual login.
play: build-client
    nix develop .#server-test --command python tests/local_server.py play

# Open the client against an already-running loopback server.
client: build-client
    nix develop .#server-test --command target/debug/openshores-client-slice

fmt:
    nix develop .#server-test --command cargo fmt --all

fmt-check:
    nix develop .#server-test --command cargo fmt --all -- --check

test:
    nix develop .#server-test --command cargo test --offline

test-client:
    nix develop .#server-test --command cargo test --offline --features client

lint:
    nix develop .#server-test --command cargo clippy --offline --features client --all-targets -- -D warnings

check: fmt-check test test-client lint

# Compare codecs with the supplied reference source, without a server.
interop: build
    nix develop .#server-test --command bash -euc 'for test in tests/reference_*.py; do python "$test"; done'

# Static patched-loader anchors + independent synthetic NT codec parity.
test-native-models: build
    nix develop .#server-test --command python tests/reference_native_models.py

# Packed position fixtures and static patched-client anchors; no target execution.
test-native-transforms: build
    nix develop .#server-test --command python tests/native_atom_positions.py
    nix develop .#server-test --command python tests/native_transform_vtables.py

# Patched item/subtype reader anchors and independent fixtures; no target execution.
test-native-items: build
    nix develop .#server-test --command python tests/native_items.py

# Full GUI inventory/world workflow against a disposable local server.
test-gui: build-client
    nix develop .#server-test --command python tests/local_server.py gui

# Actual protocol/conversation logic with a synthetic nearby native fixture.
test-conversation: build-client
    nix develop .#server-test --command python tests/local_server.py conversation

# Exercise automatic login without the inventory-test script.
test-quick-dev: build-client
    nix develop .#server-test --command python tests/local_server.py quick-smoke

# A specific isolated protocol workflow (move, drop, pickup, use, pose, peer, etc.).
test-server mode="move": build
    nix develop .#server-test --command python tests/local_server.py {{quote(mode)}}

# All isolated gameplay workflows; each creates and cleans up its own database.
test-gameplay: build
    nix develop .#server-test --command bash -euc 'for mode in move drop pickup position pose peer use use-noop; do python tests/local_server.py "$mode"; done'
