namespace {{ context.detail_name() }} {

/// Estimates the worst-case UTF-8 encoded length for a UTF-16 string.
CheckedInt<size_t> EstimateUTF8Length(size_t aUTF16Length) {
  // `ConvertUtf16toUtf8` expects the destination to have at least three times
  // as much space as the source string, even if it doesn't use the excess
  // capacity.
  CheckedInt<size_t> length(aUTF16Length);
  length *= 3;
  return length;
}

/// Reads values out of a byte buffer received from Rust.
class MOZ_STACK_CLASS Reader final {
 public:
  explicit Reader(const {{ context.ffi_rustbuffer_type() }}& aBuffer) : mBuffer(aBuffer), mOffset(0) {}

  /// Returns `true` if there are unread bytes in the buffer, or `false` if the
  /// current position has reached the end of the buffer. If `HasRemaining()`
  /// returns `false`, attempting to read any more values from the buffer will
  /// assert.
  bool HasRemaining() { return mOffset.value() < mBuffer.mLen; }

  /// `Read{U}Int{8, 16, 32, 64}` read fixed-width integers from the buffer at
  /// the current position.

  uint8_t ReadUInt8() {
    return ReadAt<uint8_t>(
        [this](size_t aOffset) { return mBuffer.mData[aOffset]; });
  }

  int8_t ReadInt8() { return BitwiseCast<int8_t>(ReadUInt8()); }

  uint16_t ReadUInt16() {
    return ReadAt<uint16_t>([this](size_t aOffset) {
      uint16_t value;
      memcpy(&value, &mBuffer.mData[aOffset], sizeof(uint16_t));
      // `PR_ntohs` ("network to host, short") because the UniFFI serialization
      // format encodes integers in big-endian order (also called
      // "network byte order").
      return PR_ntohs(value);
    });
  }

  int16_t ReadInt16() { return BitwiseCast<int16_t>(ReadUInt16()); }

  uint32_t ReadUInt32() {
    return ReadAt<uint32_t>([this](size_t aOffset) {
      uint32_t value;
      memcpy(&value, &mBuffer.mData[aOffset], sizeof(uint32_t));
      return PR_ntohl(value);
    });
  }

  int32_t ReadInt32() { return BitwiseCast<int32_t>(ReadUInt32()); }

  uint64_t ReadUInt64() {
    return ReadAt<uint64_t>([this](size_t aOffset) {
      uint64_t value;
      memcpy(&value, &mBuffer.mData[aOffset], sizeof(uint64_t));
      return PR_ntohll(value);
    });
  }

  int64_t ReadInt64() { return BitwiseCast<int64_t>(ReadUInt64()); }

  /// `Read{Float, Double}` reads a floating-point number from the buffer at
  /// the current position.

  float ReadFloat() { return BitwiseCast<float>(ReadUInt32()); }

  double ReadDouble() { return BitwiseCast<double>(ReadUInt64()); }

  /// Reads a sequence or record length from the buffer at the current position.
  size_t ReadLength() {
    // The UniFFI serialization format uses signed integers for lengths.
    auto length = ReadInt32();
    MOZ_RELEASE_ASSERT(length >= 0);
    return static_cast<size_t>(length);
  }

  /// Reads a UTF-8 encoded string at the current position.
  void ReadCString(nsACString& aValue) {
    auto length = ReadInt32();
    CheckedInt<int32_t> newOffset = mOffset;
    newOffset += length;
    AssertInBounds(newOffset);
    aValue.Append(AsChars(Span(&mBuffer.mData[mOffset.value()], length)));
    mOffset = newOffset;
  }

  /// Reads a UTF-16 encoded string at the current position.
  void ReadString(nsAString& aValue) {
    auto length = ReadInt32();
    CheckedInt<int32_t> newOffset = mOffset;
    newOffset += length;
    AssertInBounds(newOffset);
    AppendUTF8toUTF16(AsChars(Span(&mBuffer.mData[mOffset.value()], length)),
                      aValue);
    mOffset = newOffset;
  }

