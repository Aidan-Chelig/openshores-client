# OpenShores Rust client

Rust reimplementation of the OpenShores client. The supplied Python server is
the first interoperability target. A connected local vertical slice now combines
six-slot login/avatar selection, live scene state, Bevy base terrain and primitive
entity markers, Gear/Bio panels, and verified inventory actions. Original UI,
animation, collision and asset presentation remain unfinished.

The patched executable's native atom opcodes/layout differ from the supplied
server writers. The GUI currently uses that reference-server profile; native
dispatch, packed prefixes and position-state primitives are being ported
separately, not auto-detected or substituted into local sessions. See the
[native protocol/transform evidence and remaining work](docs/native-entity-transforms.md).

Assets come from a user-supplied Hazeron installation. On launch, choose its folder
with **Choose Hazeron installation…**; extraction and geometry
conversion run in the background and write only to a user-local cache, never
to the installation or repository. `HAZERON_INSTALL` preselects a folder.
See [asset mounting, cache and current conversion scope](docs/assets.md).

For the offline native avatar diagnostic, run `just preview-avatar body`.
The body preview includes a modern render-only wrist/ankle seam repair; source
assets and native geometry remain unchanged. Add `--native-seams` to the
`view-native-body INSTALLATION body` command for the unrepaired comparison.
`just preview-avatar body reference/hazeron 60 many side` selects the second
anatomy and side view; views are `oblique`, `front`, `side`, and `back`.
`just preview-avatar-untextured body` checks the same posed geometry without
skin. Existing `head` and `torso` inspection remains available. This is fixture
DNA, not received player appearance or in-game avatar integration. Apparent
extremity gaps and torso surface fidelity remain under investigation; see the
[assembly evidence and verification ledger](docs/native-avatar-assembly.md).

Try the complete slice in an owned disposable world:

```sh
just quick-dev
```

This builds the client, starts the supplied server with a disposable database,
and logs into **rust-local-test**, selecting **Rust Explorer** automatically.
It contacts only loopback addresses and performs no scripted inventory actions.
Passwords are supplied through stdin, not command-line arguments.
Quick-dev also prepares the supplied `reference/hazeron` assets automatically;
set `HAZERON_INSTALL` to use another copy. Its cache is disposable like its world.
Use `just play` instead for manual login; that harness prints synthetic credentials.
Right-click switches mouse-look/hand-cursor; Ctrl gives
a temporary cursor. Select a gear entry, move/drop it, select the dropped world
item and pick it back up. Use Beans to observe server-updated hunger and inventory.
The connected development session is currently bounded to 60 seconds; closing
the window shuts down the owned server and disposable database.

`just` lists tasks. Use `just check` for formatting, both Rust test configurations
and linting; `just interop` for reference parity tests; `just test-gui` for the
full GUI workflow; `just test-quick-dev` for automatic-login verification; and
`just test-server drop` (or `just test-gameplay`) for isolated gameplay tests.
Recipes enter the appropriate Nix environment themselves. If `just` is not on
your PATH, start with `nix develop .#server-test`, which includes it and the
graphics libraries needed on NixOS.

Implemented shortcuts follow the supplied controls page: W/A/S/D, C/Z,
Shift+W, U/middle-click, T, wheel/{/}, Alt+digits/digits, F1/F4/F8, Shift+Esc,
Backspace, and top-down [/] height controls. Unsupported shortcuts are not
repurposed. Movement speed/camera tuning are prototype choices, not reconstructed
physics. Drop has no invented default shortcut. See
[slice scope and evidence](docs/vertical-slice.md).

For an already running local server:

```sh
cargo run --offline --features client --bin openshores-client-slice
```

Optional `IP:PORT`, `--scene-port PORT` and `--chat-port PORT` select loopback-only
endpoints. Networking stays off the render thread; credentials never enter scene
captures. The automatic GUI integration uses only synthetic disposable accounts:

```sh
nix develop .#server-test --command python tests/local_server.py gui
```

Client UI lives in small `src/client_ui/` modules; CLI logic and scene-session
tests are split under `src/cli/` and `src/local_scene/`. Core state/networking
remain independent of Bevy and GPUI.

