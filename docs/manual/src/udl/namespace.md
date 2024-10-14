# Namespace

Every crate has a UniFFI "namespace". This is the name exposed to the foreign bindings, typically as a module.

Every UDL file *must* have a `namespace` block:

```idl
namespace math {
  double exp(double a);
};
```

which might be used as `from math import exp`/`import math`/`import omg.wtf.math.exp` etc.

Proc macros use the crate name as the namespace by default, but it can be specified with

```rust
uniffi::setup_scaffolding!("namespace");
```
