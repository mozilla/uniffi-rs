# Regression test for missing newline error in v0.19.1

In between v0.18.0 and v0.19.1 the Python code generation was changed,
causing invalid code to be generated in some very specific instances:

A void top-level function as well as the use of a hashmap somewhere.
It missed adding a newline in the right place, thus merging subsequent lines,
causing the code to be invalid.
