# Composed native Person body reception

NativePersonBodyState now owns the actual Creature (including Unit), Sentient
and Person suffix states. Its receive_qt7 method executes those existing real
readers in order on one cursor. It returns their nested results/diagnostics
and per-stage byte accounting without copying/skipping unparsed wire data.
Earlier mutations/effects remain after later errors; no later stage runs after
an earlier failure. Sparse repeated updates operate on the same retained state.

This is ordinary source composition of the previously reconstructed readers,
not new native analysis. Evidence for the native inheritance/call sequence is
indexed in native-person.md, native-sentient.md and native-creature.md. No new
analysis session/import, native target execution or public server was needed.

The enclosing atom/base reception must already have succeeded. This method
does not receive the base twice, invent Item/Person constructor defaults, or
substitute source-server packets for the native profile. Its Default remains
observation state, not a complete native object constructor. Actual prior
Person660/state1130 must be supplied before masked updates can succeed.

NativePersonReceiveContext extends the required Creature effects with real
order initialization matrix/wall-clock and monotonic-time operations. No
production/default successful context exists yet. An unchanged retained order
does not request another seed; an unchanged Person short does not request
another timer timestamp. Unsupported nested readers and context effects remain
explicit errors rather than silently bypassing native behavior.

Offline fixtures exercise actual Unit scalar, Creature HP/tail, Sentient order/
skill map, and Person content/scalar readers in all three precision profiles.
They verify byte boundaries, trailing sentinel alignment, repeated sparse
updates, every truncated prefix and every exercised required-context failure.
The context is explicitly trace-only: these fixtures prove inherited codec
composition and ordering, not native model/capsule/runtime effects.

Still required for the unchanged goal: remaining item/job/inventory readers and
weight/default behavior; concrete model/Creature context and inherited native
construction; base/object lifecycle/dispatch integration; native packet fixtures
reaching actual minimal-client avatar presentation; applicable local-server
verification with protocol limitations stated. The body receiver is not that
completed end-to-end path.

Verification: `just check` passes405 headless and426 client library tests,
9 CLI tests per profile,1 graphical-entry CLI test, formatting and strict
client-feature all-target Clippy.