 private:
  void AssertInBounds(const CheckedInt<int32_t>& aNewOffset) const {
    MOZ_RELEASE_ASSERT(aNewOffset.isValid() &&
                       aNewOffset.value() <= mBuffer.mLen);
  }

  template <typename T>
  T ReadAt(const std::function<T(size_t)>& aClosure) {
    CheckedInt<int32_t> newOffset = mOffset;
    newOffset += sizeof(T);
    AssertInBounds(newOffset);
    T result = aClosure(mOffset.value());
    mOffset = newOffset;
    return result;
  }

  const {{ context.ffi_rustbuffer_type() }}& mBuffer;
  CheckedInt<int32_t> mOffset;
};

/// Writes values into a Rust buffer.
class MOZ_STACK_CLASS Writer final {
 public:
  Writer() {
    {{ context.ffi_rusterror_type() }} err = {0, nullptr};
    mBuffer = {{ ci.ffi_rustbuffer_alloc().name() }}(0, &err);
    if (err.mCode) {
      MOZ_ASSERT(false, "Failed to allocate empty Rust buffer");
    }
  }

  /// `Write{U}Int{8, 16, 32, 64}` write fixed-width integers into the buffer at
  /// the current position.

  void WriteUInt8(const uint8_t& aValue) {
    WriteAt<uint8_t>(aValue, [](uint8_t* aBuffer, const uint8_t& aValue) {
      *aBuffer = aValue;
    });
  }

  void WriteInt8(const int8_t& aValue) {
    WriteUInt8(BitwiseCast<uint8_t>(aValue));
  }

  void WriteUInt16(const uint16_t& aValue) {
    WriteAt<uint16_t>(aValue, [](uint8_t* aBuffer, const uint16_t& aValue) {
      // `PR_htons` ("host to network, short") because, as mentioned above, the
      // UniFFI serialization format encodes integers in big-endian (network
      // byte) order.
      uint16_t value = PR_htons(aValue);
      memcpy(aBuffer, &value, sizeof(uint16_t));
    });
  }

  void WriteInt16(const int16_t& aValue) {
    WriteUInt16(BitwiseCast<uint16_t>(aValue));
  }

  void WriteUInt32(const uint32_t& aValue) {
    WriteAt<uint32_t>(aValue, [](uint8_t* aBuffer, const uint32_t& aValue) {
      uint32_t value = PR_htonl(aValue);
      memcpy(aBuffer, &value, sizeof(uint32_t));
    });
  }

  void WriteInt32(const int32_t& aValue) {
    WriteUInt32(BitwiseCast<uint32_t>(aValue));
  }

  void WriteUInt64(const uint64_t& aValue) {
    WriteAt<uint64_t>(aValue, [](uint8_t* aBuffer, const uint64_t& aValue) {
      uint64_t value = PR_htonll(aValue);
      memcpy(aBuffer, &value, sizeof(uint64_t));
    });
  }

  void WriteInt64(const int64_t& aValue) {
    WriteUInt64(BitwiseCast<uint64_t>(aValue));
  }

  /// `Write{Float, Double}` writes a floating-point number into the buffer at
  /// the current position.

  void WriteFloat(const float& aValue) {
    WriteUInt32(BitwiseCast<uint32_t>(aValue));
  }

  void WriteDouble(const double& aValue) {
    WriteUInt64(BitwiseCast<uint64_t>(aValue));
  }

  /// Writes a sequence or record length into the buffer at the current
  /// position.
  void WriteLength(size_t aValue) {
    MOZ_RELEASE_ASSERT(
        aValue <= static_cast<size_t>(std::numeric_limits<int32_t>::max()));
    WriteInt32(static_cast<int32_t>(aValue));
  }

