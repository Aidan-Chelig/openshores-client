# Neutral local-player camera

The connected renderer uses the prepared Skull EyePosition instead of the
prototype 1.7-unit height. This changes only the camera, never received body
coordinates, vertices or reports. First-person suppresses Skull-derived head
batches only; inspection/top-down restores them. Other body parts stay rendered.

## Static evidence

Target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
No original executable was run and no new broad analysis import was needed.

- Retained `native-body-cache-generation.json`, evidence
  `ev_980dd5bc2a83b8aa63ce6dbf99ce4dc7bfc9da7a34e4d275eb90eb0ae9364f21`:
  independently scale EyePosition and ConnectPt, width-adjust about Head pivot,
  then subtract. Read original group points, before head merge baking.
- Retained `native-dp-head-create-skin.json`, evidence
  `ev_3c792d45c4c6bf3d03eb3a6a1ef7ce75b05a71d21a3ab192e442aba9baabe426`:
  copy camera metadata to body fields 0x4050..0x4060.
- Retained `native-dp-render-skin.json`, evidence
  `ev_af9cc8fdb6fa22f65648aac8c12d75f8990df1203bd28fa9a39d6bacb6cb689a`:
  Head rendering is conditional; other body parts are not collectively omitted.
  Local-person dispatch sets the flag for global mode 2. Calling that mode
  first-person is an inference, not a recovered enum or runtime observation.

Bounded GNU disassembly of CurrentCameraLoc (`0x1412d08e0`) confirms an
asymmetry: `0x1412d090b` zeroes xmm0; `0x1412d0960` adds that zero to head X,
while Y/Z add the body offsets. Rust preserves this rather than adding authored
lateral offset. Head index 1 selects bone 12, otherwise bone 7.
CameraLocation (`0x1412d0bf0`) selects the head using DNA byte 5 bit 1 and
field3a. The adapter takes field3a explicitly; this neutral context supplies
false. Other poses, growth/vehicle rules and production camera parity remain
unsupported, not inferred from these observations.

## Verification checkpoint

`just test-connected-avatar-first-person` captures the ordinary camera against
the owned loopback server with received-DNA, origin and height gates.
The initial `target/connected-avatar-4nptgpdz/first-person.png` showed only avatar
interior. After the eye/head-mask fix,
`target/connected-avatar-ryiqb1i8/first-person.png` was opened and inspected:
terrain and sky have an unobstructed horizon. These remain ignored local images.
This capture preceded the lateral-zero correction (under 0.001 native units);
a final-binary rerun is still required.

CPU coverage checks southern camera composition without moving the body,
inspection head restoration, ignored lateral offset and metadata operation order.
The GUI regression checks the prepared eye when present, otherwise the prototype
fallback. Terrain clearance affects only the camera, using rendered tessellation.
Final camera rerun: `target/connected-avatar-ttoj5lhc/first-person.png`,
`first-person-down.png` and `inspection-restored.png` all passed and were opened.
Forward view has a clear horizon; looking down shows unobstructed terrain (not
a claim that feet must enter that frustum). Inspection restores the complete
textured head/body after 300 first-person frames, without replacing the body.
The native lateral-zero correction is included. Southern CPU coverage also
checks pitched/yawed camera direction and that non-head visibility is untouched.

## Elevated view follow-up

Backspace now translates the ordinary camera eight native world units upward
without changing pitch/yaw or forcing it to look at the avatar. Both prepared
and fallback cameras use the same view helper. The old elevated offset was in
display units and made the avatar tiny; height is now scaled exactly once.
The entire body/head is visible in elevated mode; aim down to see it.

At the user's request, holding [/] smoothly lowers/raises the offset at six
native units per second initially. A follow-up increases that baseline by 35%
to 8.1, accelerating to 9.9 (+65%) over two seconds held. The user's next
adjustment triples both endpoints to 24.3–29.7; a later maximum-only increase
sets the current ramp to 24.3–89.1, keeping two seconds, with release/reversal
reset and frame-rate-independent integration. Height stays bounded to 0.1–100.
Opposite keys cancel, release stops,
and modified brackets retain their other meanings. This is a deliberate modern
control change, not a claim of native stepped-control fidelity. CPU input tests
cover held-not-just-pressed frames, release, opposite/modifier keys and mode
exclusions; southern camera tests check equal orientation, scaled upward-only
translation, head restoration and unchanged body placement.

Verification: 778 client-library tests pass (six asset/GPU cases ignored),
warnings-denied client Clippy and formatting pass. The owned-server elevated
capture `target/connected-avatar-zegpdmlx/elevated-look-down.png` passed received
origin/DNA/height gates and was opened: the textured head/shoulders are visible
from above at the expected scale, with no full-screen head occlusion. The capture
remains local and ignored. Reproduce just this view with
`OPENSHORES_TEST_AVATAR_FIRST_PERSON=elevated` in the connected avatar harness;
the ordinary first-person recipe now includes it in its full camera batch.
