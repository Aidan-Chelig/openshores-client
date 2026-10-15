"""Static definition forwarding checks, never native execution."""
import hashlib
import json
import subprocess
from pathlib import Path

target = Path("reference/openshores-full/OpenShores.exe")
digest = hashlib.sha256(target.read_bytes()).hexdigest()
assert digest == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
records = {}
for name in ["constructor", "cache-generation", "instance-generation"]:
    filename = "native-body-definition-constructor" if name == "constructor" else "native-body-" + name
    record = json.loads(Path(f"docs/evidence/{filename}.json").read_text())
    assert record["subject"]["digest"]["sha256"] == digest
    records[name] = record["normalized_result"]

def disassemble(symbol):
    return subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", f"--disassemble={symbol}", str(target)], text=True)

wrapper = disassemble("_ZN14AuGLModelCache15CreateBodyModelER17AuOpenGLFunctionsRK7QStringR7AuPointS6_b24Au3dsGLSLModelDefinition")
for anchor in ["lea    0x40(%rax),%r10", "movsd  0x8(%rax),%xmm0", "movsd  (%rax),%xmm0", "call   1413fe560"]:
    assert anchor in wrapper, anchor
for source, destination in [(0x58,0x60),(0x60,0x68),(0x68,0x70),(0x70,0x78),(0x78,0x80)]:
    load = f"movsd  0x{source:x}(%rax),%xmm0"
    store = f"movsd  %xmm0,0x{destination:x}(%rsp)"
    assert load in wrapper and store in wrapper, (load, store)

instance = records["instance-generation"]
for offset, parameter in [(0x140,6),(0x178,11),(0x170,12),(0x188,13),(0x180,14)]:
    assert f"+ 0x{offset:x}) = param_{parameter};" in instance
assert "+ 400) = param_15;" in instance
assert "(float)dVar21,(float)dVar20" in instance and "= (float)dVar4;" in instance
constructor = disassemble("_ZN24Au3dsGLSLModelDefinitionC1EddddddiiiRK7AuPointddddd6QColorfS3_fffbbbj")
for anchor in ["movups %xmm1,(%rbx)", "movups %xmm3,0x10(%rbx)", "movups %xmm0,0x20(%rbx)",
               "movups %xmm1,0x40(%rbx)", "movups %xmm0,0x50(%rbx)", "movups %xmm0,0x60(%rbx)", "movups %xmm0,0x70(%rbx)"]:
    assert anchor in constructor, anchor
print("Verified constructor geometry layout, cache argument forwarding, generator field assignments and float translation narrowing. Not full definition/model generation or avatar rendering.")
