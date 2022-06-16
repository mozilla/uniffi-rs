These fixtures are designed to test that keywords from supported languages
are able to be used.

Each directory is a specialized fixture for each language (although the
`rust` directory does double-duty for Python)

The reason we don't try and combine these into a single fixture is that we'd
like a highe degree is assurance that each language has a keyword used
in every possible context. By trying to combine them, we would end up needing
multiple, say, `enum`, `interface`, function arguments, variant discriminators,
etc. So there's a reasonable change we'd accidently not have an `enum` with a
(say) kotlin keyword.

Separate fixtures means someone familiar with Kotlin and look at 1 fixture and
fairly easily verify all cases are covered.

Feel free to have a poke at uniffi(hah)-ing them though!
