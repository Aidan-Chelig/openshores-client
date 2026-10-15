# Next goal: a complete, assembled native avatar preview

Implementation/evidence ledger: [native avatar assembly](native-avatar-assembly.md).

Render one complete, textured avatar in a verified neutral standing pose in the
offline Rust preview. Assemble the existing native head, torso, neck, arm/hand,
leg/foot and optional tail geometry using the reconstructed bone hierarchy and
weights. Replace the current separate unposed part inspection with an additional
full-body preview, while retaining those diagnostic views.

This advances the OpenShores Rust port toward real in-game avatars. It does not
substitute a hand-arranged mannequin or a generic humanoid for native anatomy.

## Work sequence

1. Audit the current part plans, geometry merges, bone transforms, neutral-pose
   calculations and vertex weights. Reuse retained evidence and the existing
   analyzed session; investigate only missing dependencies. Identify the native
   distinction between object, bone-local and posed coordinates before applying
   transforms. Do not infer attachment offsets from rendered bounding boxes.
2. Build a reusable CPU avatar-generation result from explicit fixture DNA and
   owner appearance fields. Include all supported parts in native order, their
   actual bone bindings, merged surfaces, UVs, materials and skin controls. Keep
   generated state independent of the viewer and preserve raw native payloads.
3. Reconstruct and apply the required neutral standing bone pose. Skin vertices
   and normals with the native hierarchy, transform order and influence rules;
   report unsupported inputs instead of inventing fallback weights or poses.
   Do not apply already baked deformation or seed projection a second time.
4. Publish the complete posed geometry through the existing textured preview
   pipeline, preserving atlas selection, seams and angle-based smoothing. Fit
   the camera to the assembled result and establish its up/forward convention
   from evidence. Keep modern diagnostic lighting; faithful shading is deferred.
5. Add a full-body `just preview-avatar body` workflow plus targeted offline and
   GPU tests. Inspect the assembled model from multiple directions and document
   what is verified, fixture-supplied and still unsupported.

## Completion criteria

- A complete fixture avatar is visibly assembled: head connected through its
  neck to the torso, arms/hands and legs/feet attached in the correct places,
  and the selected tail attached correctly. No floating inspection pieces,
  collapsed limbs, double transforms or manually guessed placement offsets.
- The neutral standing pose follows reconstructed native calculations. It is
  not merely each mesh's local coordinates displayed together.
- A second supported DNA fixture with different proportions/part counts proves
  the assembly is driven by native plans rather than hard-coded for one body.
  Choose its supported shape from available evidence/assets and record it.
- Texture placement and top/bottom skin composition remain correct on every
  displayed part. UV seams do not become unintended hard edges; sharp edges
  remain hard at the selected smoothing threshold.
- CPU tests cover hierarchy/order, known attachment positions, skinning weights,
  normal transformation, merge-to-pose sequencing and explicit unsupported cases.
  Geometry retains consistent vertex/UV/weight counts and valid face indices.
- Existing head, torso, untextured and texture-regression workflows still pass.
  Full-body GPU captures are nonblank and visually inspected from multiple views;
  a clean process exit alone is not completion evidence.
- Launches are repeatable offline. Tests, lint and formatting pass; meaningful
  milestones and the final verification/limitations are committed and documented.

## Boundaries and follow-on work

Use only assets mounted read-only from a user-provided Hazeron installation.
Never execute reference binaries/DLLs, contact public game servers, modify the
installation or redistribute its assets/captures. Any cache and captures belong
outside the installation. Reuse the existing mount, decoders and skin material.

Preserve the native assembly/cache semantics needed to produce the preview;
do not claim full production cache lifecycle parity where it remains unverified.
Avoid unrelated UI, lighting, terrain or rendering-engine work.

This goal establishes reusable avatar generation and a complete offline visual
reference. Received player appearance, production receive-context ownership,
in-world placement, animation and clothing/armor coverage remain follow-on work;
do not describe this preview as finished multiplayer avatar integration.
