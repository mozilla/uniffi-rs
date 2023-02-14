# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
import urllib
from imported_types_lib import *
from uniffi_one import *

class CallbackInterfaceImpl(UniffiOneCallbackInterface):
    def on_done(self, done_val):
        return f"{done_val}-frompython"

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

    def test_get_uniffi_one_enum(self):
        e = UniffiOneEnum.ONE
        self.assertEqual(e, get_uniffi_one_enum(e))
        self.assertEqual(e, get_maybe_uniffi_one_enum(e))
        self.assertEqual(None, get_maybe_uniffi_one_enum(None))
        self.assertEqual([e], get_uniffi_one_enums([e]))
        self.assertEqual([e, None], get_maybe_uniffi_one_enums([e, None]))

    def test_get_uniffi_one_interface(self):
        assert(isinstance(get_uniffi_one_interface(), UniffiOneInterface))

    def test_use_uniffi_one_callback_interface(self):
        cb_interface = CallbackInterfaceImpl()
        self.assertEqual(use_uniffi_one_callback_interface(cb_interface), "fromrust-frompython")




if __name__=='__main__':
    unittest.main()