  /// Writes a UTF-8 encoded string at the current offset.
  void WriteCString(const nsACString& aValue) {
    CheckedInt<size_t> size(aValue.Length());
    size += sizeof(uint32_t);  // For the length prefix.
    MOZ_RELEASE_ASSERT(size.isValid());
    Reserve(size.value());

    // Write the length prefix first...
    uint32_t lengthPrefix = PR_htonl(aValue.Length());
    memcpy(&mBuffer.mData[mBuffer.mLen], &lengthPrefix, sizeof(uint32_t));

    // ...Then the string. We just copy the string byte-for-byte into the
    // buffer here; the Rust side of the FFI will ensure it's valid UTF-8.
    memcpy(&mBuffer.mData[mBuffer.mLen + sizeof(uint32_t)],
           aValue.BeginReading(), aValue.Length());

    mBuffer.mLen += static_cast<int32_t>(size.value());
  }

  /// Writes a UTF-16 encoded string at the current offset.
  void WriteString(const nsAString& aValue) {
    auto maxSize = EstimateUTF8Length(aValue.Length());
    maxSize += sizeof(uint32_t);  // For the length prefix.
    MOZ_RELEASE_ASSERT(maxSize.isValid());
    Reserve(maxSize.value());

    // Convert the string to UTF-8 first...
    auto span = AsWritableChars(Span(
        &mBuffer.mData[mBuffer.mLen + sizeof(uint32_t)], aValue.Length() * 3));
    size_t bytesWritten = ConvertUtf16toUtf8(aValue, span);

    // And then write the length prefix, with the actual number of bytes
    // written.
    uint32_t lengthPrefix = PR_htonl(bytesWritten);
    memcpy(&mBuffer.mData[mBuffer.mLen], &lengthPrefix, sizeof(uint32_t));

    mBuffer.mLen += static_cast<int32_t>(bytesWritten) + sizeof(uint32_t);
  }

  /// Returns the buffer.
  {{ context.ffi_rustbuffer_type() }} Buffer() { return mBuffer; }

 private:
  /// Reserves the requested number of bytes in the Rust buffer, aborting on
  /// allocation failure.
  void Reserve(size_t aBytes) {
    if (aBytes >= static_cast<size_t>(std::numeric_limits<int32_t>::max())) {
      NS_ABORT_OOM(aBytes);
    }
    {{ context.ffi_rusterror_type() }} err = {0, nullptr};
    {{ context.ffi_rustbuffer_type() }} newBuffer = {{ ci.ffi_rustbuffer_reserve().name() }}(
      mBuffer, static_cast<int32_t>(aBytes), &err);
    if (err.mCode) {
      NS_ABORT_OOM(aBytes);
    }
    mBuffer = newBuffer;
  }

  template <typename T>
  void WriteAt(const T& aValue,
               const std::function<void(uint8_t*, const T&)>& aClosure) {
    Reserve(sizeof(T));
    aClosure(&mBuffer.mData[mBuffer.mLen], aValue);
    mBuffer.mLen += sizeof(T);
  }

  {{ context.ffi_rustbuffer_type() }} mBuffer;
};

/// A "trait" struct with specializations for types that can be read and
/// written into a byte buffer. This struct is specialized for all serializable
/// types.
template <typename T>
struct Serializable {
  /// Reads a value of type `T` from a byte buffer.
  static bool ReadFrom(Reader& aReader, T& aValue) = delete;

