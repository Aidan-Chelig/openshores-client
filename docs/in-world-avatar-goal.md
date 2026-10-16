# Next goal: render the logged-in player's actual avatar in-world

Implementation status: [progress ledger](in-world-avatar-progress.md).

## Milestone in one sentence

Run `just quick-dev` and see the logged-in player's actual received, textured
avatar standing, moving and turning correctly in the world—not just in a preview.

## Delivery checkpoints

1. **Receive and prepare:** map the selected player's received DNA and prepare
   its geometry and textures asynchronously, with ownership-safe invalidation.
   This checkpoint is implemented; see the progress ledger for verification.
2. **Render in-world:** attach the prepared body to a persistent player render
   root, replace only that player's marker, and preserve native model units.
   Moving must update the transform without regenerating the body.
3. **Verify placement:** establish the body origin, globe-local orientation and
   ground-contact convention. Cover turning, southern-hemisphere placement and
   deliberately off-ground positions without corrective terrain snapping.
4. **Prove the connected workflow:** exercise appearance changes and session
   cleanup against the owned local server, inspect real GPU captures, run
   regressions and document the reproducible result.

Checkpoint completion is not milestone completion: the goal is done only when
the connected-client acceptance criteria below pass.

## Objective

Connect the locally logged-in player's received appearance to the verified
native avatar generator and render that textured avatar in the live Rust client,
replacing its primitive entity marker. Use the server-provided position and
orientation, the existing terrain renderer and the user-mounted Hazeron assets.

The end-to-end acceptance workflow is `just quick-dev`: log into the owned local
server as `rust-local-test`, select Rust Explorer, and inspect the actual received
avatar standing and moving in the world. This brings the
[completed offline preview](assembled-avatar-acceptance.md) into the game client;
another fixture-only viewer does not satisfy this goal.

## Work sequence

1. Trace local login/avatar selection through person creation, appearance updates
   and scene ownership. Identify the authoritative local-player entity and the
   received DNA/appearance fields required by `NativeDpAvatarGeometry::generate`.
   Preserve the explicit reference-server/native protocol boundary described in
   [entity transforms](native-entity-transforms.md); do not infer compatibility
   from matching field names or auto-detect one profile as another.
2. Add an appearance-to-render preparation path using the existing mount,
   decoders, neutral Stand pose, smoothing, skin material and approved render-only
   sleeves. Keep CPU generation reusable and expensive work off the frame loop.
   Key reuse/invalidation by appearance and relevant asset/render settings, not
   position. Do not silently substitute the offline fixture DNA when received
   appearance is missing or unsupported.
3. Attach the prepared model to the actual local-player entity. Replace only its
   primitive marker; retain other entity diagnostics. Apply world position,
   orientation, scale and globe-local up/forward conventions once, through the
   connected client's transform path. Keep native pose/model coordinates separate
   from world placement and floating-origin/display transforms.
4. Establish the body origin and ground-contact convention from existing
   transform/pose evidence. A landed player must stand at the right terrain
   height without being buried or floating. Preserve authoritative coordinates:
   do not hide an incorrect anchor with a bounding-box offset, camera movement,
   or unconditional terrain snapping. Off-ground positions must remain off-ground.
5. Handle lifecycle changes: appearance updates, selection/reconnect, disconnect,
   entity removal and failed preparation. Discard stale asynchronous results and
   release obsolete render entities/resources. Report unsupported appearances or
   missing assets clearly instead of leaving a misleading silent placeholder.
6. Add focused CPU, local-server and real GPU coverage. Provide a minimal way to
   inspect the player's full body in the connected client if the normal camera
   hides it; this is an inspection aid, not a separate avatar showcase or UI redesign.

## Completion criteria

- `just quick-dev` launches the owned local server and Rust client, logs into
  `rust-local-test`, selects Rust Explorer, and displays that entity's actual
  received textured avatar. No manual model import, fixture-DNA replacement or
  duplicate primitive player marker is needed.
- Controlled local-server appearance data is demonstrably reflected in the
  rendered model. At least one second supported appearance is delivered through
  the receive/update path to prove this is not a hard-coded preview attachment.
  If the existing test account uses unsupported anatomy, add the required support
  or explicitly prepare a documented supported local-server test appearance;
  never change it only inside the renderer and call that received appearance.
- Moving and turning update the avatar in place without rebuilding its mesh each
  frame, accumulating transforms or separating its parts. Tests cover the relevant
  globe-local orientation/frame changes, including a southern-hemisphere case to
  guard against the earlier inverted-movement/frame issue.
- The body has the correct scale, up/forward axes and terrain-relative anchor.
  Landed and off-ground cases distinguish correct placement from forced snapping.
  World motion and player-camera behavior do not regress.
- Appearance changes and reconnect/selection rebuild or reuse the right model
  cleanly. Late work cannot attach an old appearance to a new session/entity;
  disconnect/removal leaves no duplicate or orphaned avatar render entities.
- Missing installations/assets and unsupported DNA/appearance produce an
  actionable visible status and useful diagnostics. No fabricated geometry,
  fallback weights, silent fixture substitution or success claim masks failure.
- CPU tests cover received-field mapping, appearance keys/invalidation, transform
  order/anchor, lifecycle races and error handling. Owned local-server integration
  exercises login/selection, movement, turning and an appearance update.
- Real connected-client GPU output is captured and visually inspected for body
  assembly, skin placement, world orientation and ground contact. A clean launch
  or nonblank image alone is not sufficient. Existing offline avatar previews and
  local-client regressions still pass; formatting and lint pass.
- Relevant task-runner recipes and documentation explain how to reproduce the
  result. Meaningful implementation milestones and final verification are committed.

## Boundaries and follow-on work

Use only the supplied local development server in an owned disposable world.
Never contact public game servers or execute original reference binaries/DLLs.
Mount the user's Hazeron installation read-only; keep derived caches and captures
outside it and do not redistribute game assets or captures.

Reuse retained native evidence and existing analysis. Investigate a concrete
missing dependency only when needed; avoid repeated broad static-analysis imports.
Record evidence, deliberate modern choices and unsupported cases separately.

Keep neutral Stand, existing angle smoothing, approved render-only seam repair
and modern diagnostic shading. Walking/idle animation, clothing/armor, broader
anatomy coverage, remote-player population rendering, full production cache parity,
new collision/locomotion physics and unrelated UI/terrain/shading changes are
follow-on work. Do not expand this milestone into them unless a specific dependency
is genuinely required for the logged-in player's appearance and placement.
