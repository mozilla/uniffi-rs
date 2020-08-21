// TODO: Add back errors. We may need runtime errors so that we can throw
// type errors if, say, we omit a dictionary field. (In Gecko WebIDL,
// all dictionary fields are optional unless required; in UniFFI IDL,
// they're required by default). Maybe this can be `Result<T, ErrorResult>`,
// so we can propagate these with more details (like allocation failures,
// type errors, serialization errors, etc.)

/// A helper class to read values out of a Rust byte buffer.
class MOZ_STACK_CLASS Reader final {
 public:
  explicit Reader(const RustBuffer& aBuffer) : mBuffer(aBuffer), mOffset(0) {}

  /// Indicates if the offset has reached the end of the buffer.
  bool HasRemaining() {
    return static_cast<int64_t>(mOffset.value()) < mBuffer.mLen;
  }

  /// Helpers to read fixed-width primitive types at the current offset.
  /// Fixed-width integers are read in big endian order.

  uint8_t ReadUInt8() {
    return ReadAt<uint8_t>(
        [this](size_t aOffset) { return mBuffer.mData[aOffset]; });
  }

  int8_t ReadInt8() { return BitwiseCast<int8_t>(ReadUInt8()); }

  uint16_t ReadUInt16() {
    return ReadAt<uint16_t>([this](size_t aOffset) {
      uint16_t value;
      memcpy(&value, &mBuffer.mData[aOffset], sizeof(uint16_t));
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

  float ReadFloat() { return BitwiseCast<float>(ReadUInt32()); }

  double ReadDouble() { return BitwiseCast<double>(ReadUInt64()); }

  /// Reads a length-prefixed UTF-8 encoded string at the current offset. The
  /// closure takes a `Span` pointing to the raw bytes, which it can use to
  /// copy the bytes into an `nsCString` or `nsString`.
  ///
  /// Safety: The closure must copy the span's contents into a new owned string.
  /// It must not hold on to the span, as its contents will be invalidated when
  /// the backing Rust byte buffer is freed. It must not call any other methods
  /// on the reader.
  template <typename T>
  T ReadRawString(const std::function<T(Span<const char>)>& aClosure) {
    uint32_t length = ReadInt32();
    CheckedInt<size_t> newOffset = mOffset;
    newOffset += length;
    AssertInBounds(newOffset);
    const char* begin =
        reinterpret_cast<const char*>(&mBuffer.mData[mOffset.value()]);
    T result = aClosure(Span(begin, length));
    mOffset = newOffset;
    return result;
  }

 private:
  void AssertInBounds(const CheckedInt<size_t>& aNewOffset) const {
    MOZ_RELEASE_ASSERT(aNewOffset.isValid() &&
                       static_cast<int64_t>(aNewOffset.value()) <=
                           mBuffer.mLen);
  }

  template <typename T>
  T ReadAt(const std::function<T(size_t)>& aClosure) {
    CheckedInt<size_t> newOffset = mOffset;
    newOffset += sizeof(T);
    AssertInBounds(newOffset);
    T result = aClosure(mOffset.value());
    mOffset = newOffset;
    return result;
  }

  const RustBuffer& mBuffer;
  CheckedInt<size_t> mOffset;
};

class MOZ_STACK_CLASS Writer final {
 public:
  explicit Writer(size_t aCapacity) : mBuffer(aCapacity) {}

  void WriteUInt8(const uint8_t& aValue) {
    WriteAt<uint8_t>(aValue, [this](size_t aOffset, const uint8_t& aValue) {
      mBuffer[aOffset] = aValue;
    });
  }

  void WriteInt8(const int8_t& aValue) {
    WriteUInt8(BitwiseCast<uint8_t>(aValue));
  }

  // This code uses `memcpy` and other eye-twitchy patterns because it
  // originally wrote values directly into a `RustBuffer`, instead of
  // an intermediate `nsTArray`. Once #251 is fixed, we can return to
  // doing that, and remove `ToRustBuffer`.

  void WriteUInt16(const uint16_t& aValue) {
    WriteAt<uint16_t>(aValue, [this](size_t aOffset, const uint16_t& aValue) {
      uint16_t value = PR_htons(aValue);
      memcpy(&mBuffer.Elements()[aOffset], &value, sizeof(uint16_t));
    });
  }

  void WriteInt16(const int16_t& aValue) {
    WriteUInt16(BitwiseCast<uint16_t>(aValue));
  }

  void WriteUInt32(const uint32_t& aValue) {
    WriteAt<uint32_t>(aValue, [this](size_t aOffset, const uint32_t& aValue) {
      uint32_t value = PR_htonl(aValue);
      memcpy(&mBuffer.Elements()[aOffset], &value, sizeof(uint32_t));
    });
  }

  void WriteInt32(const int32_t& aValue) {
    WriteUInt32(BitwiseCast<uint32_t>(aValue));
  }

  void WriteUInt64(const uint64_t& aValue) {
    WriteAt<uint64_t>(aValue, [this](size_t aOffset, const uint64_t& aValue) {
      uint64_t value = PR_htonll(aValue);
      memcpy(&mBuffer.Elements()[aOffset], &value, sizeof(uint64_t));
    });
  }

  void WriteInt64(const int64_t& aValue) {
    WriteUInt64(BitwiseCast<uint64_t>(aValue));
  }

  void WriteFloat(const float& aValue) {
    WriteUInt32(BitwiseCast<uint32_t>(aValue));
  }

  void WriteDouble(const double& aValue) {
    WriteUInt64(BitwiseCast<uint64_t>(aValue));
  }

  /// Writes a length-prefixed UTF-8 encoded string at the current offset. The
  /// closure takes a `Span` pointing to the byte buffer, which it should fill
  /// with bytes and return the actual number of bytes written.
  ///
  /// This function is (more than a little) convoluted. It's written this way
  /// because we want to support UTF-8 and UTF-16 strings. The "size hint" is
  /// the maximum number of bytes that the closure can write. For UTF-8 strings,
  /// this is just the length. For UTF-16 strings, which must be converted to
  /// UTF-8, this can be up to three times the length. Once the closure tells us
  /// how many bytes it's actually written, we can write the length prefix, and
  /// advance the current offset.
  ///
  /// Safety: The closure must copy the string's contents into the span, and
  /// return the exact number of bytes it copied. Returning the wrong count can
  /// either truncate the string, or leave uninitialized memory in the buffer.
  /// It must not call any other methods on the writer.
  void WriteRawString(size_t aSizeHint,
                      const std::function<size_t(Span<char>)>& aClosure) {
    // First, make sure the buffer is big enough to hold the length prefix.
    // We'll start writing our string directly after the prefix.
    CheckedInt<size_t> newOffset = mOffset;
    newOffset += sizeof(uint32_t);
    AssertInBounds(newOffset);
    char* begin =
        reinterpret_cast<char*>(&mBuffer.Elements()[newOffset.value()]);

    // Next, ensure the buffer has space for enough bytes up to the size hint.
    // We may write fewer bytes than hinted, but we need to handle the worst
    // case if needed.
    newOffset += aSizeHint;
    AssertInBounds(newOffset);

    // Call the closure to write the bytes directly into the buffer.
    size_t bytesWritten = aClosure(Span(begin, aSizeHint));

    // Great, now we know the real length! Write it at the beginning.
    uint32_t lengthPrefix = PR_htonl(bytesWritten);
    memcpy(&mBuffer.Elements()[mOffset.value()], &lengthPrefix,
           sizeof(uint32_t));

    // And figure out our actual offset.
    newOffset -= aSizeHint;
    newOffset += bytesWritten;
    AssertInBounds(newOffset);
    mOffset = newOffset;
  }

  RustBuffer ToRustBuffer() {
    auto size = static_cast<uint32_t>(mOffset.value());
    auto buffer = {{ ci.ffi_bytebuffer_alloc().name() }}(size);
    memcpy(buffer.mData, mBuffer.Elements(), size);
    return buffer;
  }

 private:
  void AssertInBounds(const CheckedInt<size_t>& aNewOffset) const {
    MOZ_RELEASE_ASSERT(aNewOffset.isValid() &&
                       aNewOffset.value() <= mBuffer.Capacity());
  }

  template <typename T>
  void WriteAt(const T& aValue,
               const std::function<void(size_t, const T&)>& aClosure) {
    CheckedInt<size_t> newOffset = mOffset;
    newOffset += sizeof(T);
    AssertInBounds(newOffset);
    aClosure(mOffset.value(), aValue);
    mOffset = newOffset;
  }

  nsTArray<uint8_t> mBuffer;
  CheckedInt<size_t> mOffset;
};

/// A "trait" with specializations for types that can be read and written into
/// a byte buffer.
///
/// The scare quotes are because C++ doesn't have traits, but we can fake them
/// using partial template specialization. Instead of using a base class with
/// pure virtual functions that are overridden for each type, we define a
/// primary template struct with our interface here, and specialize it for each
/// type that we support.
///
/// When we have some type `T` that we want to extract from a buffer, we write
/// `T value = Serializable<T>::ReadFrom(reader)`.
///
/// Deleting the functions in the primary template gives us compile-time type
/// checking. If `Serializable` isn't specialized for `T`, the compiler picks
/// the primary template, and complains we're trying to use a deleted function.
/// If we just left the functions unimplemented, we'd get a confusing linker
/// error instead.
template <typename T>
struct Serializable {
  /// Returns the serialized size of the value, in bytes. This is used to
  /// calculate the allocation size for the Rust byte buffer.
  static size_t Size(const T& aValue) = delete;

  /// Reads a value of type `T` from a byte buffer.
  static T ReadFrom(Reader& aReader) = delete;

  /// Writes a value of type `T` into a byte buffer.
  static void WriteInto(const T& aValue, Writer& aWriter) = delete;
};

// A "trait" with specializations for types that can be transferred back and
// forth over the FFI. This is analogous to the Rust trait of the same name.
// As above, this gives us compile-time type checking for type pairs. If
// `ViaFfi<T, U>::Lift(U)` compiles, we know that a value of type `U` from the
// FFI can be lifted into a value of type `T`.
template <typename T, typename FfiType>
struct ViaFfi {
  static T Lift(const FfiType& aValue) = delete;
  static FfiType Lower(const T& aValue) = delete;
};

// This macro generates boilerplate specializations for primitive numeric types
// that are passed directly over the FFI without conversion.
#define UNIFFI_SPECIALIZE_SERIALIZABLE_PRIMITIVE(Type, readFunc, writeFunc) \
  template <>                                                               \
  struct Serializable<Type> {                                               \
    static size_t Size(const Type& aValue) { return sizeof(Type); }         \
    static Type ReadFrom(Reader& aReader) { return aReader.readFunc(); }    \
    static void WriteInto(const Type& aValue, Writer& aWriter) {            \
      aWriter.writeFunc(aValue);                                            \
    }                                                                       \
  };                                                                        \
  template <>                                                               \
  struct ViaFfi<Type, Type> {                                               \
    static Type Lift(const Type& aValue) { return aValue; }                 \
    static Type Lower(const Type& aValue) { return aValue; }                \
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

/// Booleans are passed as unsigned integers over the FFI, because JNA doesn't
/// handle `bool`s well.

template <>
struct Serializable<bool> {
  static size_t Size(const bool& aValue) { return 1; }
  static bool ReadFrom(Reader& aReader) { return aReader.ReadUInt8() != 0; }
  static void WriteInto(const bool& aValue, Writer& aWriter) {
    aWriter.WriteUInt8(aValue ? 1 : 0);
  }
};

template <>
struct ViaFfi<bool, uint8_t> {
  static bool Lift(const uint8_t& aValue) { return aValue != 0; }
  static uint8_t Lower(const bool& aValue) { return aValue ? 1 : 0; }
};

/// Strings are length-prefixed and UTF-8 encoded when serialized
/// into byte buffers, and are passed as null-terminated, UTF-8
/// encoded `char*` pointers over the FFI.
///
/// Gecko has two string types: `nsCString` for "narrow" strings, and `nsString`
/// for "wide" strings. `nsCString`s don't have a fixed encoding: these can be
/// ASCII, Latin-1, or UTF-8. `nsString`s are always UTF-16. JS prefers
/// `nsString` (UTF-16; also called `DOMString` in WebIDL); `nsCString`s
/// (`ByteString` in WebIDL) are pretty uncommon.
///
/// `nsCString`s can be passed to Rust directly, and copied byte-for-byte into
/// buffers. The UniFFI scaffolding code will ensure they're valid UTF-8. But
/// `nsString`s must be converted to UTF-8 first.

template <>
struct Serializable<nsCString> {
  static size_t Size(const nsString& aValue) {
    CheckedInt<size_t> size(aValue.Length());
    size += sizeof(uint32_t);  // For the length prefix.
    MOZ_RELEASE_ASSERT(size.isValid());
    return size.value();
  }

  static nsCString ReadFrom(Reader& aReader) {
    return aReader.ReadRawString<nsCString>(
        [](Span<const char> aRawString) { return nsCString(aRawString); });
  }

  static void WriteInto(const nsCString& aValue, Writer& aWriter) {
    aWriter.WriteRawString(aValue.Length(), [aValue](Span<char> aRawString) {
      memcpy(aRawString.Elements(), aValue.BeginReading(), aRawString.Length());
      return aRawString.Length();
    });
  }
};

template <>
struct ViaFfi<nsCString, char*> {
  static nsCString Lift(const char*& aValue) {
    return nsCString(MakeStringSpan(aValue));
  }

  static char* Lower(const nsCString& aValue) {
    RustError error{0, nullptr};
    char* result = {{ ci.ffi_string_alloc_from().name() }}(aValue.BeginReading(), &error);
    MOZ_RELEASE_ASSERT(!error.mCode,
                       "Failed to copy narrow string to Rust string");
    return result;
  }
};

template <>
struct Serializable<nsString> {
  static size_t Size(const nsString& aValue) {
    auto size = EstimateUTF8Length(aValue);
    size += sizeof(uint32_t);  // For the length prefix.
    MOZ_RELEASE_ASSERT(size.isValid());
    return size.value();
  }

  static nsString ReadFrom(Reader& aReader) {
    return aReader.ReadRawString<nsString>([](Span<const char> aRawString) {
      nsAutoString result;
      AppendUTF8toUTF16(aRawString, result);
      return result;
    });
  }

  static void WriteInto(const nsString& aValue, Writer& aWriter) {
    auto length = EstimateUTF8Length(aValue);
    MOZ_RELEASE_ASSERT(length.isValid());
    aWriter.WriteRawString(length.value(), [aValue](Span<char> aRawString) {
      return ConvertUtf16toUtf8(aValue, aRawString);
    });
  }

  /// Estimates the UTF-8 encoded length of a UTF-16 string. This is a
  /// worst-case estimate.
  static CheckedInt<size_t> EstimateUTF8Length(const nsAString& aUTF16) {
    CheckedInt<size_t> length(aUTF16.Length());
    // `ConvertUtf16toUtf8` expects the destination to have at least three times
    // as much space as the source string.
    length *= 3;
    return length;
  }
};

template <>
struct ViaFfi<nsString, char*> {
  static nsString Lift(const char*& aValue) {
    nsAutoString utf16;
    CopyUTF8toUTF16(MakeStringSpan(aValue), utf16);
    return std::move(utf16);
  }

  static char* Lower(const nsString& aValue) {
    // Encode the string to UTF-8, then make a Rust string from the contents.
    // This copies the string twice, but is safe.
    nsAutoCString utf8;
    CopyUTF16toUTF8(aValue, utf8);
    RustError error{0, nullptr};
    char* result = {{ ci.ffi_string_alloc_from().name() }}(utf8.BeginReading(), &error);
    MOZ_RELEASE_ASSERT(!error.mCode,
                       "Failed to copy wide string to Rust string");
    return result;
  }
};

/// Nullable values are prefixed by a tag: 0 if none; 1 followed by the
/// serialized value if some. These are turned into Rust `Option<T>`s.
///
/// Fun fact: WebIDL also has a `dom::Optional<T>` type. They both use
/// `mozilla::Maybe<T>` under the hood, but their semantics are different.
/// `Nullable<T>` means JS must pass some value for the argument or dictionary
/// field: either `T` or `null`. `Optional<T>` means JS can omit the argument
/// or member entirely.
///
/// These are always serialized, never passed directly over the FFI.

template <typename T>
struct Serializable<dom::Nullable<T>> {
  static size_t Size(const dom::Nullable<T>& aValue) {
    if (!aValue.WasPassed()) {
      return 1;
    }
    CheckedInt<size_t> size(1);
    size += Serializable<T>::Size(aValue.Value());
    MOZ_RELEASE_ASSERT(size.isValid());
    return size.value();
  }

  static dom::Nullable<T> ReadFrom(Reader& aReader) {
    uint8_t hasValue = aReader.ReadUInt8();
    MOZ_RELEASE_ASSERT(hasValue == 0 || hasValue == 1,
                       "Unexpected nullable type tag");
    if (!hasValue) {
      return dom::Nullable<T>();
    }
    return dom::Nullable<T>(std::move(Serializable<T>::ReadFrom(aReader)));
  };

  static void WriteInto(const dom::Nullable<T>& aValue, Writer& aWriter) {
    if (!aValue.WasPassed()) {
      aWriter.WriteUInt8(0);
    } else {
      aWriter.WriteUInt8(1);
      Serializable<T>::WriteInto(aValue.Value(), aWriter);
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

/// Shared traits for serializing sequences.
template <typename T>
struct SequenceTraits {
  static size_t Size(const T& aValue) {
    if (aValue.IsEmpty()) {
      return sizeof(uint32_t);
    }
    // Arrays are limited to `uint32_t` bytes.
    CheckedInt<uint32_t> size(
        Serializable<typename T::elem_type>::Size(aValue.ElementAt(0)));
    size *= aValue.Length();
    size += sizeof(uint32_t);  // For the length prefix.
    MOZ_RELEASE_ASSERT(size.isValid());
    return size.value();
  }

  static void WriteInto(const T& aValue, Writer& aWriter) {
    aWriter.WriteUInt32(aValue.Length());
    for (const typename T::elem_type& element : aValue) {
      Serializable<typename T::elem_type>::WriteInto(element, aWriter);
    }
  }
};

template <typename T>
struct Serializable<dom::Sequence<T>> {
  static size_t Size(const dom::Sequence<T>& aValue) {
    return SequenceTraits<dom::Sequence<T>>::Size(aValue);
  }

  // We leave `ReadFrom` unimplemented because sequences should only be
  // lowered from the C++ WebIDL binding to the FFI. If the FFI function
  // returns a sequence, it'll be lifted into an `nsTArray<T>`, not a
  // `dom::Sequence<T>`. See the note about sequences above.
  static dom::Sequence<T> ReadFrom(Reader& aReader) = delete;

  static void WriteInto(const dom::Sequence<T>& aValue, Writer& aWriter) {
    SequenceTraits<dom::Sequence<T>>::WriteInto(aValue, aWriter);
  }
};

template <typename T>
struct Serializable<nsTArray<T>> {
  static size_t Size(const nsTArray<T>& aValue) {
    return SequenceTraits<nsTArray<T>>::Size(aValue);
  }

  static nsTArray<T> ReadFrom(Reader& aReader) {
    uint32_t length = aReader.ReadUInt32();
    auto result = nsTArray<T>(length);
    for (uint32_t i = 0; i < length; ++i) {
      result.AppendElement(std::move(Serializable<T>::ReadFrom(aReader)));
    }
    return std::move(result);
  };

  static void WriteInto(const nsTArray<T>& aValue, Writer& aWriter) {
    SequenceTraits<nsTArray<T>>::WriteInto(aValue, aWriter);
  }
};

/// Partial specialization for all types that can be serialized into a byte
/// buffer. This is analogous to the `ViaFfiUsingByteBuffer` trait in Rust.

template <typename T>
struct ViaFfi<T, RustBuffer> {
  static T Lift(const RustBuffer& aBuffer) {
    auto reader = Reader(aBuffer);
    T value = Serializable<T>::ReadFrom(reader);
    MOZ_RELEASE_ASSERT(!reader.HasRemaining(), "Junk left in incoming buffer");
    {{ ci.ffi_bytebuffer_free().name() }}(aBuffer);
    return value;
  }

  static RustBuffer Lower(const T& aValue) {
    size_t size = Serializable<T>::Size(aValue);
    auto writer = Writer(size);
    Serializable<T>::WriteInto(aValue, writer);
    return writer.ToRustBuffer();
  }
};