  /// Writes a value of type `T` into a byte buffer.
  static void WriteInto(Writer& aWriter, const T& aValue) = delete;
};

/// A "trait" with specializations for types that can be transferred back and
/// forth over the FFI. This is analogous to the Rust trait of the same name.
/// As above, this gives us compile-time type checking for type pairs. If
/// `ViaFfi<T, U>::Lift(U, T)` compiles, we know that a value of type `U` from
/// the FFI can be lifted into a value of type `T`.
///
/// The `Nullable` parameter is used to specialize nullable and non-null
/// strings, which have the same `T` and `FfiType`, but are represented
/// differently over the FFI.
template <typename T, typename FfiType, bool Nullable = false>
struct ViaFfi {
  /// Converts a low-level `FfiType`, which is a POD (Plain Old Data) type that
  /// can be passed over the FFI, into a high-level type `T`.
  ///
  /// `T` is passed as an "out" parameter because some high-level types, like
  /// dictionaries, can't be returned by value.
  static bool Lift(const FfiType& aLowered, T& aLifted) = delete;

  /// Converts a high-level type `T` into a low-level `FfiType`. `FfiType` is
  /// returned by value because it's guaranteed to be a POD, and because it
  /// simplifies the `ViaFfi::Lower` calls that are generated for each argument
  /// to an FFI function.
  static FfiType Lower(const T& aLifted) = delete;
};

// This macro generates boilerplate specializations for primitive numeric types
// that are passed directly over the FFI without conversion.
#define UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(Type, readFunc, writeFunc)  \
  template <>                                                                \
  struct Serializable<Type> {                                                \
    [[nodiscard]] static bool ReadFrom(Reader& aReader, Type& aValue) {      \
      aValue = aReader.readFunc();                                           \
      return true;                                                           \
    }                                                                        \
    static void WriteInto(Writer& aWriter, const Type& aValue) {             \
      aWriter.writeFunc(aValue);                                             \
    }                                                                        \
  };                                                                         \
  template <>                                                                \
  struct ViaFfi<Type, Type> {                                                \
    [[nodiscard]] static bool Lift(const Type& aLowered, Type& aLifted) {    \
      aLifted = aLowered;                                                    \
      return true;                                                           \
    }                                                                        \
    [[nodiscard]] static Type Lower(const Type& aLifted) { return aLifted; } \
  }

UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(uint8_t, ReadUInt8, WriteUInt8);
UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(int8_t, ReadInt8, WriteInt8);
UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(uint16_t, ReadUInt16, WriteUInt16);
UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(int16_t, ReadInt16, WriteInt16);
UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(uint32_t, ReadUInt32, WriteUInt32);
UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(int32_t, ReadInt32, WriteInt32);
UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(uint64_t, ReadUInt64, WriteUInt64);
UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(int64_t, ReadInt64, WriteInt64);
UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(float, ReadFloat, WriteFloat);
UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(double, ReadDouble, WriteDouble);

#undef UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE

/// In the UniFFI serialization format, Booleans are passed as `int8_t`s over
/// the FFI.

template <>
struct Serializable<bool> {
  [[nodiscard]] static bool ReadFrom(Reader& aReader, bool& aValue) {
    aValue = aReader.ReadInt8() != 0;
    return true;
  }
  static void WriteInto(Writer& aWriter, const bool& aValue) {
    aWriter.WriteInt8(aValue ? 1 : 0);
  }
};

template <>
struct ViaFfi<bool, int8_t> {
  [[nodiscard]] static bool Lift(const int8_t& aLowered, bool& aLifted) {
    aLifted = aLowered != 0;
    return true;
  }
  [[nodiscard]] static int8_t Lower(const bool& aLifted) {
    return aLifted ? 1 : 0;
  }
};

/// Strings are length-prefixed and UTF-8 encoded when serialized
/// into Rust buffers, and are passed as UTF-8 encoded `RustBuffer`s over
/// the FFI.

/// `ns{A}CString` is Gecko's "narrow" (8 bits per character) string type.
/// These don't have a fixed encoding: they can be ASCII, Latin-1, or UTF-8.
/// They're called `ByteString`s in WebIDL, and they're pretty uncommon compared
/// to `ns{A}String`.

template <>
struct Serializable<nsACString> {
  [[nodiscard]] static bool ReadFrom(Reader& aReader, nsACString& aValue) {
    aReader.ReadCString(aValue);
    return true;
  }
  static void WriteInto(Writer& aWriter, const nsACString& aValue) {
    aWriter.WriteCString(aValue);
  }
};

template <>
struct ViaFfi<nsACString, {{ context.ffi_rustbuffer_type() }}, false> {
  [[nodiscard]] static bool Lift(const {{ context.ffi_rustbuffer_type() }}& aLowered,
                                 nsACString& aLifted) {
    if (aLowered.mData) {
      aLifted.Append(AsChars(Span(aLowered.mData, aLowered.mLen)));
      {{ context.ffi_rusterror_type() }} err = {0, nullptr};
      {{ ci.ffi_rustbuffer_free().name() }}(aLowered, &err);
      if (err.mCode) {
        MOZ_ASSERT(false, "Failed to lift `nsACString` from Rust buffer");
        return false;
      }
    }
    return true;
  }

