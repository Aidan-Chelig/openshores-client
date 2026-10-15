"""Static anatomical arithmetic anchors, never execution of the target."""
import runpy

state = runpy.run_path("tests/native_body_scale.py")
assembly = state["assembly"]
pseudocode = state["evidence"]["normalized_result"]
# Full retained analysis is already digest-bound by native_body_scale.py.
for anchor in [
    "if (local_187 == '\\0')", "if (local_164 != '\\0')",
    "if (local_161 != '\\0')", "if (local_185 == '\\0')",
    "if (local_162 != '\\0')", "(in_RDX + 0x180)",
    "(in_RDX + 0x188)", "if ((local_209 == '\\0') || (!bVar8))",
    "dVar19 = (dVar24 - dVar31) + dVar19;",
    "dVar20 = (dVar28 - dVar22) + dVar20;",
    "dVar17 = (dVar23 - dVar30) + dVar17;",
]:
    assert anchor in pseudocode, anchor
final = assembly[assembly.index("141396603:"):assembly.index("141396645:")]
for anchor in [
    "subsd  %xmm14,%xmm7", "subsd  %xmm5,%xmm9", "subsd  %xmm13,%xmm6",
    "addsd  %xmm2,%xmm7", "addsd  %xmm3,%xmm9", "addsd  %xmm1,%xmm6",
    "movsd  %xmm6,(%rax)", "movsd  %xmm9,0x8(%rax)", "movsd  %xmm7,0x10(%rax)",
]:
    assert anchor in final, anchor
neck = assembly[assembly.index("141396abe:"):assembly.index("141396b5e:")]
for anchor in ["movsd  0x180(%rsi),%xmm0", "jp     141397119",
               "movsd  0x188(%rsi),%xmm0", "mulsd  %xmm0,%xmm3",
               "jmp    141396a15"]:
    assert anchor in neck, anchor
assert "call " not in neck and "call " not in final
torso = assembly[assembly.index("14139724d:"):assembly.index("141397376:")]
for anchor in ["movsd  0x170(%rsi),%xmm0", "jp     141397273",
               "mulsd  %xmm0,%xmm2", "mulsd  %xmm0,%xmm1",
               "movsd  0x20(%rsp),%xmm0", "je     141396603",
               "mulsd  %xmm15,%xmm3", "mulsd  %xmm15,%xmm5",
               "jmp    1413965cd"]:
    assert anchor in torso, anchor
assert "0x180(%rsi)" not in torso and "0x188(%rsi)" not in torso
assert "call " not in torso
hand = assembly[assembly.index("1413971a7:"):assembly.index("14139724d:")]
for anchor in ["mulsd  %xmm12,%xmm2", "mulsd  %xmm12,%xmm3", "mulsd  %xmm12,%xmm1",
               "mulsd  %xmm12,%xmm14", "mulsd  %xmm12,%xmm5", "mulsd  %xmm12,%xmm13",
               "movsd  0x188(%rsi),%xmm12", "jmp    141396e8f"]:
    assert anchor in hand, anchor
assert "call " not in hand
print("Verified retained anatomical branch precedence, neck/torso scale stages and final independent offset-minus-connection-plus-vertex. Static/Rust checks only; no native execution or rendered-avatar claim.")
