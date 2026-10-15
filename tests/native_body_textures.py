"""Static skin atlas/offset and image row-order anchors, no target execution."""
import json
import runpy
import struct
import subprocess
from pathlib import Path

state = runpy.run_path("tests/native_dp_connections.py")
for filename in ["native-dp-select-textures", "native-dp-setup-skin-material",
                 "native-body-get-texture-weight", "native-gl-texture-create-bitmap"]:
    record = json.loads(Path(f"docs/evidence/{filename}.json").read_text())
    assert record["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"

for address, value in [(0x1422BFAA8, 1024.), (0x1422BFAE8, 1./4096.),
                       (0x1422BFAB0, .75), (0x1422BFA00, 1.)]:
    assert struct.unpack("<d", state["read"](address, 8))[0] == value
for address, value in [(0x1422BDB58, b"Skin"), (0x1422BDB97, b"Exo"),
                       (0x1422BDB9B, b"Scales"), (0x1422BDBA2, b"FurFeathers")]:
    assert state["read"](address, len(value)+1) == value+b"\0"

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--no-show-raw-insn",
                                    "--disassemble="+symbol, str(state["target"])], text=True)

select = assembly("_ZN6DpBody14SelectTexturesER11AuGLEffects")
for anchor in ["movzbl 0x28(%rbx),%eax", "mov    $0x84,%edx",
               "mov    $0x21,%edx", "mov    $0x42,%edx"]:
    assert anchor in select, anchor
setup = assembly("_ZN6DpBody17SetupSkinMaterialEv")
for anchor in ["movzbl 0x2b(%rax),%edx", "movzbl 0x1b(%rax),%edx",
               "movsd  %xmm0,0x4120(%rax)", "movsd  %xmm1,0x4130(%rax)",
               "movups %xmm0,0x4070(%rax)"]:
    assert anchor in setup, anchor
weight = assembly("_ZN15Au3dsGLSLObject16GetTextureWeightE15Au3dsGLSLVertex")
for anchor in ["cmpl   $0x7,0x34(%r8)", "findNode", "pxor   %xmm0,%xmm0", "1422cf250"]:
    assert anchor in weight, anchor
image = assembly("_ZN11AuGLTexture16CreateFromBitmapER17AuOpenGLFunctionsRK6QImageb")
for anchor in ["sub    $0x1,%eax", "sub    $0x1,%ebx", "mov    %ebx,%r8d",
               "mov    %cl,-0x4(%rbp)", "mov    %al,-0x1(%rbp)"]:
    assert anchor in image, anchor
print("Verified skin selection, DNA tile offsets, kind7 texture control and bottom-up RGBA upload anchors. Static evidence, not rendered parity.")