  [[nodiscard]] static {{ context.ffi_rustbuffer_type() }} Lower(const nsACString& aLifted) {
    MOZ_RELEASE_ASSERT(
        aLifted.Length() <=
        static_cast<size_t>(std::numeric_limits<int32_t>::max()));
    {{ context.ffi_rusterror_type() }} err = {0, nullptr};
    {{ context.ffi_foreignbytes_type() }} bytes = {
        static_cast<int32_t>(aLifted.Length()),
        reinterpret_cast<const uint8_t*>(aLifted.BeginReading())};
    {{ context.ffi_rustbuffer_type() }} lowered = {{ ci.ffi_rustbuffer_from_bytes().name() }}(bytes, &err);
    if (err.mCode) {
      MOZ_ASSERT(false, "Failed to lower `nsACString` into Rust string");
    }
    return lowered;
  }
};

template <>
struct Serializable<nsCString> {
  [[nodiscard]] static bool ReadFrom(Reader& aReader, nsCString& aValue) {
    aReader.ReadCString(aValue);
    return true;
  }
  static void WriteInto(Writer& aWriter, const nsCString& aValue) {
    aWriter.WriteCString(aValue);
  }
};

/// `ns{A}String` is Gecko's "wide" (16 bits per character) string type.
/// These are always UTF-16, so we need to convert them to UTF-8 before
/// passing them to Rust. WebIDL calls these `DOMString`s, and they're
/// ubiquitous.

template <>
struct Serializable<nsAString> {
  [[nodiscard]] static bool ReadFrom(Reader& aReader, nsAString& aValue) {
    aReader.ReadString(aValue);
    return true;
  }
  static void WriteInto(Writer& aWriter, const nsAString& aValue) {
    aWriter.WriteString(aValue);
  }
};

template <>
struct ViaFfi<nsAString, {{ context.ffi_rustbuffer_type() }}, false> {
  [[nodiscard]] static bool Lift(const {{ context.ffi_rustbuffer_type() }}& aLowered,
                                 nsAString& aLifted) {
    if (aLowered.mData) {
      CopyUTF8toUTF16(AsChars(Span(aLowered.mData, aLowered.mLen)), aLifted);
      {{ context.ffi_rusterror_type() }} err = {0, nullptr};
      {{ ci.ffi_rustbuffer_free().name() }}(aLowered, &err);
      if (err.mCode) {
        MOZ_ASSERT(false, "Failed to lift `nsAString` from Rust buffer");
        return false;
      }
    }
    return true;
  }