Scene replay now decodes a source-established player-state prefix: raw DNA,
HP, pose bytes, head tilt, stamina/growth and supported hunger/agent deltas.
`inspect-scene CAPTURE_FILE` prints the merged player state. Omitted fields
retain earlier values; unknown layers and effect payloads remain opaque rather
than being guessed. Login with `--avatar` still stops after bootstrap by default.
An optional receive-only local session now stays connected and applies updates:

```sh
cargo run -- login YOUR_USERNAME --avatar NUMERIC_AVATAR_ID --watch-seconds 10 --capture new-session.frames
```

Use a real avatar ID from the local server's login roster, not the placeholder.
The watch runs 1–60 seconds after bootstrap, retains scene/chat channels and
captures bootstrap plus subsequent inbound scene packets. It sends no gameplay
requests and uses only loopback endpoints. Idle deadline and clean disconnect
are normal exits; frames begun before the boundary get at most 100 ms extra to
complete. Incomplete frames after that grace and malformed updates are errors.
The connected graphical slice uses the same wire/state foundation.

Two explicit local gameplay commands are now available during a watch:
`--look-at NUMERIC_ID` (zero clears) and `--move-gear SRC_SLOT SRC_SUB DST_SLOT`.
Both require `--avatar` and `--watch-seconds`; slot/subindex fields must be 0–15.
For example, with an actual avatar from your local roster:

```sh
cargo run -- login YOUR_USERNAME --avatar NUMERIC_AVATAR_ID --watch-seconds 5 --move-gear 5 0 1
```

This asks the server to move gear from slot 5/subindex 0 to slot 1. The server
chooses the destination subindex; no optimistic inventory changes are made.
The CLI reports the latest received gear batch, not a fabricated acknowledgement.
These are interoperability controls, not reconstructed game UI bindings.

Top-level gear drops are supported separately with `--drop-gear SRC_SLOT SRC_SUB`
(slot 1–9, subindex 0–15), also requiring an explicit avatar and bounded watch.
Drop requests use the authenticated chat channel; the resulting world item and
inventory removal are applied only from server updates. Move and drop flags
cannot be combined because cross-channel ordering is not guaranteed. Nested
container drops and reconstructed pickup controls remain unimplemented.

For a single-avatar local reference server, `--pickup-target NUMERIC_ITEM_ID`
adds a bounded pickup interoperability profile (requires `--watch-seconds 3..60`).
It sends four minimal scene-input target pins at 500 ms intervals, then clears
the pin, while still receiving updates. This tests the reference server's hold
trigger, not original mouse controls, input cadence or range behavior. It cannot
be combined with gear move/drop flags. Inventory and visibility are updated only
from server messages; cached item identity is not erased just because visibility
changes.

World-state replay also recognizes dropped-item atoms (`DaItem`, `0x11`), their
parent/transforms, subtype identity and supported commodity/name/quality fields.
Unknown item variants and weapon/container tails remain available as raw bytes;
faithful pickup controls, nested drops and original item rendering are not implemented;
the local slice currently uses primitive markers and explicit pickup buttons.
Supported gear-only player updates now expose slot/subindex identities, explicit
empty slots and nested item trees. These are the latest received gear batches,
not an inferred authoritative inventory snapshot. The supported citizen bootstrap
profile now also exposes initial gear, home fields, hunger/cursor and XP,
reputation and bank statistics, including the default story-job list.
Unknown office layouts and unsupported job forms
still stop safely with raw bytes retained; inventory actions remain unimplemented.

```sh
nix develop
cargo run -- inspect-reference
cargo run -- inspect-gd
cargo run -- login YOUR_USERNAME
cargo test
```

See [the reconstruction plan](docs/reconstruction.md) for the evidence and milestones.

The default Nix shell includes Ghidra and a full JDK 21, exporting
`GHIDRA_INSTALL_DIR` and `JAVA_HOME` for REA. Check the analysis provider with:

```sh
nix develop --command npx -y rea-agents@6.4.0 doctor --provider ghidra
```

Start REA's MCP server inside this shell as well; a server launched directly
with `npx` outside it will not inherit these tools or environment variables.
After changing an MCP launcher, restart/reconnect Codex to use the new process.

Inspect, verify, or extract the supplied DLL's Qt resources offline:

