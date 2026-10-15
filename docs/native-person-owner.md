# Person receive storage and owner lifecycle

`NativePersonObjects` keys receive-visible Person objects by stable handle,
not wire ID. This uses the existing verified generic native owner lifecycle,
not the source-protocol entity model. It does not allocate skeletal models or
claim full Person packet dispatch; see native-person-construction.md for the
explicit construction boundary.

`construct_candidate(Person, handle, graph)` seeds the established constructor
subset and admits a Person cache node with matching position/status. Other
classes, duplicate handles, graph collisions and over-budget storage fail.
The 20,000-object bound is a Rust safety policy, not a recovered native limit.
`discard_candidate` validates graph/owner/lookup/retirement guards before
removing either store. A temporary with the same wire ID cannot erase the
retained object or shared index. Dropping Rust receive fields does not emulate
unmodeled C++ model destructors.

Candidate base reception validates class and unowned status, consumes the real
base layers using an explicit float profile, and synchronizes completed graph
position/status changes to the concrete Person. Later malformed fields preserve
completed earlier changes. Conditional position auxiliary work is required
from the caller; unresolved effects are not replaced by a successful no-op.

Fresh owner addition uses `add_atom_owner`, including its real routed map,
child-list arrival and runaway-recovery behavior. The Person owner bit is set
inside the actual owner-assignment callback, before recovery proceeds. An
existing wire ID is refused here: callers must select the retained object
instead of replacing it with a new candidate. Inconsistent cache class is also
refused before owner work.

Retained base reception requires matching owner/lookup/class observations. Its
position stage uses `NativePositionReceiver`, not a direct position overwrite:
wire-ID runaway queues, actual registration and reparent decisions are preserved.
Deferred owner context returns WouldBlock before later base/body reception.
The caller must supply concrete destination, registration and auxiliary effects;
these APIs do not fabricate runtime environment observations.

Offline tests cover separate handles sharing an identity, retained/candidate
disposal guards, duplicate class/graph admission, all float profiles for candidate
position reception, partial-error graph synchronization, actual fresh owner
arrival, refusal of an indexed replacement, and retained missing-parent queue/
registration using wire ID7 rather than handle90. Registration observations in
that fixture are explicit synthetic context, not recovered Person ticking policy.

The Native ReadPacket Person adapter now exists; see native-person-dispatch.md.
Its concrete client model context and the minimal client's presentation bridge
remain unfinished. These storage/lifecycle tests are not end-to-end
packet-to-avatar or live-native-session proof; no public server is involved.
