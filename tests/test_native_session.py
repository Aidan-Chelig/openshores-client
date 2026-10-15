"""Transport tests without REA, Ghidra, network or target execution."""
import io
import json
from types import SimpleNamespace
import unittest
from native_session import Rpc


class SessionTransportTests(unittest.TestCase):
    def test_notifications_do_not_end_or_restart_request(self):
        process = SimpleNamespace(stdin=io.StringIO(), stdout=io.StringIO(
            '{"jsonrpc":"2.0","method":"notifications/progress","params":{}}\n'
            '{"jsonrpc":"2.0","id":1,"result":{"complete":true}}\n'
            '{"jsonrpc":"2.0","id":2,"result":{"next":true}}\n'))
        rpc = Rpc(process)
        self.assertEqual(rpc.request("first", {}), {"complete": True})
        self.assertEqual(rpc.request("second", {}), {"next": True})
        sent = [json.loads(line) for line in process.stdin.getvalue().splitlines()]
        self.assertEqual([message["id"] for message in sent], [1, 2])

    def test_terminal_eof_is_an_error_not_implicit_relaunch(self):
        process = SimpleNamespace(stdin=io.StringIO(), stdout=io.StringIO())
        with self.assertRaisesRegex(RuntimeError, "exited"):
            Rpc(process).request("query", {})

    def test_protocol_errors_are_not_successful_evidence(self):
        process = SimpleNamespace(stdin=io.StringIO(), stdout=io.StringIO(
            '{"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"bad"}}\n'))
        with self.assertRaises(RuntimeError):
            Rpc(process).request("query", {})


if __name__ == "__main__":
    unittest.main()
