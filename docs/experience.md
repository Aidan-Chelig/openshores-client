# Experience fidelity

The Rust client should preserve the OpenShores experience. Bevy is the preferred
renderer and now powers an optional offline geometry inspector; GPUI is a
possible UI layer, not a committed dependency.
Keep protocol, assets, and scene state independent of either framework so the
renderer/UI choice does not dictate game behavior.

Faithful shading is deferred: modern lighting may be a welcome visual change.
This flexibility does not relax fidelity requirements for gameplay, protocol,
screen flow or interactions. Original material semantics remain an evidence
question, independent of the inspector's current lighting choices.

## Current development priority

The goal is a Rust OpenShores client, not a standalone terrain/model inspector.
The existing inspector is sufficient for asset and terrain debugging; defer
additional viewer polish unless it directly unblocks reconstruction work.
Next prioritize decoding player/entity state, ongoing scene/session updates and
gameplay message interoperability against the supplied local reference server.
Then connect verified state to the game view and reconstruct interaction/screen
flow from evidence. A synthetic globe demo is not a playable-world milestone.
Never test against the public OpenShores service.

Before building the graphical interface, establish the original screen flow,
window/panel behavior, menus, keyboard shortcuts, camera controls, selection,
and feedback through reference evidence. Record observed behavior alongside
each implementation. The current CLI is an interoperability tool, not the
intended game interface.

The `viewer` feature's model-inspection window is deliberately a diagnostic
tool, not the game interface. Orbit/zoom/wireframe/reset controls, neutral
shading, display normalization and Z-up camera are implementation choices for
inspection. They establish no original camera feel, UI layout, lighting,
material rules or rendering axes. Keep these defaults scoped to the inspector
when reconstructing the actual experience. Core codecs/state remain Bevy-free.

Preserve the six avatar slots and their identities, including empty positions.
Do not infer the selected avatar from a roster index or automatically pick the
first populated slot. Existing protocol fields remain available to the future
UI, including avatar DNA, sex, handedness, initialization text, and scene time.

Evaluate GPUI after a representative original panel and a Bevy scene can be
reconstructed. The evaluation must cover focus, text input, keyboard routing,
pointer capture, scaling, and compositing with the game view. Its inclusion is
conditional on supporting the original interaction model.

Current unknowns: the original login and avatar-screen appearance, window
layout, shortcuts, camera feel, and original asset transform/material/animation
semantics. Raw model geometry is decoded, but these experience details have not
yet been observed at runtime. The reference server establishes protocol expectations,
but cannot establish how the original client presents them.
