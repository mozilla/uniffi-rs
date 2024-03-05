# Regression test for a wrong lower-check

Up to v0.26.1 the lowering check was performed before coercing an optional default-valued argument,
thus causing the lowering check on the wrong type.
This only happened for Python.
