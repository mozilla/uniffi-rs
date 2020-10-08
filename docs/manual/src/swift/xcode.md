# Integrating with XCode

It is possible to generate Swift bindings at compile time for XCode projects.
In your project configuration, add a "Run Script Phase" that applies to `*.udl` files:

```bash
$HOME/.cargo/bin/uniffi-bindgen generate $INPUT_FILE_PATH --language swift --out-dir $DERIVED_FILE_DIR\n
```
