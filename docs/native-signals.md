# Native atom signal wiring

Patched target SHA-256:
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Complete REA evidence for `_ZN6DaAtom14ConnectSignalsEv` at `0x140b280a0`
is retained in [native-connect-signals.json](evidence/native-connect-signals.json),
Evidence `ev_6a06b7cffcfa82cbe25d54383499b794fec9cbc9e5487829823baae0c795e441`
(663055ms, Ghidra 12.1.2). The full 27,385-character pseudocode was reviewed.
This is static evidence, not observed native execution or original source.

The global raw mode at `0x204` selects connection paths; meanings for mode
numbers are not inferred from neighboring UI/source-profile modes:

- Mode 9 returns without connections.
- Modes 10 and 5 first connect OperateTurboLift to OperateTurboLiftUI with
  raw connection type `0x82`. Other modes start with AddCitizenToEmpire using
  the same raw type. Failure of that first connection exits the whole routine;
  later connections are not checked in this function.
- Mode 5 then connects EmpireRequested, RestoreConsoleSettings,
  EnteredWormholeOnClient to AtomEnteredWormhole, CollisionOnClient to
  AtomCollision, DeadheadedOnClient to AtomDeadheaded, FellOnClient to AtomFell,
  transporter coordinate/long-lat UI callbacks, and CreateEmpireUI. It jumps
  to the final connection and returns, **skipping the shared tail** used by
  other non-9 modes. These are ten total attempted connections on first success.
- Mode 10 includes a larger UI connection set, including RestoreConsoleSettings,
  transporter UI callbacks, CreateEmpireUI and city/empire/avatar/trade events.
  Its EnteredDesigner and GameOver targets use slot-prefix `1`; other modes
  use signal-prefix `2` for those targets. GameTurnPoll similarly targets
  slot GameTurn in mode 10 versus signal GameTurn on the other broad path.
- Modes 10 and the broad other-mode path then connect the common tail of
  officer/avatar/job/warp/berth/political/system/stargate/spawn/trade/travel
  notifications, ending with WormholeExplored. Most raw connection types are 2;
  CityDecayed and EmpireCityWorkerSpeciesChanged use 0.

All receivers are the application singleton. Qt signature prefixes, argument
types, endpoint distinctions, connection order and raw connection values remain
preserved in the complete evidence. No Rust callback implementation is claimed
as implemented downstream behavior. `native_signals/` now ports complete plans:
57 connections for mode 10, ten for mode 5, none for mode 9, and 52 for all
other raw modes. Exact binding tables are split into mode/common modules and
transcribed from the fully retained/reviewed pseudocode. `connect_signals`
requires an explicit application connector and stops on first rejection, while
later boolean rejections do not suppress subsequent calls. Rust connector errors
stop without undoing earlier connector effects; Qt error/queue behavior is not
invented. Tests verify branch counts, signature prefixes, raw connection values,
mode-5 tail exclusion, installation order and rejection gating.

Next: bind required local event sinks, then join this stage to concrete native
owner dispatch. Qt queue/thread semantics and downstream handlers still need
their own reconstruction; installing a plan is not executing the handlers.

The atom-dispatch fixture now invokes the real signal installer before its
owner-add stage, recording exact attempted bindings for an explicit raw mode 5.
An integration test rejects the first connection and verifies that only one
binding is attempted **but AddAtom/body/post-update still proceed**: native
ConnectSignals returns void, and its gate limits wiring rather than aborting
ReadPacket. This connector records fixture bindings; it is not a downstream
event-handler implementation or a runnable native session owner.
