from external_types import *
import unittest

class TestExternalTypes(unittest.TestCase):
    def test_guid(self):
        guid = get_guid(None)
        self.assertEqual(type(guid), str)

        guid2 = get_guid(guid)
        self.assertEqual(guid, guid2)

    def test_json_object(self):
        j = get_json_object(None)
        self.assertEqual(type(j), dict)

        j2 = get_json_object(j)
        self.assertEqual(type(j2), dict)
        self.assertEqual(j, j2)

    def test_ext_types(self):
        vals = get_ext_types(None)

        self.assertEqual(type(vals.guid), str)
        self.assertEqual(vals.guid, "first-guid")
        self.assertEqual(type(vals.guids), list)
        self.assertEqual(vals.guids, ["second-guid", "third-guid"])

        self.assertEqual(type(vals.json), dict)
        self.assertEqual(type(vals.jsons), list)
        # first elt is a json array
        self.assertEqual(vals.jsons[0], ["an", "array"])
        # second a plain int.
        self.assertEqual(vals.jsons[1], 3)

        vals2 = get_ext_types(vals)
        self.assertEqual(vals, vals2)

if __name__=='__main__':
    unittest.main()
