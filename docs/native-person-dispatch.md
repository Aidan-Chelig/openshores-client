# Native Person packet dispatch

`NativePersonDispatch` adapts the existing native `receive_atom` path to actual
`NativePersonObjects`, owner/cache stores and all four inherited receive stages.
It is separate from source-profile Person parsing. Its explicit floating profile
is passed to base and every inherited body reader; nested Qt7 format limits still
apply. This is not a native login implementation or graphical runtime context.

Fresh native opcode05 packets construct receive storage and require runtime
model construction, assign identity/time, receive base (including real effects/
controllers), connect actual signal bindings and add the object through the
owner lifecycle, then receive Unit → Creature → Sentient → Person and perform
post-update work. Indexed packets assign the timestamp and receive base on the
retained handle, dispose the temporary candidate/context, then dispatch the body
on the retained Person. They do not reconnect signals or add the owner again.

`NativePersonDispatchHooks` has no successful no-op defaults. It requires actual
runtime construction/disposal, per-handle Person context, position destination/
auxiliary operations, clocks, receive diagnostics and post-update effects.
Unimplemented model/environment work must return Unsupported rather than
pretending that storing a decoded field performed its native setter. Nested
receive reports are forwarded before post-update, including null/obsolete item
factory diagnostics; they are not silently discarded by the dispatch adapter.

Other recognized classes are explicitly Unsupported in this single-class adapter.
An indexed handle not represented in Person storage is refused, not retagged or
parsed using the incoming candidate's class. Unrecognized/non-atom opcodes return
None without construction, following the existing dispatcher contract.

Errors preserve completed operations, not a synthetic rollback: construction can
precede identity EOF, owner attachment can precede a body error, and retained base
updates can precede candidate disposal. Temporary runtime disposal is guarded by
owner/lookup/retirement checks before its hook. The owning session must recover
or dispose partial candidates on Rust errors. Qt failed-stream, C++ exceptions
and unmodeled destructor parity are not claimed.

## Offline verification and remaining gates

`just test-native-person-dispatch` exercises complete unframed opcode05 packets
from identity through actual base codecs and all inherited body readers. It
checks every truncated prefix in all three float profiles, fresh owner/signal
stages, sparse retained updates, constructor defaults, order initialization,
skills, Person double/nibble fields and inventory null-factory diagnostics.
The second packet proves stable handle90 reuse, timestamp replacement and
temporary handle91 disposal before retained body reception. It also checks
explicit runtime-construction/DNA-context rejection and unsupported class
handling without post-update success.

The fixture's model/environment context is explicitly synthetic: it tracks its
own HP/time/model selection and supplies declared clock/matrix/seat observations,
while rejecting unsupported DNA/pose/scalar/console operations. These tests
therefore establish packet/storage/owner orchestration, not an actual rendered
avatar or native world/model parity. A concrete client runtime context, remaining
model effects and the minimal client's native presentation bridge are still
required. Source-server tests cannot establish native layout compatibility;
no public server or supplied executable is run here.
