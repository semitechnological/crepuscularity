import unittest

from crepuscularity_abi import CrepusAbiSession


class CrepuscularityAbiTests(unittest.TestCase):
    def test_render_and_dispatch_event(self):
        events = []
        with CrepusAbiSession() as session:
            session.set_template('input bind=count\nspan\n  "Count {count}"')
            session.set_context({"count": "1"})
            session.on_event(events.append)
            first = session.render_ir()
            self.assertEqual(first["version"], 4)
            self.assertIn("Count 1", str(first))
            result = session.dispatch_event({"handler": "bind:count:2"})
            self.assertEqual(result["handler"], "bind:count:2")
            self.assertIn("Count 2", str(result["ir"]))
        self.assertEqual(events[0]["handler"], "bind:count:2")


if __name__ == "__main__":
    unittest.main()
