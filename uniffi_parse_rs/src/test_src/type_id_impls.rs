// Hand-written FFI trait impls for generic container types from non-UniFFI crates.
//
// `custom_type!` can't cover generic types.  Instead, the `TYPE_ID_META` const in a
// hand-written `TypeId` impl declares the wire shape, and paths to the target type
// resolve like the equivalent builtin container.

mod ffi {
    use fake_indexmap::{IndexMap, IndexSet, Reversed};

    impl<T> uniffi::TypeId<crate::UniFfiTag> for IndexSet<T>
    where
        T: uniffi::TypeId<crate::UniFfiTag>,
    {
        const TYPE_ID_META: uniffi::MetadataBuffer =
            uniffi::MetadataBuffer::from_code(uniffi::metadata::codes::TYPE_HASH_SET)
                .concat(T::TYPE_ID_META);
    }

    impl<K, V> uniffi::TypeId<crate::UniFfiTag> for IndexMap<K, V> {
        const TYPE_ID_META: uniffi::MetadataBuffer =
            uniffi::MetadataBuffer::from_code(uniffi::metadata::codes::TYPE_HASH_MAP)
                .concat(K::TYPE_ID_META)
                .concat(V::TYPE_ID_META);
    }

    // The concatenation order doesn't match the type's parameter order, so the builtin's
    // positional generics wouldn't line up: this impl must NOT be registered.
    impl<K, V> uniffi::TypeId<crate::UniFfiTag> for Reversed<K, V> {
        const TYPE_ID_META: uniffi::MetadataBuffer =
            uniffi::MetadataBuffer::from_code(uniffi::metadata::codes::TYPE_HASH_MAP)
                .concat(V::TYPE_ID_META)
                .concat(K::TYPE_ID_META);
    }
}

mod usage {
    use fake_indexmap::{IndexMap, IndexSet};

    #[derive(uniffi::Record)]
    pub struct UsesContainers {
        pub set: IndexSet<u32>,
        pub map: IndexMap<String, u32>,
    }
}
