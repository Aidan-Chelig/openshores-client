"""Independently derive model-DNA bit selectors/comparison order from GNU instructions."""
from pathlib import Path
import re
import runpy
import subprocess

inventory = runpy.run_path(str(Path(__file__).with_name("native_chromosome_volumes.py")))
def asm(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol,
                                    str(inventory["target"])], text=True)

source = Path("src/world/native_body/dp/model_dna.rs").read_text()
layout_source = source.split("const LAYOUT:", 1)[1].split("=", 1)[1].split("];", 1)[0]
layout = [tuple(map(int, match)) for match in re.findall(r"\((\d+),\s*(\d+),\s*(\d+)\)", layout_source)]
assert len(layout) == 47
observed = {}
for line in asm("_ZNK5DhDNA14CreateModelDNAER12Au3dsGLSLDNA").splitlines():
    if match := re.search(r"movzbl\s+(0x[0-9a-f]+)\(%rcx\),%edx", line):
        selector = [int(match[1], 16) - 8, 0, 255]
    elif match := re.search(r"shr\s+\$0x([0-9a-f]+),%dl", line):
        selector[1] = int(match[1], 16)
        selector[2] = 255 >> selector[1]
    elif re.search(r"shr\s+(?:\$1,)?%dl", line):
        selector[1] = 1
        selector[2] = 127
    elif match := re.search(r"and\s+\$0x([0-9a-f]+),%edx", line):
        selector[2] = int(match[1], 16)
    elif match := re.search(r"mov\s+%edx,(0x[0-9a-f]+)?\(%rax\)", line):
        offset = int(match[1] or "0", 16)
        assert offset not in observed
        observed[offset] = tuple(selector)
assert observed == {i * 4: selector for i, selector in enumerate(layout)}

order_source = source.split("const ORDER:", 1)[1].split("=", 1)[1].split("];", 1)[0]
order = [int(value, 16) for value in re.findall(r"0x[0-9a-f]+", order_source)]
compare = asm("_ZNK12Au3dsGLSLDNAltERKS_")
offsets = [int(value or "0", 16) for value in re.findall(r"mov\s+(0x[0-9a-f]+)?\(%rdx\),%eax", compare)]
assert offsets == order and sorted(order) == list(range(0, 192, 4))
assert "setb   %al" in compare
hand = asm("_ZNK6DpBody10HandLengthEb")
assert "lea    0xc0(%rsp),%r12" in hand
assert "movl   $0x1,0x17c(%rsp)" in hand  # Keybc, NOT keyb4.
assert "movaps %xmm6,%xmm0" in hand  # Actual numeric return despite void pseudocode.
print("Verified47 model-DNA selector writes, all48 unsigned comparison priorities, and PA hand keybc override. Static only; no HandLength geometry/runtime parity.")
