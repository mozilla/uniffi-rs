# Regression test for logging callback interfaces

This tests creating a callback interface to forward Rust logs.  Until version 0.23.0, this would result in infinite recursion, since we logged each callback interface method call.
