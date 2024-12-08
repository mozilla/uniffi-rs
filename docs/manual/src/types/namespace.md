# Namespace

Every crate has a UniFFI "namespace". This is the name exposed to the foreign bindings, typically as a module.

For example, if our namespace is `math`, and we have Rust code:

```rust
#[uniffi::export] // if using proc-macros
fn exp(a: f64) -> f64 { ... }
```

you might find foreign code doing:
```
from math import exp # python
```
```
import math // swift
```
```
import omg.wtf.math.exp // kotlin
```

If you use UDL, it *must* have a `namespace` block:

```idl
namespace math {
  double exp(double a);
};
```

Proc-macros don't need do anything unless the namespace name is different from the crate name, in which case it can be overridden with

```rust
uniffi::setup_scaffolding!("math");
```