  [[nodiscard]] static {{ context.ffi_rustbuffer_type() }} Lower(const nsAString& aLifted) {
    auto maxSize = EstimateUTF8Length(aLifted.Length());
    MOZ_RELEASE_ASSERT(
        maxSize.isValid() &&
        maxSize.value() <=
            static_cast<size_t>(std::numeric_limits<int32_t>::max()));

    {{ context.ffi_rusterror_type() }} err = {0, nullptr};
    {{ context.ffi_rustbuffer_type() }} lowered = {{ ci.ffi_rustbuffer_alloc().name() }}(
      static_cast<int32_t>(maxSize.value()), &err);
    if (err.mCode) {
      MOZ_ASSERT(false, "Failed to lower `nsAString` into Rust string");
    }

    auto span = AsWritableChars(Span(lowered.mData, aLifted.Length() * 3));
    size_t bytesWritten = ConvertUtf16toUtf8(aLifted, span);
    lowered.mLen = static_cast<int32_t>(bytesWritten);

    return lowered;
  }
};

template <>
struct Serializable<nsString> {
  [[nodiscard]] static bool ReadFrom(Reader& aReader, nsString& aValue) {
    aReader.ReadString(aValue);
    return true;
  }
  static void WriteInto(Writer& aWriter, const nsString& aValue) {
    aWriter.WriteString(aValue);
  }
};

/// Nullable values are prefixed by a tag: 0 if none; 1 followed by the
/// serialized value if some. These are turned into Rust `Option<T>`s.
///
/// These are always serialized, never passed directly over the FFI.

template <typename T>
struct Serializable<dom::Nullable<T>> {
  [[nodiscard]] static bool ReadFrom(Reader& aReader,
                                     dom::Nullable<T>& aValue) {
    auto hasValue = aReader.ReadInt8();
    if (hasValue != 0 && hasValue != 1) {
      MOZ_ASSERT(false);
      return false;
    }
    if (!hasValue) {
      aValue = dom::Nullable<T>();
      return true;
    }
    T value;
    if (!Serializable<T>::ReadFrom(aReader, value)) {
      return false;
    }
    aValue = dom::Nullable<T>(std::move(value));
    return true;
  };

  static void WriteInto(Writer& aWriter, const dom::Nullable<T>& aValue) {
    if (aValue.IsNull()) {
      aWriter.WriteInt8(0);
    } else {
      aWriter.WriteInt8(1);
      Serializable<T>::WriteInto(aWriter, aValue.Value());
    }
  }
};

/// Optionals are serialized the same way as nullables. The distinction
/// doesn't matter in UniFFI, because Rust doesn't have optional
/// arguments or struct fields, but does matter in WebIDL: nullable means
/// "was passed, but can be `null`," but optional means "may or may not
/// be passed."
///
/// These are always serialized, never passed directly over the FFI.

template <typename T>
struct Serializable<dom::Optional<T>> {
  [[nodiscard]] static bool ReadFrom(Reader& aReader,
                                     dom::Optional<T>& aValue) {
    auto hasValue = aReader.ReadInt8();
    if (hasValue != 0 && hasValue != 1) {
      MOZ_ASSERT(false);
      return false;
    }
    if (!hasValue) {
      aValue = dom::Optional<T>();
      return true;
    }
    T value;
    if (!Serializable<T>::ReadFrom(aReader, value)) {
      return false;
    }
    aValue = dom::Optional<T>(std::move(value));
    return true;
  };

  static void WriteInto(Writer& aWriter, const dom::Optional<T>& aValue) {
    if (!aValue.WasPassed()) {
      aWriter.WriteInt8(0);
    } else {
      aWriter.WriteInt8(1);
      Serializable<T>::WriteInto(aWriter, aValue.Value());
    }
  }
};

/// Sequences are length-prefixed, followed by the serialization of each
/// element. They're always serialized, and never passed directly over the
/// FFI.
///
/// WebIDL has two different representations for sequences, though they both
/// use `nsTArray<T>` under the hood. `dom::Sequence<T>` is for sequence
/// arguments; `nsTArray<T>` is for sequence return values and dictionary
/// members.

template <typename T>
struct Serializable<dom::Sequence<T>> {
  // We leave `ReadFrom` unimplemented because sequences should only be
  // lowered from the C++ WebIDL binding to the FFI. If the FFI function
  // returns a sequence, it'll be lifted into an `nsTArray<T>`, not a
  // `dom::Sequence<T>`. See the note about sequences above.
  [[nodiscard]] static bool ReadFrom(Reader& aReader,
                                     dom::Sequence<T>& aValue) = delete;