```sh
cargo run -- inspect-qt reference/openshores-full/libAuModelRes.dll 3
cargo run -- verify-qt reference/openshores-full/libAuModelRes.dll 3
cargo run -- extract-qt reference/openshores-full/libAuModelRes.dll 3 0 0 linen-shirt.png
nix develop --command python3 tests/reference_models.py
```

Version `3` is established from the model DLL's registration code. Extraction
uses the displayed pool and entry indices, refuses existing output files, and
does not execute the DLL. This initial reader requires the supplied MinGW
COFF symbol layout; stripped DLLs and zstd-compressed resources are unsupported.
The raw-coordinate decoder supports `.3ds`, `.3dsN`, `.3dsNT`, and `.3dsX`; it does not
yet establish the original rendered appearance:

```sh
cargo run -- inspect-model extracted-model.3dsN
cargo run -- verify-models reference/openshores-full/libAuModelRes.dll 3
```

To inspect reconstructed mesh coordinates independently of world placement:

```sh
cargo run --offline -- dump-model-coordinates extracted-model.3dsX 1.0
```

This emits numeric arrays in mesh order. The optional scale is the native
loader's positive caller scale, not planet display scale or the 3DS master
scale. Raw inspections/OBJ exports remain unchanged. Entity orientation,
materials and animation are still pending. Helper classification is inspectable
with `dump-model-roles MODEL_FILE`; `inspect-model-groups MODEL_FILE` prints a
separate raw-group parity fingerprint. Raw exports retain helper objects, while
the connected renderer filters recognized native helpers.

`verify-models` scans every supported model resource. Currently 209 decode and
five supplied `.3ds` payloads fail strict root-length checks, so this command
reports those failures and exits nonzero. `.SoH` resources remain unsupported.
The offline Python model test checks geometry parity and these expected failures.
It also checks raw material parity, including the client's nonstandard float
amount records. `inspect-model` prints colors, amounts, two-sided/shading fields,
and texture filenames without converting them into renderer materials.

For a visual geometry check, export an extracted model:

```sh
cargo run -- preview-model gin.3dsN gin-wireframe.svg
cargo run -- export-model gin.3dsN gin.obj
```

The SVG opens as an image and shows X/Y, X/Z, and Y/Z wireframes. The OBJ is
geometry-only for inspection in a compatible viewer. Both preserve raw axes,
omit original transforms/materials/normals/UVs/animation, and refuse to overwrite
existing outputs. These are debugging views, not previews of the final game's
appearance. `inspect-model` also reports actual UV counts and raw-coordinate
bounds. `tests/reference_models.py` verifies both exports for all 209 valid models.

For structural diagnostics before renderer integration:

```sh
cargo run -- audit-model gin.3dsN
```

This reports unused vertices, degenerate triangles, raw-index edge incidence,
material coverage/ambiguities, UV counts, and extension-record presence. It does
not weld seams, repair geometry, select fallback materials, or establish how the
original client renders those records. Audits are independently checked for all
618 decoded meshes.

Base terrain from a saved local scene can also be exported for inspection:

```sh
cargo run -- export-terrain scene.bin GLOBE_ID 32 new-terrain.obj
```

The latitude-band count must be 3–128; longitude sectors are twice that count.
This is a closed globe-local Z-up base-terrain mesh, not original terrain LOD.
Geology deformation, water, textures and scene transforms remain unapplied.
Existing output files are never overwritten.

To inspect that globe directly in the offline Bevy window:

No scene capture yet? Start with the synthetic test-globe demo:

```sh
cargo run --offline --features viewer -- view-terrain-demo
```

It uses fixed parity-test terrain parameters and 32 latitude bands, opens no
network connections, and accepts `--unlit` and `--smoke-frames 90`.
This is not a populated game world.

The demo defaults to 32 latitude bands. To increase detail or export its raw
geometry without a window or capture:

```sh
cargo run --offline --features viewer -- view-terrain-demo 64
cargo run --offline -- export-terrain-demo 64 demo-terrain.obj
```

Resolution is bounded to 3–128 bands; export refuses to overwrite existing files.
OBJ contains geometry only, not the inspector's generated colors or normals.

View/export commands print mesh counts, base radius and the sampled altitude
range. For those diagnostics without opening a window or creating a file:

