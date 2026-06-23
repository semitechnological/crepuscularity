import pathlib
import unittest

from crepuscularity_plugin import ViewSession, render_html, render_ir


class CrepuscularityPluginTests(unittest.TestCase):
    def test_render_ir(self):
        fixture = pathlib.Path(__file__).parents[1] / "fixtures" / "hello.crepus"
        ir = render_ir(fixture, {"name": "Ada"})
        self.assertEqual(ir.version, 4)
        self.assertEqual(ir.root[0]["children"][0]["content"], "Hello Ada")
        self.assertEqual(render_html(fixture, {"name": "Ada"}), '<div data-crepus-kind="stack" data-axis="column">Hello Ada</div>')

    def test_view_session_dispatches_bind_and_rerenders(self):
        fixture = pathlib.Path(__file__).parents[1] / "fixtures" / "interactive.crepus"
        session = ViewSession(fixture, {"count": "1"})
        self.assertIn("Count 1", session.render_html())
        ir = session.dispatch("bind:count:2")
        self.assertEqual(session.context["count"], "2")
        self.assertIn("Count 2", str(ir.root))
        self.assertIn("Count 2", session.render_html())


if __name__ == "__main__":
    unittest.main()
