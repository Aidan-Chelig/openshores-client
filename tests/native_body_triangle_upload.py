"""Static projected/deformed normal-body upload stages, never target execution."""
import json
import runpy
import subprocess

context = runpy.run_path("tests/native_dp_connections.py")
symbol = "_ZN15Au3dsGLSLObject9LoadModelER20Au3dsGLSLTriBodyListRK22Au3dsGLSLMaterialArrayb10MODEL_TYPEj"
assembly = subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol,
                                    str(context["target"])], text=True)
saved = context["Path"]("docs/evidence/native-body-triangle-upload-gnu.txt").read_text()
assert assembly.strip() == saved.strip()
record = json.loads(context["Path"]("docs/evidence/native-body-load-body-model.json").read_text())
assert record["subject"]["digest"]["sha256"] == context["hashlib"].sha256(context["data"]).hexdigest()
assert record["evidence_id"] == "ev_0ad9fb10afe5f344430f9e7a2f8c50c4508527e505d2d3c8e1f4cd1150b664c0"
# The three object-matrix Project results are retained by pointer, then passed
# to ApplyScaleFactor when field369 is false. Already baked objects bypass it.
for anchor in [
    "1413a0d60:", "mov    %rcx,0xc0(%rsp)", "1413a0d70:",
    "1413a0da4:", "mov    %rcx,0x1e0(%rsp)", "1413a0dac:",
    "1413a0de0:", "mov    %rcx,0x120(%rsp)", "1413a0de8:",
    "1413a0710:", "je     1413a1ccf", "1413a1ccf:", "mov    0xc0(%rsp),%r8", "1413a1cea:",
    "1413a2020:", "mov    0x1e0(%rsp),%r8", "1413a2036:",
    "1413a1c37:", "mov    0x120(%rsp),%r8", "1413a1c52:",
    "cmpb   $0x0,0x369(%rbx)",
]:
    assert anchor in assembly, anchor
assert assembly.count("call   1418fd760") == 3
assert assembly.count("call   141395f30") == 6  # normal and type3 branches
# Normal body skin indices and weights are copied from the stored payload.
for anchor in ["1413a0759:", "movss  0x1c(%rax),%xmm7", "movups 0x10(%rax),%xmm0",
               "1413a1d60:", "mov    (%rax),%esi", "mov    0x4(%rax),%r14d",
               "mov    0x8(%rax),%esi", "mov    0xc(%rax),%eax"]:
    assert anchor in assembly, anchor
print("Verified three Project outputs feed unbaked ApplyScaleFactor, baked bypass, and normal-body raw influence upload. Static evidence, not runtime mesh contact proof.")
