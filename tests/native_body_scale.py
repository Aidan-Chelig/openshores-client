"""Static unweighted body deformation branch and hidden-result ABI checks."""
import hashlib
import json
import subprocess
from pathlib import Path

target = Path("reference/openshores-full/OpenShores.exe")
digest = hashlib.sha256(target.read_bytes()).hexdigest()
assert digest == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
evidence = json.loads(Path("docs/evidence/native-body-apply-scale.json").read_text())
assert evidence["subject"]["digest"]["sha256"] == digest
assembly = subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble=_ZNK15Au3dsGLSLObject16ApplyScaleFactorER7AuPoint", str(target)], text=True)
for anchor in ["mov    %rdx,%rsi", "mov    %r8,%rdi", "mov    %rcx,0x240(%rsp)", "cvtss2sd 0x1c0(%rdx)", "cvtss2sd 0x1c4(%rdx)", "cvtss2sd 0x1c8(%rdx)", "cvtss2sd 0x1f4(%rdx)", "cvtss2sd 0x1f8(%rdx)", "cvtss2sd 0x1fc(%rdx)", "mov    0x138(%rsi),%r9", "je     141396647"]:
    assert anchor in assembly, anchor
base = assembly[assembly.index("141396647:"):assembly.index("141396699:")]
for anchor in ["movsd  0x140(%rsi),%xmm4", "ucomisd", "jp     1413967e8", "jne    1413967e8", "pxor   %xmm1,%xmm1", "addpd  %xmm1,%xmm6", "addsd  %xmm7,%xmm0", "movups %xmm6,(%rax)", "movsd  %xmm0,0x10(%rax)"]:
    assert anchor in base, anchor
nonzero = assembly[assembly.index("1413967e8:"):assembly.index("141396818:")]
for anchor in ["mulpd  %xmm2,%xmm0", "movupd (%rdi),%xmm1", "mulpd  %xmm2,%xmm1", "subpd  %xmm0,%xmm6", "mulsd  %xmm4,%xmm0", "mulsd  0x10(%rdi),%xmm4", "subsd  %xmm0,%xmm7", "jmp    141396681"]:
    assert anchor in nonzero, anchor
assert "call " not in base and "call " not in nonzero
print("Verified body deformation hidden result/object/input ABI, float metadata widening, no-group gate, zero/NaN scale branch and independent double products. These checks do not establish grouped deformation or avatar rendering.")