  static void WriteInto(Writer& aWriter, const dom::Sequence<T>& aValue) {
    aWriter.WriteLength(aValue.Length());
    for (const T& element : aValue) {
      Serializable<T>::WriteInto(aWriter, element);
    }
  }
};

template <typename T>
struct Serializable<nsTArray<T>> {
  [[nodiscard]] static bool ReadFrom(Reader& aReader, nsTArray<T>& aValue) {
    auto length = aReader.ReadLength();
    aValue.SetCapacity(length);
    for (size_t i = 0; i < length; ++i) {
      if (!Serializable<T>::ReadFrom(aReader, *aValue.AppendElement())) {
        return false;
      }
    }
    return true;
  };

  static void WriteInto(Writer& aWriter, const nsTArray<T>& aValue) {
    aWriter.WriteLength(aValue.Length());
    for (const T& element : aValue) {
      Serializable<T>::WriteInto(aWriter, element);
    }
  }
};

/// Records are length-prefixed, followed by the serialization of each
/// key and value. They're always serialized, and never passed directly over the
/// FFI.

template <typename K, typename V>
struct Serializable<Record<K, V>> {
  [[nodiscard]] static bool ReadFrom(Reader& aReader, Record<K, V>& aValue) {
    auto length = aReader.ReadLength();
    aValue.Entries().SetCapacity(length);
    for (size_t i = 0; i < length; ++i) {
      typename Record<K, V>::EntryType* entry =
          aValue.Entries().AppendElement();
      if (!Serializable<K>::ReadFrom(aReader, entry->mKey)) {
        return false;
      }
      if (!Serializable<V>::ReadFrom(aReader, entry->mValue)) {
        return false;
      }
    }
    return true;
  };

  static void WriteInto(Writer& aWriter, const Record<K, V>& aValue) {
    aWriter.WriteLength(aValue.Entries().Length());
    for (const typename Record<K, V>::EntryType& entry : aValue.Entries()) {
      Serializable<K>::WriteInto(aWriter, entry.mKey);
      Serializable<V>::WriteInto(aWriter, entry.mValue);
    }
  }
};

/// Partial specialization for all types that can be serialized into a byte
/// buffer. This is analogous to the `ViaFfiUsingByteBuffer` trait in Rust.

template <typename T>
struct ViaFfi<T, {{ context.ffi_rustbuffer_type() }}> {
  [[nodiscard]] static bool Lift(const {{ context.ffi_rustbuffer_type() }}& aLowered, T& aLifted) {
    auto reader = Reader(aLowered);
    if (!Serializable<T>::ReadFrom(reader, aLifted)) {
      return false;
    }
    if (reader.HasRemaining()) {
      MOZ_ASSERT(false);
      return false;
    }
    {{ context.ffi_rusterror_type() }} err = {0, nullptr};
    {{ ci.ffi_rustbuffer_free().name() }}(aLowered, &err);
    if (err.mCode) {
      MOZ_ASSERT(false, "Failed to free Rust buffer after lifting contents");
      return false;
    }
    return true;
  }

