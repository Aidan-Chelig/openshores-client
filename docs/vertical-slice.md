# Local client vertical slice

The active milestone is a connected Rust client, not more inspector tooling.
The slice must support this complete loop against the supplied loopback server:

1. Enter credentials, receive the six-slot roster and explicitly select an avatar.
2. Keep the authenticated scene/chat channels alive while rendering the received
   home globe and visible player/item markers.
3. Show server-received player state and inventory with commodity labels.
4. Select a visible item, request pickup, move/drop/use a gear entry and observe
   server-originated state changes in the same session.
5. Report raw position changes through development controls, and display received
   peer transforms without pretending the sender receives a transform echo.
6. Surface connection/action errors, disconnect cleanly and verify the full loop
   with synthetic accounts and disposable world data, including a renderer smoke
   test. Never contact the public service or launch the original executable.

This is explicitly a development slice, not a claim of original UI/camera parity.
Protocol flow, six-slot selection and gameplay requests are source-established.
The original login layout, shortcuts, camera feel and animation meanings remain
unknown; temporary presentation/controls must say so. The REA skill guided the
evidence check: `libAuClientRes.dll` has only a PE version resource (Evidence
`ev_e6f88810f4e1496feb424066c9bf6477617c94fcc81f389fb87a94a5183855ca`).
Its two Qt resource registrations pass version 3 in static disassembly; the
663-entry inventory contains login/inventory/slot images but no `.ui`, HTML, XML
or text screen definitions. Asset names do not establish layout or control flow.
No original client runtime was executed. Faithful shading remains deferred.

The user supplied `reference/Controls - Hazeron Wiki.html` during this work.
Its Character Keys, Input Modes and User Interface Keys sections establish
W/S forward/back, A/D strafe, C/Z ascend/descend, Shift+W run, U/middle-click
use, mouse-wheel/{/} gear selection, Alt+digits slot mapping and digits recall.
Right-click toggles mouse-look/hand-cursor, Ctrl temporarily enables the cursor,
F4 toggles Gear, F8 toggles Bio, F1 toggles windows, Shift+Esc toggles HUD, and
Backspace toggles first-person/top-down. Implemented bindings must retain these
keys; unrelated inspector shortcuts do not apply to the client slice. Enter
currently enables hand-cursor, but chat entry is outside this inventory slice.
Unsupported controls (stations, weapons, unit orders, etc.) are not reassigned.

