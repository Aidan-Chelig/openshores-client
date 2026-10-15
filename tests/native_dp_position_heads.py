"""Static PositionHeads switch/constant/type bindings; never executes target."""
import runpy
import struct

c = runpy.run_path("tests/native_dp_connections.py")
read = c["read"]
for table, default, special in [
    (0x1422BE900, 0x1412E3B3C, [0x1412E3C8A, 0x1412E3EF0, 0x1412E3FA4, 0x1412E3E70, 0x1412E3F59]),
    (0x1422BE9D4, 0x1412E3B78, [0x1412E3C52, 0x1412E3B70, 0x1412E3FAE, 0x1412E3EB0, 0x1412E3EFD]),
]:
    for pose in range(4, 57):
        expected = default
        for values, destination in zip([[4], range(18, 22), [23], range(49, 53), [56]], special):
            if pose in values:
                expected = destination
        assert table + struct.unpack("<i", read(table + 4 * (pose - 4), 4))[0] == expected
for address, bits in [(0x1422BFA90, 0x3CD67750), (0x1422BFC7C, 0x3E860A92),
                      (0x1422BFC90, 0x3C8EFA35), (0x1422BFE58, 0x3F32B8C2)]:
    assert struct.unpack("<I", read(address, 4))[0] == bits
for vptr, _, _ in c["bindings"]:
    method = struct.unpack("<Q", read(vptr + 0x40, 8))[0]
    if vptr == 0x1423F97A0:
        assert method == 0x14190EA10 and read(method, 6) == bytes.fromhex("b8 01 00 00 00 c3")
    else:
        assert method == 0x1418F7140 and read(method, 3) == bytes.fromhex("31 c0 c3")
print("Verified both 53-entry head pose switch tables, four float constants and seven IsHeadAndNeck bindings; static only.")
