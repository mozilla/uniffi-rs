# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Test namespace
import uniffi_docstring
assert uniffi_docstring.__doc__

from uniffi_docstring import *

# Test function
assert test.__doc__ == "<docstring-function>"
assert test_without_docstring.__doc__ is None

# Test enums
assert EnumTest.__doc__ == "<docstring-enum>"

# Simple enum variants can't be tested, because `__doc__` is not supported for enums
# assert EnumTest.ONE.__doc__ == "<docstring-enum-variant>"
# assert EnumTest.TWO.__doc__ == "<docstring-enum-variant-2>"

assert AssociatedEnumTest.__doc__ == "<docstring-associated-enum>"

# `__doc__` is lost because of how enum templates are generated
# https://github.com/mozilla/uniffi-rs/blob/eb97592f8c48a7f5cf02a94662b8b7861a6544f3/uniffi_bindgen/src/bindings/python/templates/EnumTemplate.py#L60
# assert AssociatedEnumTest.TEST.__doc__ == "<docstring-associated-enum-variant>"
# assert AssociatedEnumTest.TEST2.__doc__ == "<docstring-associated-enum-variant-2>"

# Test errors
assert ErrorTest.__doc__ == "<docstring-error>"
assert ErrorTest.One.__doc__ == "<docstring-error-variant>"
assert ErrorTest.Two.__doc__ == "<docstring-error-variant-2>"

assert AssociatedErrorTest.__doc__ == "<docstring-associated-error>"
assert AssociatedErrorTest.Test.__doc__ == "<docstring-associated-error-variant>"
assert AssociatedErrorTest.Test2.__doc__ == "<docstring-associated-error-variant-2>"

# Test objects
expected_object_doc = """
    <docstring-object>

      <docstring-object-indented-1>
      <docstring-object-indented-2>
    """
assert ObjectTest.__doc__ == expected_object_doc
assert ObjectTest.__init__.__doc__ == "<docstring-primary-constructor>"
assert ObjectTest.new_alternate.__doc__ == "<docstring-alternate-constructor>"
assert ObjectTest.test.__doc__ == "<docstring-method>"

# Test records
assert RecordTest.__doc__ == "<docstring-record>"

# `__doc__` is not supported for class fields
# assert RecordTest.test.__doc__ == "<docstring-record-field>"

# Test callbacks
assert CallbackTest.__doc__ == "<docstring-callback>"
assert CallbackTest.test.__doc__ == "<docstring-callback-method>"
