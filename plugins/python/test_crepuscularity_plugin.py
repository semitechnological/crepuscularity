import pathlib
import unittest

from crepuscularity_plugin import render_ir


class CrepuscularityPluginTests(unittest.TestCase):
    def test_render_ir(self):
        fixture = pathlib.Path(__file__).parents[1] / "fixtures" / "hello.crepus"
        ir = render_ir(fixture, {"name": "Ada"})
        self.assertEqual(ir.version, 2)
        self.assertEqual(ir.root[0]["children"][0]["content"], "Hello Ada")


if __name__ == "__main__":
    unittest.main()
