# A "Decorators" test for uniffi components

This is similar to the `decorators` example, but it's intended to be contrived and to
ensure we get good test coverage of all possible options.

It's here so it doesn't even need to make a cursory effort to be a "good"
example; it intentionally panics, asserts params are certain values, has
no-op methods etc. If you're trying to get your head around uniffi then the
"examples" directory will be a much better bet.

This is its own crate, because the decorator mechanism is not implemented for all backends yet.