  [[nodiscard]] static {{ context.ffi_rustbuffer_type() }} Lower(const T& aLifted) {
    auto writer = Writer();
    Serializable<T>::WriteInto(writer, aLifted);
    return writer.Buffer();
  }
};

/// Nullable strings are a special case. In Gecko C++, there's no type-level
/// way to distinguish between nullable and non-null strings: the WebIDL
/// bindings layer passes `nsAString` for both `DOMString` and `DOMString?`.
/// But the Rust side of the FFI expects nullable strings to be serialized as
/// `Nullable<nsA{C}String>`, not `nsA{C}String`.
///
/// These specializations serialize nullable strings as if they were
/// `Nullable<nsA{C}String>`.

template <>
struct ViaFfi<nsACString, {{ context.ffi_rustbuffer_type() }}, true> {
  [[nodiscard]] static bool Lift(const {{ context.ffi_rustbuffer_type() }}& aLowered,
                                 nsACString& aLifted) {
    auto value = dom::Nullable<nsCString>();
    if (!ViaFfi<dom::Nullable<nsCString>, {{ context.ffi_rustbuffer_type() }}>::Lift(aLowered, value)) {
      return false;
    }
    if (value.IsNull()) {
      // `SetIsVoid` marks the string as "voided". The JS engine will reflect
      // voided strings as `null`, not `""`.
      aLifted.SetIsVoid(true);
    } else {
      aLifted = value.Value();
    }
    return true;
  }

  [[nodiscard]] static {{ context.ffi_rustbuffer_type() }} Lower(const nsACString& aLifted) {
    auto value = dom::Nullable<nsCString>();
    if (!aLifted.IsVoid()) {
      value.SetValue() = aLifted;
    }
    return ViaFfi<dom::Nullable<nsCString>, {{ context.ffi_rustbuffer_type() }}>::Lower(value);
  }
};

template <>
struct ViaFfi<nsAString, {{ context.ffi_rustbuffer_type() }}, true> {
  [[nodiscard]] static bool Lift(const {{ context.ffi_rustbuffer_type() }}& aLowered,
                                 nsAString& aLifted) {
    auto value = dom::Nullable<nsString>();
    if (!ViaFfi<dom::Nullable<nsString>, {{ context.ffi_rustbuffer_type() }}>::Lift(aLowered, value)) {
      return false;
    }
    if (value.IsNull()) {
      aLifted.SetIsVoid(true);
    } else {
      aLifted = value.Value();
    }
    return true;
  }

  [[nodiscard]] static {{ context.ffi_rustbuffer_type() }} Lower(const nsAString& aLifted) {
    auto value = dom::Nullable<nsString>();
    if (!aLifted.IsVoid()) {
      value.SetValue() = aLifted;
    }
    return ViaFfi<dom::Nullable<nsString>, {{ context.ffi_rustbuffer_type() }}>::Lower(value);
  }
};

/// Partial specialization for all non-null types on the C++ side that should be
/// serialized as if they were nullable. This is analogous to a blanket
/// implementation of `ViaFfiUsingByteBuffer` for `Option<T>` in Rust.

template <typename T>
struct ViaFfi<T, {{ context.ffi_rustbuffer_type() }}, true> {
  [[nodiscard]] static bool Lift(const {{ context.ffi_rustbuffer_type() }}& aLowered,
                                 T& aLifted) {
    auto reader = Reader(aLowered);
    auto hasValue = reader.ReadInt8();
    if (hasValue != 0 && hasValue != 1) {
      MOZ_ASSERT(false);
      return false;
    }
    if (!hasValue) {
      return true;
    }
    if (!Serializable<T>::ReadFrom(reader, aLifted)) {
      return false;
    }
    if (reader.HasRemaining()) {
      MOZ_ASSERT(false);
      return false;
    }
    {{ context.ffi_rusterror_type() }} err = {0, nullptr};
    {{ ci.ffi_rustbuffer_free().name() }}(aLowered, &err);
    if (err.mCode) {
      MOZ_ASSERT(false, "Failed to free Rust buffer after lifting contents");
      return false;
    }
    return true;
  }

  [[nodiscard]] static {{ context.ffi_rustbuffer_type() }} Lower(const T& aLifted) {
    auto writer = Writer();
    writer.WriteInt8(1);
    Serializable<T>::WriteInto(writer, aLifted);
    return writer.Buffer();
  }
};

}  // namespace {{ context.detail_name() }}
