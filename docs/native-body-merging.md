# Native body merging: implementation boundary

Static REA results from the existing warm session are retained in
`evidence/native-body-merge-head.json` (1413a7ea0),
`evidence/native-body-merge-torso.json` (1413aa3e0), and
`evidence/native-body-add-if-unique.json` (14139c1c0).
The three queries took 0.581s, 0.423s and 0.090s respectively; no executable
reimport, target execution, or public network request was needed.

Focused inspection establishes that both merges seed a missing destination
through CopyInstance, rather than concatenating arbitrary rendered triangles.
Head returns immediately after that seed. Torso has a distinct bool branch.
Both subsequently handle native object/group/face/material storage. Full merge
reconstruction remains incomplete: neither routine may be replaced with simple
mesh concatenation or a universal position-weld rule.

The inspected head connection branch uses inclusive per-axis absolute
differences, while the inspected torso connection branch uses strict Euclidean
distance. Head also calls AddIfUnique with its final bool true. These are
different operations and must remain distinct.

`native_body_add_if_unique` implements that complete small callee: first matching
position by strict Euclidean distance, double arithmetic on stored floats,
conditional float equality of opaque fields20/24, and snap-and-append when the
fields differ or comparison is disabled. It preserves all ten payload words;
the fields are not guessed to be UVs, normals, or semantic skin data. Tests cover
the strict boundary, first-match priority, snapping, payload preservation,
signed zero and NaN field comparisons. It does not implement the larger merge.

Next required reconstruction is ordered connection-group remapping, original
face/material/group updates and CopyInstance state. The retained complete
records can be paged locally; do not request the same functions again.
