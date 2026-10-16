# In-world avatar acceptance audit

Scope: [the in-world avatar goal](in-world-avatar-goal.md), not completion of the
OpenShores Rust port. Status: complete for this milestone. The
[progress ledger](in-world-avatar-progress.md) records implementation checkpoints
and individual failures/fixes; this audit maps the original requirements.

| Requirement | Implementation and authoritative verification |
| --- | --- |
| Actual selected player's received appearance | `ClientModel::local_player_dna` retains received bytes separately from normalized native DNA; `local_avatar_preparation` passes actual supplied-server DNA through normal preparation. Connected capture gates compare received bytes, attached content key, native height and origin. No fixture fallback exists in the connected renderer. |
| Asynchronous reusable preparation | `AvatarPreparation` keys DNA, mount revision, smoothing and seam settings separately from session/entity ownership. One worker and one completed Arc cache; CPU tests prove movement reuse, stale reconnect/entity/appearance/mount rejection, removal/reselection invalidation and sticky failure. Geometry, mesh expansion and atlas decoding run on the worker. |
| Persistent world body, no duplicate marker | `avatar_rendering::render` owns one root and GPU resources; world transform updates do not recreate meshes. `avatar_live` requires identical root/mesh IDs across server-received motion and turning. The primitive selected-player marker is removed only when the body is attached. |
| Native units, origin and anchor | Raw native geometry is not preview-fitted. One root scale converts to display units. `native_avatar_anchor.py` verifies neutral native CG convention. Landed and five-unit airborne captures at `target/connected-avatar-3inlmrs_` passed exact received/render-origin checks and were inspected. The small native sole residual remains documented, not corrected by a bounding box or terrain snap. |
| Globe orientation and turning | Shared tangent frame plus explicit source Euler convention; CPU placement/rotation tests and live server report/echo gates. Southern captures in `target/connected-avatar-lq49c67t` and `target/connected-avatar-r249tuf5` were inspected. Source bootstrap correction is confined to the explicit test-server fixture, not silently applied in Rust. |
| Second appearance through receive/update | Ordinary smaller-received capture and `avatar_live` sparse DNA update prove received size changes alter generated height. `target/connected-avatar-wpdj3q7g/live-update.png` was inspected. Old mesh assets must be released; unchanged origin is required, rather than snapping the smaller model to the terrain. |
| Removal, restoration, reconnect and selection ownership | `avatar_lifecycle` checks actual manifest removal, every owned GPU handle's release, CPU Arc reuse on restoration, 30 empty disconnected frames and authenticated same-ID reconnect with advanced generation/current bootstrap DNA. `target/connected-avatar-enxi7zms/live-lifecycle.png` was inspected. CPU races additionally cover selection changes while work is pending; the connected test does not claim to force that worker race. |
| Missing/unsupported input is visible | Preparation reports missing installation, missing DNA and native generator errors in `Presentation.avatar_status`; panels display it. Failures are sticky until request changes. Existing mounted/native rejection tests and preparation tests cover unsupported anatomy, invalid settings, dropped worker channels and missing assets. No fabricated geometry/weights or silent success substitution. |
| Camera and connected inspection | Native neutral eye metadata, head-only local-view suppression and view-only terrain clearance. Three final captures in `target/connected-avatar-ttoj5lhc` were opened: forward horizon, down-looking terrain and restored complete body. CPU checks preserve body transform and non-head visibility, including southern yaw/pitch. Shift+Backspace provides full-body inspection without modifying placement. See [camera evidence/limits](native-avatar-camera.md). |
| Quick-dev and local-client regression | `tests/local_server.py quick-smoke` passes auto-login for rust-local-test/Rust Explorer. `gui` passes login, terrain/near plane, Unicode chat/range, gear move/drop/pickup/use, mounted item transforms, hunger, position persistence and clean disconnect. All use disposable loopback worlds. All eight isolated gameplay modes pass: move, drop, pickup, position, pose, peer, use, use-noop. |
| Existing offline previews | Mounted assembly/contact/topology/neutral-pose checks: all 11 selected tests pass. Fresh eleven-view GPU batch passes both image-content checks; all eleven images in `target/avatar-textures-ah15hldu` were opened and inspected for assembly/skin placement. Its harness snapshots the viewer executable to avoid a concurrent different-feature build replacing it. |
| Test/lint/format/documentation/commits | 734 default-library, 754 viewer-library, 11 viewer-CLI and 777 client-library tests pass. Asset-dependent ignored cases are tested separately. Nine static verifiers and warnings-denied viewer/client Clippy pass. Format/diff checks pass. Recipes and README cover local launch and connected captures. This audit accompanies the final verification commit; preceding implementation checkpoints are separate commits. |

Final connected rerun after the camera/batch changes also passed every live and
lifecycle gate: server-received motion/turn with mesh reuse, sparse DNA update,
old-GPU cleanup, manifest removal/restoration with CPU reuse, disconnect and
authenticated same-ID reconnect. Its final image,
`target/connected-avatar-c4loabo7/live-lifecycle.png`, was opened and inspected:
the complete current received body is present without a stale/duplicate avatar.

## Explicit boundaries

All game resources come from the user's read-only installation. GPU captures and
derived caches stay ignored/outside that installation; no game assets are
redistributed. No public game servers or original executable/DLL execution were
used. Sparse echo/update, manifest withdrawal and southern bootstrap correction
are explicit owned-server fixtures, not claimed unmodified-server capabilities.

The supported path is neutral normal-skin Stand with modern smoothing, shading
and approved render-only seam sleeves. Animation, clothing/armor, broader anatomy,
remote populations, production cache/camera parity and new physics remain the
goal's deferred work. Native source weld/opening limitations and small sole
residuals are preserved and documented, not hidden by body/camera anchoring hacks.
