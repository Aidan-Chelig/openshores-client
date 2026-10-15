# Native avatar presentation boundary

Successful `NativePersonDispatch` outcomes can now feed the minimal client's
actual presentation model and Bevy marker/HUD components. This is explicitly a
read-only diagnostic native preview, not completed native model rendering or a
native login/capture player. It does not claim anatomical meshes, animation or
world/environment placement are implemented.

`NativeAvatarPresentation::from_dispatched(&people, outcome)` checks the stable
handle, received wire ID and attached owner, then snapshots received native HP,
body kind, raw nullable Unit text, local position/parent and collection counts.
Call it immediately after a successful dispatch, never after a partial error.
Unobserved fields remain Option values; native Unit text is not renamed to an
unproven character-name field. HUD text is bounded without changing retained
wire strings. Native inventory/gear are not converted into source item structs.

`Event::NativeAvatar` enters `Phase::NativePreview`. This explicitly clears the
source world/roster/selected-avatar/report state; the native snapshot has its own
identity. Source-profile events are rejected before mutation while in this mode.
Existing source gameplay controls require Phase::Playing and remain disabled.
Disconnect clears the native snapshot and allows an explicit later source
session; no packet auto-detection or cross-profile fallback is introduced.

`client_ui::run_native_preview(snapshot, smoke_frames)` launches the same minimal
client UI with an offline runtime: no sockets, request channels or worker thread.
The marker is dimensionless and centered in a diagnostic frame; actual native
local coordinates are displayed as data, not reinterpreted as a globe transform.
The read-only panel shows native identity, HP/body and collection observations.
The marker is not a replacement for generating the user's mounted native body
mesh. Closing the window exits this offline view.

## Verification

`just test-native-avatar-presentation` starts with complete native Person packet
fixtures, goes through real dispatch/base/owner and all inherited readers,
constructs the presentation snapshot, applies the actual client event, and
runs the real Bevy marker and panel systems in a headless App. It verifies marker
identity, actual HUD Text content and marker removal on disconnect in all three
float profiles. It also checks retained timestamp/HP, candidate-disposal handle
rejection, wrong identity rejection and source-profile event rejection without
revision changes. This is ECS/component proof, not a screenshot or GPU fidelity
claim. The model/environment context in those packet fixtures remains explicitly
synthetic and rejects unimplemented operations.

The ordinary source-profile quick-dev path remains separate and is regression
tested against the disposable loopback reference server. `just test-quick-dev`
passed after these changes, including asset mounting and automatic synthetic
avatar selection. That
test does not prove native packet compatibility. Remaining goal requirements
include the actual production native runtime/session context, required model/
weight/animation operations, native scene/environment placement and real avatar
body presentation. No public servers are used for this work.
