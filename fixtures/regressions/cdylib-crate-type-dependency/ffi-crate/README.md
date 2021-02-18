# Regression test for issue #356

In pull request [#388](https://github.com/mozilla/uniffi-rs/pull/388)
we discovered that when we have a dependency which contains cdylib as its crate-type, 
this dependency will end up in our cdylib lists, which we can use `pkg_dir` to exclude.

This crate is a minimal reproduction of the issue, and its would fail to compile in 
the presence of the bug.