Static person-default construction in `OpenShores.exe` corroborates the character
bindings. `tests/reference_shortcuts.py` reads the symbol-bound initializer,
constructor arguments and PE-backed labels without executing code. Its scope is
67 calls of one constructor overload, not a complete control configuration;
source SHA-256 is `285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Drop Item is unbound by default (`QtKey=0x01ffffff`), so no invented drop hotkey
is added. The HTML also confirms Alt+digit mappings refer to inventory positions,
not item identity: consuming/moving an item must not silently change its mapping.

Keep networking/game state independent of Bevy. A minimal Bevy UI is a temporary
composition choice; GPUI remains undecided. Resource labels are not executable
paths, and opaque protocol data stays available.

## Completed connected loop

The optional `client` feature adds a standalone `openshores-client-slice` binary.
It has editable masked login fields, explicit six-slot roster selection, a live
home-globe mesh, source-state Gear/Bio panels, visible world-item selection and
move/drop/pickup/use actions. `T` selects source-flagged food from gear without
inventing nutrition calculations. Alt+digit mappings retain slot/subindex
positions, including when the item at that position is moved or consumed.

`ClientRuntime` owns the authenticated scene/chat channels on a dedicated thread,
with bounded request/event queues and a 60-second connected development-session
limit. `Session::receive_update` separates idle, clean close and partial-frame
errors, preserves inbound capture bounds and rejects avatar changes. The UI
never performs socket reads. `ClientModel` merges server state independently of
Bevy; successfully written coordinate reports are tracked separately from
received transforms, not fabricated as server echoes. Secrets are zeroized and
excluded from captures/status/debug output. Public endpoints and unauthenticated
avatar selections are rejected before scene connections.

The renderer uses base terrain, catalog-bound item geometry from the user
installation, and primitive player/fallback markers. Geology, roads, flora,
body meshes, textures, native entity orientation, animation and collision
remain outside this slice. Item meshes now use the reconstructed native
matrix/vertex loading sequence before globe display scaling (see
`docs/native-models.md`); caller scale is 1.0. Native object acceptance and
weapon/station/screen/up-vector helper filtering are now implemented without
discarding raw helper metadata. Station/weapon interaction poses remain pending.
W/A/S/D and C/Z use a prototype radial basis, with
prototype speed and camera sensitivity. Mouse-look changes the local camera;
wire rotation convention is not guessed. The default view is first person and
Backspace toggles top-down; [/] adjusts tactical height. The synthetic GUI test
avatar is seeded above the reference base surface. No original-physics or
original-art/appearance parity is claimed. Unsupported station/weapon/unit-order
shortcuts are reserved, not mapped to unrelated debug actions.

The UI is split into small modules for actions, controls, panels, scene rendering
and the opt-in integration scenario. CLI dispatch/login/tests now live under
`src/cli/`; scene-session code/tests are separated under `src/local_scene/`.
Historical reconstruction notes naming the old flat files refer to these moved
implementations, not deleted behavior.

## Verification and reproduction

The actual reference-server GUI integration passed the complete sequence:
authenticate, explicitly select the supplied avatar, receive 615 commodity rows,
render terrain, move Flag from 5/0 to 1/0, drop it, select its visible world ID,
pick it up, consume only Beans at 5/3, observe increased hunger with the other
Beans entry intact, report a position change, capture a rendered screenshot and
disconnect. A read-only query of the disposable database verifies persisted
position, increased hunger and offline status. The screenshot was inspected;
the generated globe and live inventory panel are visible. No public server or
original executable was contacted/executed.

All 80 headless and 91 client-feature Rust tests passed, including shortcut
modifier exclusions, hand-cursor behavior, position-based hotbar mappings,
partial-frame handling and unauthorized joins. Client Clippy/format/diff checks
passed. The independent Python comparison verifies all 615 commodity strings and
raw flags; the immutable native-default shortcut audit also passed.

```sh
cargo build --offline
cargo build --offline --features client --bin openshores-client-slice
nix develop .#server-test --command python tests/local_server.py gui
```

Use `play` instead of `gui` for manual interaction with the disposable server;
the harness prints only its known synthetic test credentials and cleans up when
the window closes. For a retained new screenshot, set `GUI_SCREENSHOT` to a new
absolute PNG path when running the `gui` test. Existing screenshot files are
refused. The original headless CLI and viewer remain separately usable.
## Camera clearance

The development camera now clamps its eye above the rendered terrain mesh and
uses a near plane scaled from 0.05 world units instead of Bevy's default scene
distance. Prototype eye height is 1.7 world units; this is not reconstructed
creature anatomy. The correction is view-only: received avatar positions and
outgoing position reports remain unchanged. It is not avatar collision physics.

## Incoming chat

The runtime now polls the authenticated chat socket separately from scene
updates. Partial chat frames fail explicitly; idle reads are bounded. Chat
payloads never enter scene captures or the world-state decoder. A 100-message
inbox feeds a minimal received-message panel showing the latest eight entries.
Unimplemented AuComm types remain opaque and are counted by type.

The server's `build_chat_aucomm_v4` emits an opcode plus QByteArray with no
timestamp field, unlike its handler for client-originated chat. The incoming
codec therefore treats the two directions separately. Source-generated packet
parity covers encrypted/compressed Unicode messages and optional range/auxiliary
fields. The minimal composer now selects indexed channels, sends up to 256
characters with selectable Close/Hail/System/Sector/Galaxy/Universe range and
untargeted scope, and displays only
server broadcasts (no optimistic local echo). Text entry suspends movement;
Enter sends and Escape ends entry only while the composer is focused. The
unused outbound envelope header is zero in this development profile; native
header generation and full channel/scope controls remain unverified. Both
current and maximum range fields use the chosen range in this minimal UI.
The actual disposable-server GUI test sends Unicode Voice chat and verifies
the rebroadcast's authenticated sender ID and server-provided character name,
plus a second message's selected Universe range. Range labels and server
delivery behavior come from `CHAT_RANGE_NAMES` and `gameplay/chat_range.py`;
that server treats System and wider ranges as unrestricted here, so this is
not evidence of original-client distance or galaxy filtering parity.
NPC `0x2C` continued speech and `0x2B` choice offers are now decoded. Choice
buttons preserve the supplied index/story instance/speaker and send a bare
`0x7C` reply on chat. The worker only accepts outstanding choices offered to
the authenticated avatar; choices are bounded, cleared on disconnect and
retired after a successful write, without inventing a dialogue result.
The real Python native builders independently verify choice and long-text
decoding; synthetic socket tests verify reply bytes and channel routing.
The Hail button sends an untargeted Close-range Voice greeting with the
authenticated roster's character name. Incoming hail replies retain their
environment ID and show the greeting. Story speech (`0x2D`), narration (`0x4A`),
section titles (`0x4E`) and narrated choices (`0x4C`) are decoded separately and
shown in a minimal story panel. Narrated choices omit the ordinary choice's
trailing recipient field and can reply with offered-speaker zero; runtime
authorization still requires an actual offer for the selected avatar. Story
instance IDs and titles are preserved, with a bounded 32-entry story history.
Source-builder comparisons cover all four formats. Close/hint/resend buttons
send the source's bare `0x7D`/`0x7E`/`0x7F` chat controls only after a story or
narrated choice has been received for the selected avatar. Close hides the
story panel after the write without deleting its history; hint/resend do not
invent responses. Source-handler tests verify close cancels a pending choice
and hint/resend are accepted. The supplied hint handler currently only logs
the request. Story audio and a complete live tutorial remain pending. Runtime API,
worker and tests now live in separate module-folder files, as do model state
and its tests under `src/client_model/`.

`just test-conversation` verifies greeting, an authenticated offered choice,
and the next NPC line with the real source handlers and one synthetic nearby
native. **This is an adapted-server test:** the supplied native handler writes
bare AuComm payloads, unlike ordinary chat's `write_framed`. The unmodified
hail test failed at the transport boundary (`invalid server chat envelope`).
Only `tests/conversation_server.py` adapts native egress framing; no reference
source files are edited and production decoding remains strict. The fixture
does not prove native spawning, rendered NPC bodies, or unmodified-server NPC
compatibility. Those remain unfinished requirements.
