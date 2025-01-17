# How do UniFFI handles and handle maps work?

UniFFI uses handles to pass Rust objects across the FFI to the foreign code.
The handles point to an entry inside a `HandleMap`

## HandleMap

A `HandleMap` is a `Vec` where each item is either:

- **Occupied**
  - The foreign side holds a handle that's associated with the entry.
  - Stores a `T` value (typically an `Arc<>`).
  - Stores a `generation` counter used to detect use-after-free bugs
  - Stores a `map-id` value used to detect handles used with the wrong `HandleMap`
- **Vacant**
  - Each vacant entry stores the index of the next vacant entry.
    These form a linked-list of available entries and allow us to quickly allocate a new entry in the `HandleMap`
  - Stores the `generation` counter value from the last time it was occupied, or 0 if it was never occupied.

Furthermore, each `HandleMap` stores its own `next` value, which points to the first vacant entry on the free list.
The value u32::MAX indicates indicates there is no next item and is represented by the `EOL` const.

Here's an example `HandleMap`:

```
----------------------------------------------------------------
| OCCUPIED           | VACANT             |  OCCUPIED          |
| item: Foo          | next: EOL          |  item: Bar         |
| generation: 100    | generation: 40     |  generation: 30    |
----------------------------------------------------------------

HandleMap.next: 1
```

### Inserting entries

To insert a new entry:
  - Remove the first entry in the free list, 
  - Convert it to an `OCCUPIED`
  - Increment the generation counter

For example inserting a `Baz` entry in the above `HandleMap` would result in:

```
----------------------------------------------------------------
| OCCUPIED           | OCCUPIED           |  OCCUPIED          |
| item: Foo          | item: Baz          |  item: Bar         |
| generation: 100    | generation: 41     |  generation: 30    |
----------------------------------------------------------------

HandleMap.next: EOL
```

If there are no vacant entries, then we append an entry to the end of the list.
For example, inserting a `Qux` entry in the above `HandleMap` would result in:

```
-------------------------------------------------------------------------------------
| OCCUPIED           | OCCUPIED           |  OCCUPIED          | OCCUPIED           |
| item: Foo          | item: Baz          |  item: Bar         | item: Qux          |
| generation: 100    | generation: 41     |  generation: 30    | generation: 0      |
-------------------------------------------------------------------------------------

HandleMap.next: EOL
```

### Removing entries

To remove an entry:
  - Convert it to `VACANT`
  - Add it to the head of the free list.

For example, removing the `Foo` entry from the above handle map would result in:

```
-------------------------------------------------------------------------------------
| VACANT             | OCCUPIED           |  OCCUPIED          | OCCUPIED           |
| next: EOL          | item: Baz          |  item: Bar         | item: Qux          |
| generation: 100    | generation: 41     |  generation: 30    | generation: 0      |
-------------------------------------------------------------------------------------

HandleMap.next: 0
```

Removing the `Bar` entry after that would result in:

```
-------------------------------------------------------------------------------------
| VACANT             | OCCUPIED           |  OCCUPIED          | OCCUPIED           |
| next: EOL          | item: Baz          |  next: 0           | item: Qux          |
| generation: 100    | generation: 41     |  generation: 30    | generation: 0      |
-------------------------------------------------------------------------------------

HandleMap.next: 2
```

### Getting entries

When an entry is inserted, we return a `Handle`.
This is a 64-bit integer, segmented as follows:
- Bits 0-32: `Vec` index
- Bit 32: foreign bit that's set for handles for foreign objects, but not Rust objects.
  This allows us to differentiate trait interface implementations.
- Bits 33-40: map id -- a unique value that corresponds to the map that generated the handle
- Bits 40-48: generation counter
- Bits 48-64: unused

When the foreign code passes the Rust code a handle, we use it to get the entry as follows:

- Use the index to get the entry in the raw `Vec`
- Check that the entry is `OCCUPIED`, the generation counter value matches, and the map_id matches.
- Get the stored item and do something with it.  Usually this means cloning the `Arc<>`.

