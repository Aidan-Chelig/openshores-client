"""Offline nested gear parity with supplied Python writers and unpacker."""
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.protocol.atoms.gear import _pack_au_gear, _unpack_au_gear
from openshores.protocol.atoms.gear_variety import _build_variety_gear_entries
from openshores.protocol.atoms.item_seed import _pack_auitem_seed_body
from openshores.protocol.atoms.container import _container_encode_body
from openshores.protocol.atoms.person import _build_augear_only_daperson_update
from openshores.protocol.framing import encode_size


def main():
    entries = _build_variety_gear_entries()
    leaf = _pack_auitem_seed_body(1, 130, name="Nested 🚀")
    box = _container_encode_body(_pack_auitem_seed_body(1, 2000), 16, [[77, 1, leaf]])
    flora = _pack_auitem_seed_body(14, 130, for_world=True)
    container = (_pack_auitem_seed_body(1, 2001) + struct.pack(">iiB", 4, 4, 1)
                 + bytes([9, 4, 14]) + flora)
    entries += [(1, 2, 18, box), (1, 3, 11, container)]
    gear = _pack_au_gear(entries)
    decoded = _unpack_au_gear(gear)
    with tempfile.TemporaryDirectory(prefix="openshores-inventory-") as directory:
        path = Path(directory) / "gear.frames"
        def inspect(blob):
            packet = _build_augear_only_daperson_update(struct.pack(">I", 42), blob,
                _stamina_byte=lambda _: 90, agent_bits_for=lambda _: 5)
            path.write_bytes(encode_size(len(packet)) + packet)
            return subprocess.run([str(ROOT / "target/debug/openshores-client"), "inspect-scene", str(path)],
                                  capture_output=True, text=True, check=True, timeout=30).stdout
        output = inspect(gear)
        assert f"declared_slots: {len(decoded)}" in output
        assert "stop: None, unparsed_suffix: []" in output, output
        for slot, sub, type_id, body in decoded:
            assert f"slot_type: {slot}, subindex: {sub}, item: Some(InventoryItem {{ type_id: {type_id}" in output
            assert "raw_body: " + str(list(body)) in output, (type_id, body, output)
        assert "metadata: [77]" in output and "metadata: [9, 4]" in output, output
        assert 'name: Some(Some("Nested 🚀"))' in output
        empty = inspect(bytes([1, 0x25, 0]))
        assert "slot_type: 5, subindex: 2, item: None" in empty
        unknown = inspect(bytes([2, 0x25, 0, 0x15, 0xfe, 9, 8]))
        assert "unknown inventory type 0xfe" in unknown
        assert "unparsed_suffix: [21, 254, 9, 8]" in unknown
    print("Inventory: Python variety gear, weapon boundaries, nested box/container/flora, empty slots and unknown stops verified offline.")


if __name__ == "__main__":
    main()
