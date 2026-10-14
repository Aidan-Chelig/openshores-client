"""Actual server with one synthetic conversation-only native in the owned world.

The fixture is deliberately not evidence of native spawning or rendered bodies.
Protocol dispatch, nearest-native selection and conversation logic are real source.
The native source currently omits transport framing; this fixture explicitly
adapts only that egress defect. It is not an unmodified-server compatibility test.
"""
import runpy
from openshores.gameplay.natives import conversation, village
from openshores.network.chat_channel.operations import aucomm_envelope
from openshores.protocol.framing import encode_size

original_picker = conversation.pick_addressee
original_packet = conversation._pkt


def framed_native_packet(*args, **kwargs):
    payload = original_packet(*args, **kwargs)
    return encode_size(len(payload)) + payload


def fixture_picker(live_avatars, player):
    position = (live_avatars.get(player) or {}).get("xyz")
    if position:
        village.register_idle_body(0x7B000001, village.ROLE_ADULT,
                                   "Rust Local Native", 0, position, 0.0,
                                   village.DNA_DEFAULT_HUMAN, mins_to_full_grown=0)
        village._IDLE_BODIES[0x7B000001]["xyz"] = tuple(position)
    return original_picker(live_avatars, player)


aucomm_envelope.pick_addressee = fixture_picker
conversation._pkt = framed_native_packet
print("TEST FIXTURE: synthetic nearby native and explicit NPC-egress framing adapter", flush=True)
runpy.run_module("openshores", run_name="__main__")
