# Regression test for issue #1015

In issue [#1015](https://github.com/mozilla/uniffi-rs/issues/1015)
we discovered that the generated scaffolding code was assuming that
some std types were in scope, which makes it fragile to changes in
the containing crate's Rust code. The scaffolding should instead
use fully-qualified type names.

This crate is a minimal reproduction of the issue, which will fail
to compile if the scaffolding depends on particular types being
in scope.

There deliberately aren't any tests in this crate; the test is
whether or not it compiles successfully. If you find that this crate
no longer compiles, you've probably added some generated scaffolding
code that is depending on a particular type name being in scope.
Change it to use a fully-qualified name, e.g. `std::collections::HashMap`
instead of just `HashMap`.