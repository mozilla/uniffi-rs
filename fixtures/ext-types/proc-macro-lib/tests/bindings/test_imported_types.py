# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import asyncio
import unittest
import urllib
from ext_types_guid import *
from imported_types_lib import *
from uniffi_one_ns import *

class TestIt(unittest.TestCase):
    def test_it(self):
        ct = get_combined_type(None)
        self.assertEqual(ct.uot.sval, "hello")
        self.assertEqual(ct.guid, "a-guid")
        self.assertEqual(ct.url.scheme, 'http')
        self.assertEqual(ct.url.netloc, 'example.com')
        self.assertEqual(ct.url.path, '/')

        ct2 = get_combined_type(ct)
        self.assertEqual(ct, ct2)

    def test_get_url(self):
        url = urllib.parse.urlparse("http://example.com/")
        self.assertEqual(get_url(url), url)
        self.assertEqual(get_urls([url]), [url])
        self.assertEqual(get_maybe_url(url), url)
        self.assertEqual(get_maybe_url(None), None)
        self.assertEqual(get_maybe_urls([url, None]), [url, None])

    def test_get_uniffi_one_type(self):
        t1 = UniffiOneType("hello")
        self.assertEqual(t1, get_uniffi_one_type(t1))
        self.assertEqual(t1, get_maybe_uniffi_one_type(t1))
        self.assertEqual(None, get_maybe_uniffi_one_type(None))
        self.assertEqual([t1], get_uniffi_one_types([t1]))
        self.assertEqual([t1, None], get_maybe_uniffi_one_types([t1, None]))
        async def test_async():
            self.assertEqual(t1, await get_uniffi_one_type_async(t1))
        asyncio.run(test_async())

    def test_get_uniffi_one_proc_macro_type(self):
        t1 = UniffiOneProcMacroType("hello")
        self.assertEqual(t1, get_uniffi_one_proc_macro_type(t1))

    def test_get_uniffi_one_enum(self):
        e = UniffiOneEnum.ONE
        self.assertEqual(e, get_uniffi_one_enum(e))
        self.assertEqual(e, get_maybe_uniffi_one_enum(e))
        self.assertEqual(None, get_maybe_uniffi_one_enum(None))
        self.assertEqual([e], get_uniffi_one_enums([e]))
        self.assertEqual([e, None], get_maybe_uniffi_one_enums([e, None]))

    def test_get_guid_procmacro(self):
        g = get_guid_procmacro(None)
        self.assertEqual(g, get_guid_procmacro(g))

    def test_get_customs(self):
        g = get_guid(None)
        self.assertEqual(g, get_guid(g))

        u = get_uuid(None)
        self.assertEqual(u, get_uuid(u))
        self.assertEqual(get_uuid_value(u), "new")

        h = get_newtype_handle(None)
        self.assertEqual(h, get_newtype_handle(h))
        self.assertEqual(get_newtype_handle_value(h), 42)

if __name__=='__main__':
    unittest.main()
