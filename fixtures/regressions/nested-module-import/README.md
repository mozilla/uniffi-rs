# Regression test for include scaffolding in a module

It is possible to include the generated scaffolding in a submodule,
then reexport everything on te top-level.

This recently broke with the introduction of `UniffiCustomTypeConverter` in the scaffolding as a private type.
The fix was easy: make the type public, so it is also re-exported properly.
