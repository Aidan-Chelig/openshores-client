# OpenShores Rust client

Rust reimplementation of the OpenShores client. The supplied Python server is
the first interoperability target. This repository currently contains a working
reference inventory, transport primitives, an interactive login command, and
scene bootstrap codecs and a loopback-only scene probe. Login displays the
six-slot avatar roster returned by the server. The probe completes world
initialization and reconstructs the first manifest's entity hierarchy;
rendering is still unfinished.

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
complete. Incomplete frames after that grace and malformed updates are errors. Gameplay
request handling and an interactive game session remain future core milestones.

World-state replay also recognizes dropped-item atoms (`DaItem`, `0x11`), their
parent/transforms, subtype identity and supported commodity/name/quality fields.
Unknown item variants and weapon/container tails remain available as raw bytes;
pickup/drop requests and item rendering are not implemented.
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
The raw-coordinate decoder supports `.3ds`, `.3dsN`, and `.3dsX`; it does not
yet establish the original rendered appearance:

```sh
cargo run -- inspect-model extracted-model.3dsN
cargo run -- verify-models reference/openshores-full/libAuModelRes.dll 3
```

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
