# OpenShores Rust client

Rust reimplementation of the OpenShores client. The supplied Python server is
the first interoperability target. This repository currently contains a working
reference inventory, transport primitives, an interactive login command, and
scene bootstrap codecs and a loopback-only scene probe. Login displays the
six-slot avatar roster returned by the server. The probe completes world
initialization and reconstructs the first manifest's entity hierarchy;
rendering is still unfinished.

```sh
nix develop
cargo run -- inspect-reference
cargo run -- inspect-gd
cargo run -- login YOUR_USERNAME
cargo test
```

See [the reconstruction plan](docs/reconstruction.md) for the evidence and milestones.

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
transforms, selected name prefixes, and world zone/orbit identity. Sparse
updates preserve omitted fields; manifests track visibility separately from
cached entities. The local integration test resolves all 49 manifest atoms and
the full player ancestry. Type-specific payload bytes remain available for
further decoding. Terrain, models, rotation conversion, and persistent
gameplay are still unfinished.

Bevy is the preferred future 3D renderer; GPUI remains a UI candidate. See
[experience fidelity](docs/experience.md) for the current direction.
