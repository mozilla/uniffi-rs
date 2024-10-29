# Rules for the Python template code

## Naming

Private variables, classes, functions, etc. should be prefixed with `_uniffi`, `_Uniffi`, or `_UNIFFI`.
The `_` indicates the variable is private and removes it from the autocomplete list.
The `uniffi` part avoids naming collisions with user-defined items.
Users can use leading underscores in their names if they want, but "uniffi" is reserved for our purposes.

In particular, make sure to use the `_uniffi` prefix for any variable names in generated functions.
If you name a variable something like `result` the code will probably work initially.
Then it will break later on when a user decides to define a function with a parameter named `result`.

Note: this doesn't apply to items that we want to expose, for example users may want to catch `InternalError` so doesn't get the `Uniffi` prefix.