```sh
cargo run --offline -- inspect-terrain-demo 64
```

The altitude limits are extrema of the sampled vertices, not guaranteed global
terrain extrema. Changing resolution can change the reported range.

For your own captured globe:

```sh
cargo run --offline --features viewer -- view-terrain scene.bin GLOBE_ID 32
```

This uses the same orbit, zoom, lighting and wireframe controls as the model
inspector. Display normalization does not change the exported/raw coordinates.
Terrain lighting uses area-weighted smooth vertex normals across the shared
seam and poles. Model previews retain flat normals; original model smoothing
semantics are not guessed.
Terrain also uses a diagnostic relative-elevation color ramp (low blue through
green/brown to high pale). Colors are relative to the sampled mesh, not sea
level or biome classification; blue does not represent a rendered water surface.
Right-click terrain to mark the nearest hit and print globe-local coordinates,
latitude/longitude and approximate altitude in the terminal. This probes the
rendered triangle mesh, not the exact terrain function; coarse triangles can
lie below the sampled surface. Clicking empty space clears the marker. Model
previews do not enable this terrain-only probe.

The optional Bevy model inspector opens an interactive 3D window:

```sh
nix develop
cargo run --offline --features viewer -- view-model gin.3dsN
```

Drag with the left mouse button to orbit, scroll to zoom, press `W` to toggle
wireframe, `L` to toggle lighting, `R` to reset the camera, and `Esc` to quit.
It centers/scales a display-only copy and defaults to modern lit, double-sided
geometry with generated flat normals. Append `--unlit` to start unlit. Original materials, textures,
transforms, normals, UVs and animation remain unapplied. These are inspection
controls, not reconstructed game controls. No login or server connection occurs.
Without `--features viewer`, the client remains headless and Bevy is not built.

For bounded graphics smoke checks, append `--smoke-frames 90`. This exits after
90 application updates, not a guarantee of 90 presented frames. Requires a
working local graphics driver/display; there is no silent headless fallback.

```sh
cargo test --offline --features viewer
cargo clippy --offline --features viewer --all-targets -- -D warnings
```

Bevy is pinned to 0.19.1 to retain this repo's Rust 1.95 toolchain compatibility.

`inspect-gd` defaults to the server's GD file and lists its 83 construction
processes. The full Windows client's GD uses a different layout and is rejected
by this initial reader.

Login defaults to `127.0.0.1:16757`. Select a different local port with
`cargo run -- login YOUR_USERNAME 127.0.0.1:26757`.
Passwords use a hidden terminal prompt. Automated callers can supply a single
UTF-8 line via stdin with `--password-stdin`; passwords are never command-line
arguments. Redirects are displayed and require an explicit reconnect.

Run the reference interoperability checks after compiling:

```sh
cargo build
python3 tests/reference_interop.py
```

Those checks use the supplied Python server's actual codecs on temporary
localhost listeners, with synthetic credentials and no database. They do not
establish authentication or gameplay against a fully running server.

For the full reference-server integration test:

```sh
cargo build
nix develop .#server-test --command python tests/local_server.py
```

This starts disposable PostgreSQL on a private Unix socket, generates a local
reference world and synthetic avatar, and runs the actual server source. It
verifies authentication, world redirect/resume, the chat initialization ACK,
the first scene manifest, and a replayable scene capture. Temporary data and
processes are cleaned up afterward; existing reference accounts/databases are
not used. All test client connections are loopback. Never test against the
public OpenShores service.

To probe an already running local development server, use the avatar ID shown
by login:

```sh
cargo run -- login YOUR_USERNAME --avatar 512 --capture scene.frames
```

Replace `512` with an ID in your roster. The probe follows only loopback world
redirects and stops after the first post-initialization manifest. Use
`--scene-port` and `--chat-port` when local ports differ from defaults. The
reference's initialization packet currently advertises the default chat port,
so an override is needed when running its chat listener on another port.

Capture files retain inbound scene payloads with regenerated length framing;
login traffic is excluded. Existing files are never overwritten. Replay a
capture without any network access:

```sh
cargo run -- inspect-scene scene.frames
```

