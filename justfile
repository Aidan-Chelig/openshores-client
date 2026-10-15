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

test-native-body-scale:
    nix develop .#server-test --command python tests/native_body_scale.py
    nix develop .#server-test --command cargo test --offline --lib model::body_scale

test-native-body-groups:
    nix develop .#server-test --command python tests/native_body_groups.py
    nix develop .#server-test --command cargo test --offline --lib model::body_groups

test-native-body-object-groups:
    nix develop .#server-test --command python tests/native_body_object_groups.py
    nix develop .#server-test --command cargo test --offline --lib model::body_object_groups

test-native-body-selection:
    nix develop .#server-test --command python tests/native_body_selection.py
    nix develop .#server-test --command cargo test --offline --lib model::body_selection

# Native job factory/vptr bindings and Qt7 datetime anchors; no target execution.
test-native-jobs:
    nix develop .#server-test --command python tests/native_jobs.py

# Native inventory factory/filter/duplicate insertion anchors; no target execution.
test-native-inventory:
    nix develop .#server-test --command python tests/native_inventory.py

# Proven receive-visible Person constructor defaults, without target execution.
test-native-person-construction:
    nix develop .#server-test --command python tests/native_person_construction.py

# Complete offline Person packets through native dispatch; explicit fixture context.
test-native-person-dispatch:
    nix develop .#server-test --command cargo test --offline --lib world::native_person::atom::tests::lifecycle::packets

# Native packet -> selected presentation -> actual Bevy marker/HUD components, headless.
test-native-avatar-presentation:
    nix develop .#server-test --command cargo test --offline --features client --lib complete_native_person_packets

# Independent native outer DNA-application anchors; no target execution.
test-native-dna-application:
    nix develop .#server-test --command python tests/native_dna_application.py

# Person virtual scalar binding and native head-angle operation anchors.
test-native-head-tilt:
    nix develop .#server-test --command python tests/native_head_tilt.py

# Native f32 movement modifier and growth/burden operation anchors.
test-native-movement:
    nix develop .#server-test --command python tests/native_movement.py

# Native model HP cache/volume/wing and growth-operation anchors.
test-native-health:
    nix develop .#server-test --command python tests/native_health.py

# Static chromosome-volume dispatch inventory, not model generation.
test-native-chromosome-volumes:
    nix develop .#server-test --command python tests/native_chromosome_volumes.py

# Static MakeParts factory and Wing CanFly bindings, not complete morphology.
test-native-model-parts:
    nix develop .#server-test --command python tests/native_model_parts.py

# Static native body dimension tables/constants, not complete model creation.
test-native-dp-dimensions:
    nix develop .#server-test --command python tests/native_dp_dimensions.py

# Static model-key selectors/comparator and PA hand override.
test-native-dp-model-keys:
    nix develop .#server-test --command python tests/native_dp_model_keys.py

# Native pose rhythm constants, classification and timer-order fixtures.
test-native-pose-rhythm:
    nix develop .#server-test --command python tests/native_pose_rhythm.py
    nix develop .#server-test --command cargo test --offline --lib pose_rhythm

test-native-dp-height:
    nix develop .#server-test --command python tests/native_dp_height.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::height

test-native-dp-connections:
    nix develop .#server-test --command python tests/native_dp_connections.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::connection

test-native-dp-neck:
    nix develop .#server-test --command python tests/native_dp_neck.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::neck

test-native-dp-head-pose:
    nix develop .#server-test --command python tests/native_dp_head_pose.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::head_pose

test-native-dp-position-heads:
    nix develop .#server-test --command python tests/native_dp_position_heads.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::position_heads

test-native-dp-tail-bend:
    nix develop .#server-test --command python tests/native_dp_tail_bend.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::tail_bend

test-native-dp-tail-slither:
    nix develop .#server-test --command python tests/native_dp_tail_slither.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::tail_slither

test-native-dp-tail-slither-pose:
    nix develop .#server-test --command python tests/native_dp_tail_slither_pose.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::tail_slither_pose

test-native-dp-tail-fold:
    nix develop .#server-test --command python tests/native_dp_tail_fold.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::tail_fold

test-native-dp-tail-motion:
    nix develop .#server-test --command python tests/native_dp_tail_motion.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::tail_motion

test-native-dp-position-tail:
    nix develop .#server-test --command python tests/native_dp_position_tail.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::tail_special

test-native-dp-waist:
    nix develop .#server-test --command python tests/native_dp_waist.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::waist

test-native-dp-body-waist:
    nix develop .#server-test --command python tests/native_dp_body_waist.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::arm_move

test-native-dp-torso-base:
    nix develop .#server-test --command python tests/native_dp_torso_base.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::torso_base

test-native-dp-position-torso:
    nix develop .#server-test --command python tests/native_dp_position_torso.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::position_torso

test-native-dp-arm-shoulder:
    nix develop .#server-test --command python tests/native_dp_arm_shoulder.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::arm_shoulder

test-native-dp-arm-elbow:
    nix develop .#server-test --command python tests/native_dp_arm_elbow.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::arm_elbow

test-native-dp-arm-wrist:
    nix develop .#server-test --command python tests/native_dp_arm_wrist.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::arm_wrist

test-native-dp-wing-bow:
    nix develop .#server-test --command python tests/native_dp_wing_bow.py
    nix develop .#server-test --command cargo test --offline --lib native_body::dp::wing_bow

# Mounted hands, feet, part calculations and standing heights; entirely offline.
test-native-dp-mounted installation:
    nix develop .#server-test --command env OPENSHORES_HAZERON_INSTALLATION={{quote(installation)}} cargo test --offline --test native_dp_mounted_hand -- --ignored --nocapture
    nix develop .#server-test --command env OPENSHORES_HAZERON_INSTALLATION={{quote(installation)}} cargo test --offline --test native_dp_mounted_tail_pose -- --ignored --nocapture
    nix develop .#server-test --command env OPENSHORES_HAZERON_INSTALLATION={{quote(installation)}} cargo test --offline --test native_dp_mounted_position_tail -- --ignored --nocapture
    nix develop .#server-test --command env OPENSHORES_HAZERON_INSTALLATION={{quote(installation)}} cargo test --offline --test native_dp_mounted_waist -- --ignored --nocapture
    nix develop .#server-test --command env OPENSHORES_HAZERON_INSTALLATION={{quote(installation)}} cargo test --offline --test native_dp_mounted_position_torso -- --ignored --nocapture
    nix develop .#server-test --command env OPENSHORES_HAZERON_INSTALLATION={{quote(installation)}} cargo test --offline --test native_dp_mounted_arm_elbow -- --ignored --nocapture
    nix develop .#server-test --command env OPENSHORES_HAZERON_INSTALLATION={{quote(installation)}} cargo test --offline --test native_body_mounted_groups -- --ignored --nocapture

# Packed position fixtures and static patched-client anchors; no target execution.
test-native-transforms: build
    nix develop .#server-test --command python tests/native_atom_positions.py
    nix develop .#server-test --command python tests/native_transform_vtables.py
    nix develop .#server-test --command python tests/native_rotations.py

# Patched item/subtype reader anchors and independent fixtures; no target execution.
test-native-items: build
    nix develop .#server-test --command python tests/native_items.py
    nix develop .#server-test --command python tests/native_item_captures.py

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
