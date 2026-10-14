# Native condition-list reception

Selected target: `reference/openshores-full/OpenShores.exe`, SHA-256
`285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80`.
Reviewed full static Ghidra 12.1.2 evidence:
`evidence/native-condition-list.json`,
`ev_18e528f9cecea1628d77c0a42a1961d8283c16d03fe6bd38eab346a48663d10d`.

`AuConditionList::operator<<` (`0x1408be450`) first erases every old list
entry and invokes each non-null condition's virtual destructor. Only then does
it read a short count. Each iteration reads an 8-bit type and calls
`AuCondition::CreateCondition`; a non-null result receives its virtual body at
vtable offset `0x70` and is appended. A null factory result emits a warning and
continues with the next type: there is no observed unknown-body length or skip.
Zero count therefore clears the previous list, unlike the additive base effect
and controller layers. Prior completed appends precede later iterations.

Remaining unknowns: count overload signedness/loop interpretation, complete
factory mapping, each concrete virtual reader and constructors. Do not treat
these as Item/controller types or silently skip arbitrary condition payloads.
Rust malformed-input behavior must be documented separately from Qt streams.

## Factory mapping

Complete reviewed `CreateCondition` evidence is retained in
`evidence/native-condition-factory.json`,
`ev_cd21060fb6057b8729a6808bf12bf403ed5d7ead3cf23b2e34f5bb7840f79866`.
Observed types: 1 Poison, 2 Disease, 3 Paralysis, 4 ImplantDNA, 5 Acid,
7 Thought, 8 Fire, 9 Pregnant, 10 Bandage, 11 Bounty, 12 ImplantSpecies,
13 Antitoxin. Type 6, zero and all other values enter warning/null fallback;
they are not generic base-condition allocations.

Every known allocation zeroes qword `0x08`. Poison/Paralysis/Acid initialize
the word at `0x10` to `0x0100`; Disease zeroes that word and sets byte `0x12`
to one. Bandage/Antitoxin set byte `0x10` to one. Thought uses the shared-null
string at `0x10`. ImplantDNA allocates and constructs DhDNA for pointer `0x10`;
ImplantSpecies constructs AuSpecies in place at `0x10`. Pregnant and Bounty
have distinct additional zero/string initializers; full semantic defaults
remain unestablished. Fire has only the base-sized allocation (`0x10` bytes).
These observed constructors do not establish the virtual receive body layouts.

Next resolve the base reader and concrete virtual receive bindings, then compose
the concrete counted layer into Unit reception.
