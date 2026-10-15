"""Source-bound head MoveBy dispatch and TiltHeadTo constants, static only."""
import runpy
import struct

# Reuse the existing SHA-256-bound PE reader and concrete vtable checks.
context = runpy.run_path("tests/native_dp_connections.py")
read = context["read"]
assert struct.unpack("<Q", read(0x1423F97A0 + 0x68, 8))[0] == 0x1412B89F0
assert struct.unpack("<d", read(0x1422BFB00, 8))[0] == 1e-5
assert struct.unpack("<d", read(0x1422BFCB0, 8))[0] == -1e-5
for address, bits in [(0x1422BFA48, 0x3FC90FDB),
                      (0x1422BFA50, 0xBFC90FDB),
                      (0x1422BFB2C, 0x40490FDB)]:
    assert struct.unpack("<I", read(address, 4))[0] == bits
print("Verified head MoveBy virtual dispatch and TiltHeadTo threshold/angle constants. Static evidence, not runtime parity.")
