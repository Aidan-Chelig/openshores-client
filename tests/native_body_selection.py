"""Digest-bound static selection checks; no target execution."""
import runpy

runpy.run_path("tests/native_body_scale.py")
import subprocess

assembly = subprocess.check_output([
    "objdump", "-d", "--no-show-raw-insn",
    "--disassemble=_ZNK15Au3dsGLSLObject16ApplyScaleFactorER7AuPoint",
    "reference/openshores-full/OpenShores.exe",
], text=True)
selection = assembly[assembly.index("1413962e0:"):assembly.index("14139645f:")]
for anchor in [
    "QMapData", "findNode", "cvtpd2ps", "cvtsd2ss",
    "mov    $0x1,%r9d", "QString7indexOf", "cmovne %eax,%r13d",
    "cmp    $0x2,%eax", "je     141396708",
    "cmp    $0x4,%eax", "je     141396720",
    "cmp    $0x5,%eax", "je     141396740",
    "cmp    $0x6,%eax", "je     141396818",
    "add    $0x90,%rbx", "jne    1413962e0", "je     14139664f",
]:
    assert anchor in selection, anchor
positions = [selection.index(f"movzbl 0x{offset:x}(%rbx),%ebp") for offset in range(0x50, 0x54)]
assert positions == sorted(positions)
for anchor in ["movzbl 0x54(%rbx),%ebp", "movzbl 0x55(%rbx),%ebp",
               "cvtss2sd 0x28(%rbx)", "cvtss2sd 0x2c(%rbx)", "cvtss2sd 0x30(%rbx)"]:
    assert anchor in assembly, anchor
print("Verified narrowed membership lookup, case-sensitive sticky Wing selection, special-kind precedence, anatomical flag order, pivot widening and ordered iteration. No weighted arithmetic/rendering parity claim.")
