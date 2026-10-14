# User-supplied assets

The client requires a user-provided Hazeron installation. No Hazeron assets are
bundled or downloaded. The `reference/` development fixture remains gitignored.
Choose its directory with **Choose Hazeron installation…** in the launch window;
selection starts preparation automatically. A manual path field and **Prepare
installation assets** button remain available if the desktop file dialog is unavailable.
Linux uses the desktop's XDG file-selection portal.
`HAZERON_INSTALL` can preselect a directory; `just quick-dev` defaults to the
supplied `reference/hazeron` and automatically prepares it before login.

Preparation runs on a worker thread. It reads the installation without executing
or dynamically loading any of its programs or libraries. It extracts byte-exact
resources into a content-addressed user-local cache. The default cache is
`$XDG_CACHE_HOME/openshores-client/assets-v1`, falling back to
`$HOME/.cache/openshores-client/assets-v1`; `OPENSHORES_ASSET_CACHE` overrides it.
Cache directories inside the installation are rejected **before creating them**.
The test harness uses a disposable cache, removed with its local world.

Keys are explicit and case-sensitive:

- `loose/tex/Exo.png` (and other non-executable loose files).
- `har/sfx.har/Chat2.wav` for sequential HAR audio.
- `qt/AuModelRes13.dll/POOL/COUNTRY/LANGUAGE/data/NAME` for embedded resources.

Qualified identities preserve duplicate basenames, pools and locale variants;
the mount does not guess original override precedence or locale selection.
Server catalog Qt resource paths can now be resolved directly, accepting both
`:data/NAME` and `:/data/NAME`. Lookup uses the complete path, never basename
matching. Byte-identical aliases are accepted; conflicting module/pool/locale
variants produce an explicit ambiguity error. Flag, Hay and Torch's catalog
model paths resolve and decode against the supplied pre-patch installation.
Use `cargo run --offline -- inspect-mounted-model INSTALLATION :data/mFlag.3dsX`
to check a binding. This establishes resource binding, not scene orientation,
material application or original rendering parity.
Symlinks are skipped in loose-file indexing and resolved reads must stay within
the selected installation. File, archive, traversal and backing-memory budgets
apply. Native framework DLLs are not asset sources; the game's Au*13 DLLs and
two game executables are inspected for supported Qt registrations.

Supported model geometry is converted to raw-axis OBJ for development. This is
not faithful renderer conversion: UVs, materials, source transforms, animation
and unknown extensions are not represented by OBJ. All original model bytes
are retained, and unrecognized variants are listed in the cache manifest rather
than guessed or discarded. Scene markers and terrain are still prototypes;
player bodies still do not use original meshes.
The connected scene now replaces supported world-item markers with catalog-bound
model geometry from the selected installation. It reconstructs the native mesh
coordinate stages before applying globe display scale, without fitting models
to an arbitrary bounding box. Matrix/vertex chunk order is retained: X scales
existing vertices and matrix translation by 1/12 when loading a matrix; a
nonidentity matrix moves vertices into its local frame with float storage,
then the triangle builder projects them back. See `docs/native-models.md` for
the static evidence, independent numerical checks and remaining limitations.
A 64-entry least-recently-used mesh cache is invalidated by installation preparation
or globe scale changes. Missing/ambiguous/unsupported resources retain primitive
markers and report diagnostics. This initial pass uses temporary flat colors
and generated smooth normals. Loader caller scale is explicitly 1.0; master
scale is not guessed. Recognized helper meshes and rejected native objects are
excluded from connected draw geometry while remaining in raw inspections and
cache exports. Entity rotation, helper interaction poses, UV/material
assignment, textures and animation remain unresolved. Player
bodies remain primitive markers. It is not faithful model presentation yet.

The supplied copy indexes 4,882 assets, including 104 archived sounds, and
converts 2,224 model entries. `2LegClosedToLeg.3dsN` and `3Ear22.3dsN` in the
body-parts resource DLL do not have an exact supported 3DS root; they remain raw
and are recorded as opaque in the manifest. These counts describe this copy,
not a promised inventory for every Hazeron release.

## Evidence and supported profile

The supplied pre-patch `AuModelRes13.dll` has SHA-256
`32404b94bd4053c877a9c9dba3d791b6e0a975d6b0265e2fcc80dc9a6609e784`.
Static Ghidra/REA evidence (no target execution):

- `ev_8fc699e52cf5da0f49af4a03b4fbc9bd5dd095462abe03b90f137a024b984237`:
  its constructor calls seven registration helpers.
- `ev_8b8f29fed0ab3125e746339be49917d66dda3e925373b69af86725e1190d348f`:
  helper `0x1800012c0` sets version 2 and RIP-relative data/name/tree pointers.
- `ev_3241a50b02807fb1bd5dec65e7770728d9a4c9c11cc63877a57ced7674d41188`:
  the helper's call target is an import thunk for `qRegisterResourceData`.

The new reader supports that narrow MSVC x64 instruction layout and validates
the import identity; it does not heuristically scan arbitrary data for trees.
Unsupported compiler/platform/stripped layouts produce an explicit error.
The existing symbol-based Qt reader and indexed HAR reader remain separate.
The pre-patch HAR has a different sequential count/QString/QByteArray layout,
independently checked by `tests/reference_mount.py` against zlib and struct.

Run `just interop` for source/container parity and cache checks, or
`just prepare-assets INSTALLATION CACHE_DIRECTORY` to prepare a chosen copy.
