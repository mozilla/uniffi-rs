# Default arguments in struct constructors

We support default values for any field in a struct/dictionary.
Fields with default values can come before fields without any.
This always worked for Kotlin and Swift.
Kotlin allows optionally calling with named arguments.
Swift enforces (by default) named arguments.

However this broke for Python,
because one cannot specify any non-defaulted arguments after defaulted ones.
This is now fixed and Python enforces named arguments.
