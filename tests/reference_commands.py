"""Exercise source scene handlers in memory; no sockets or database writes."""
import asyncio
from pathlib import Path
import sys
import struct
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.network.scene.operations.lookat import handle_0x18_lookat
from openshores.network.scene.operations.gear import handle_0x60_move_gear_slot
from openshores.protocol.atoms.item_seed import _pack_auitem_seed_body
from openshores.database.journal import get_queue
from openshores.protocol.delta_tick import parse


async def main():
    assert get_queue() is None, "offline fixture must not have a database journal"
    tick = parse(bytes([0x42, 4, 0, 0x70, 0, 0, 0]))
    assert tick == {"flags": 0x0400, "auid_400": 0x70000000, "_consumed": 7}
    report = parse(bytes([0x42, 0, 2]) + struct.pack(">6d", 100.5, 200.25, -300.125, 0.25, -0.5, 0.75))
    assert report == {"flags": 2, "projected": (100.5, 200.25, -300.125),
                      "rotation": (0.25, -0.5, 0.75), "_consumed": 51}
    session = SimpleNamespace(player_auid=42, lookat_target_auid=0, lookat_since_ts=0,
        augear=[[5, 0, 1, _pack_auitem_seed_body(1, 1)],
                [1, 0, 1, _pack_auitem_seed_body(1, 5)]])
    await handle_0x18_lookat(session, bytes([0x18, 0x89, 0xab, 0xcd, 0xef]), _DROPPED_ITEMS={})
    assert session.lookat_target_auid == 0x89ABCDEF and session.lookat_since_ts > 0
    await handle_0x18_lookat(session, bytes([0x18, 0, 0, 0, 0]), _DROPPED_ITEMS={})
    assert session.lookat_target_auid == 0
    refreshed = []
    async def refresh(avatar, **kwargs):
        refreshed.append(avatar)
    await handle_0x60_move_gear_slot(session, bytes([0x60, 5, 0, 0, 1]), _push_augear_refresh_for=refresh)
    assert session.augear[0][:2] == [1, 1]  # Server chooses first free subindex.
    assert session.augear[1][:2] == [1, 0]
    assert refreshed == [42]
    await handle_0x60_move_gear_slot(session, bytes([0x60, 5, 0, 0, 1]), _push_augear_refresh_for=refresh)
    assert refreshed == [42]  # Missing source is ignored, not acknowledged.
    print("Scene commands: look-at/clear, gear move, server-assigned subindex and missing-source behavior verified offline.")


if __name__ == "__main__":
    asyncio.run(main())