The world model decodes atom identities, parents, optional timestamps,
transforms, selected name prefixes, world zone/orbit identity, and globe size,
atmosphere, water, and base terrain parameters. Sparse
updates preserve omitted fields; manifests track visibility separately from
cached entities. The local integration test resolves all 49 manifest atoms and
the full player ancestry. Type-specific payload bytes remain available for
further decoding. The base globe terrain sampler matches the Python reference
at 32 fixed coordinates and 56 coordinates across 14 captured globes. Geological
features, roads, models, rotation conversion, and persistent gameplay remain
unfinished.

Sample the base terrain of a captured globe offline (angles are degrees):

```sh
cargo run -- sample-terrain scene.frames 0x00294365 37.5 -122
```

Replace the example ID with a globe ID shown by `inspect-scene`. The sampler
preserves the reference's seven-octave noise, dateline blending, polar blending,
and Z-axis globe orientation. This is base terrain, before geological/road
modifiers, and does not claim original-client mesh or LOD parity.

The shipped `sfx.har` is readable as a named, compressed archive:

```sh
cargo run -- inspect-har reference/openshores-full/sfx.har
cargo run -- verify-har reference/openshores-full/sfx.har
cargo run -- extract-har reference/openshores-full/sfx.har Chat2.wav Chat2.wav
python3 tests/reference_assets.py
```

All 112 entries decompress successfully. The asset test compares the directory
and decompressed sizes against Python and checks three WAV extractions byte for
byte. Extraction requires an explicit new output file and never overwrites one.
This layout is established for the supplied sound archive; embedded model/texture
containers and blueprint formats are still under investigation.

Bevy is the preferred future 3D renderer; GPUI remains a UI candidate. See
[experience fidelity](docs/experience.md) for the current direction.

For a disposable local avatar, `login` also accepts
`--position-report X Y Z RX RY RZ` with `--avatar` and `--watch-seconds`.
These are raw wire coordinates/rotation, not camera angles or walking controls.

Values must be finite and fit in f32; pickup/move/drop combinations are rejected.
This local test profile sends two identical reports 500 ms apart, accommodating
the reference server's first-occurrence opcode logging branch.
The server broadcasts reports to other players, not the sender, so sending one
does not optimistically change the local scene. Test persistence in isolation:

```sh
cargo build --offline
nix develop .#server-test --command python tests/local_server.py position
nix develop .#server-test --command python tests/local_server.py peer
```

Patched-native item readers and evidence are documented in
[native item decoding](docs/native-items.md). `just test-native-items` checks
factory/reader anchors and independent offline fixtures. Replay established
item bodies with `cargo run --offline -- inspect-native-item-capture scene.bin`;
unknown kinds, subtypes and base layers remain opaque. This does not switch
the playable reference-server client to native protocol decoding.
`replay-native-items scene.bin` retains established item-field observations
across updates; unknown fields remain unknown, not guessed constructor defaults.

`--pose-report HEX20` reports ten raw pose bytes with auxiliary trigger fields
zero, using the same two-report local test schedule. It requires `--avatar` and
`--watch-seconds` and cannot combine with position/pickup/move/drop. It does not
assign animation names, head tilt or keyboard controls. The isolated `pose`
integration checks server-persisted pose and received stamina updates:

```sh
nix develop .#server-test --command python tests/local_server.py pose
```

Scene state now retains the server's commodity catalog, resolving received gear
IDs to names without guessed item labels. Inspect its nullable names,
descriptions, bitmap/model paths and any unknown row suffix offline:

```sh
cargo run --offline -- inspect-commodities scene.frames
nix develop .#server-test --command python tests/reference_commodities.py
```

The source comparison covers all 615 supplied commodity rows. Catalog rows are
not world entities; numeric gameplay properties remain raw and resource paths
are labels, not automatically opened files.

Top-level item use is available as `login ... --use-gear SRC_SLOT SRC_SUB`
with `--avatar` and `--watch-seconds 3..60`. Slots are 1..9 and subindices
0..15; nested paths are not supported. This local test profile sends one chat
request after one second while receiving scene updates. It cannot combine with
other actions except look-at. The server decides whether an item is consumed,
toggled or ignored; Rust does not optimistically change gear or hunger.

```sh
nix develop .#server-test --command python tests/local_server.py use
nix develop .#server-test --command python tests/local_server.py use-noop
```