These checks can usually ensure that handles are only used with the `HandleMap` that generated them and that they aren't used after the entry is removed.
However, this is limited by the bit-width of the handle segments:

- Because the generation counter is 8-bits, we will fail to detect use-after-free bugs if an entry has been reused exactly 256 items or some multiple of 256.
- Because the map id is only 7-bits, we may fail to detect handles being used with the wrong map if we generate over 128 `HandleMap` tables.
  This can only happen if there more than 100 user-defined types and less than 1% of the time in that case.

### Handle map creation / management

The Rust codegen creates a static `HandleMap` for each object type that needs to be sent across the FFI, for example:
  - `HandleMap<Arc<T>>` for each object type exposed by UniFFI.
  - `HandleMap<Arc<dyn Trait>>` for each trait interface exposed by UniFFI.
  - `HandleMap<Arc<dyn RustFutureFfi<FfiType>>>` for each FFI type.  This is used to implement async Rust functions.
  - `HandleMap<oneshot::Sender<FfiType>>` for each FFI type.  This will be used to implement async callback methods.

The `HandleAlloc` trait manages access to the static `HandleMap` instances and provides the following methods:
  - `insert(value: Self) -> Handle` insert a new entry into the handle map
  - `remove(handle: Handle) -> Self` remove an entry from the handle map
  - `get(handle: Handle) -> Self` get a cloned object from the handle map without removing the entry.
  - `clone_handle(handle: Handle) -> Handle` get a cloned handle that refers to the same object.

If the user defines a type `Foo` in their interface then:
 - `<Arc<Foo> as HandleAlloc>::insert` is called when lowering `Foo` to pass it across the FFI to the foreign side.
 - `<Arc<Foo> as HandleAlloc>::get` is called when lifting `Foo` after it's passed back across the FFI to Rust.
 - `<Arc<Foo> as HandleAlloc>::clone_handle` is called when the foreign side needs to clone the handle.
   See https://github.com/mozilla/uniffi-rs/issues/1797 for why this is needed.
 - `<Arc<Foo> as HandleAlloc>::remove` is called when the foreign side calls the `free` function for the object.

Extra details:
  - The trait is actually `HandleAlloc<UT>`, where `UT` is the "UniFFI Tag" type. See `uniffi_core/src/ffi_converter_traits.rs` for details.
  - The last two `HandleAlloc` methods are only implemented for `T: Clone`, which is true for the `Arc<>` cases, but not `oneshot::Sender`.
    This is fine because we only use `insert`/`remove` for the `oneshot::Sender` case.

### Concurrency

`insert` and `remove` require serialization since there's no way to atomically update the free list.
In general, `get` does not require any serialization since it will only read occupied entries, while `insert` and `remove` only access vacant entries.
However, there are 2 edge cases where `get` does access the same entries as `insert`/`remove`:

  - If `insert` causes the Vec to grow this may cause the entire array to be moved, which will affect `get`
  - If the foreign code has a use-after-free bug, then `get` may access the same entry as an `insert`/`remove` operation.

UniFFI uses the following system to handle this:

- A standard `Mutex` is used to serialize `insert` and `remove`.
- We use the `append_only_vec` crate, which avoids moving the array when the `Vec` grows.
- Each entry has a 8-bit read-write spin-lock to avoid issues in the face of use-after-free bugs.
  This lock will only be contested if there's a use-after-free bug.

### Concurrency: alternative option if we choose 2 from the ADR.  This one is simpler, but slower.

To allow concurrent access, a `RwLock` is used to protect the entire `HandleMap`.
`insert` and `remove` acquire the write lock while accessing entries acquires the read lock.

### Space usage

The `HandleMap` adds an extra 64-bits of memory to each occupied item, which is the lower-limit on a 64-bit machine.
This means that `HandleMap` tables that store normal `Arc` pointers add ~100% extra space overhead and ones that store wide-pointers add ~50% overhead.
`HandleMap` tables don't have any way of reclaiming unused space after items are removed.

This is can be a very large amount of memory, but in practice libraries only generate a relatively small amount of handles.
