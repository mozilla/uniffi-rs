class MOZ_STACK_CLASS Reader final {
 public:
  explicit Reader(const RustBuffer& aBuffer) : mBuffer(aBuffer), mOffset(0) {}

  bool HasRemaining() {
    return static_cast<int64_t>(mOffset.value()) < mBuffer.mLen;
  }

  Result<uint8_t, nsresult> ReadUInt8() {
    return ReadAt<uint8_t>(
        [this](size_t aOffset) { return mBuffer.mData[aOffset]; });
  }

  Result<int8_t, nsresult> ReadInt8() {
    return ReadUInt8().map(
        [](uint8_t aValue) { return static_cast<int8_t>(aValue); });
  }

  Result<uint16_t, nsresult> ReadUInt16() {
    return ReadAt<uint16_t>([this](size_t aOffset) {
      uint16_t value = mBuffer.mData[aOffset + 1];
      value |= static_cast<uint16_t>(mBuffer.mData[aOffset]) << 8;
      return value;
    });
  }

  Result<int16_t, nsresult> ReadInt16() {
    return ReadUInt16().map(
        [](uint16_t aValue) { return static_cast<int16_t>(aValue); });
  }

  Result<uint32_t, nsresult> ReadUInt32() {
    return ReadAt<uint32_t>([this](size_t aOffset) {
      uint32_t value = mBuffer.mData[aOffset + 3];
      value |= static_cast<uint32_t>(mBuffer.mData[aOffset + 2]) << 8;
      value |= static_cast<uint32_t>(mBuffer.mData[aOffset + 1]) << 16;
      value |= static_cast<uint32_t>(mBuffer.mData[aOffset]) << 24;
      return value;
    });
  }

  Result<int32_t, nsresult> ReadInt32() {
    return ReadUInt32().map(
        [](uint32_t aValue) { return static_cast<int32_t>(aValue); });
  }

  Result<uint64_t, nsresult> ReadUInt64() {
    return ReadAt<uint64_t>([this](size_t aOffset) {
      uint64_t value = mBuffer.mData[aOffset + 7];
      value |= static_cast<uint64_t>(mBuffer.mData[aOffset + 6]) << 8;
      value |= static_cast<uint64_t>(mBuffer.mData[aOffset + 5]) << 16;
      value |= static_cast<uint64_t>(mBuffer.mData[aOffset + 4]) << 24;
      value |= static_cast<uint64_t>(mBuffer.mData[aOffset + 3]) << 32;
      value |= static_cast<uint64_t>(mBuffer.mData[aOffset + 2]) << 40;
      value |= static_cast<uint64_t>(mBuffer.mData[aOffset + 1]) << 48;
      value |= static_cast<uint64_t>(mBuffer.mData[aOffset]) << 56;
      return value;
    });
  }

  Result<int64_t, nsresult> ReadInt64() {
    return ReadUInt64().map(
        [](uint64_t aValue) { return static_cast<int64_t>(aValue); });
  }

  Result<float, nsresult> ReadFloat() {
    return ReadUInt32().map(
        [](uint32_t aValue) { return static_cast<float>(aValue); });
  }

  Result<double, nsresult> ReadDouble() {
    return ReadUInt64().map(
        [](uint64_t aValue) { return static_cast<double>(aValue); });
  }

 private:
  template <typename T>
  Result<T, nsresult> ReadAt(const std::function<T(size_t)>& aClosure) {
    CheckedInt<size_t> newOffset = mOffset;
    newOffset += sizeof(T);
    if (!newOffset.isValid() || int64_t(newOffset.value()) >= mBuffer.mLen) {
      return Err(NS_ERROR_ILLEGAL_VALUE);
    }
    T result = aClosure(mOffset.value());
    mOffset = newOffset;
    return result;
  }

  const RustBuffer& mBuffer;
  CheckedInt<size_t> mOffset;
};

class MOZ_STACK_CLASS Writer final {
 public:
  explicit Writer(RustBuffer& aBuffer);

  Result<Ok, nsresult> WriteUInt8(const uint8_t& aValue) {
    return WriteAt<uint8_t>(aValue,
                            [this](size_t aOffset, const uint8_t& aValue) {
                              mBuffer.mData[aOffset] = aValue;
                            });
  }

  Result<Ok, nsresult> WriteInt8(const int8_t& aValue) {
    auto value = static_cast<uint8_t>(aValue);
    return WriteUInt8(value);
  }

  Result<Ok, nsresult> WriteUInt16(const uint16_t& aValue) {
    return WriteAt<uint16_t>(aValue,
                             [this](size_t aOffset, const uint16_t& aValue) {
                               mBuffer.mData[aOffset] = (aValue >> 8) & 0xff;
                               mBuffer.mData[aOffset + 1] = aValue & 0xff;
                             });
  }

  Result<Ok, nsresult> WriteInt16(const int16_t& aValue) {
    auto value = static_cast<uint16_t>(aValue);
    return WriteUInt16(value);
  }

  Result<Ok, nsresult> WriteUInt32(const uint32_t& aValue) {
    return WriteAt<uint32_t>(
        aValue, [this](size_t aOffset, const uint32_t& aValue) {
          mBuffer.mData[aOffset] = (aValue >> 24) & 0xff;
          mBuffer.mData[aOffset + 1] = (aValue >> 16) & 0xff;
          mBuffer.mData[aOffset + 2] = (aValue >> 8) & 0xff;
          mBuffer.mData[aOffset + 3] = aValue & 0xff;
        });
  }

  Result<Ok, nsresult> WriteInt32(const int32_t& aValue) {
    auto value = static_cast<uint32_t>(aValue);
    return WriteUInt32(value);
  }

  Result<Ok, nsresult> WriteUInt64(const uint64_t& aValue) {
    return WriteAt<uint64_t>(
        aValue, [this](size_t aOffset, const uint64_t& aValue) {
          mBuffer.mData[aOffset] = (aValue >> 56) & 0xff;
          mBuffer.mData[aOffset + 1] = (aValue >> 48) & 0xff;
          mBuffer.mData[aOffset + 2] = (aValue >> 40) & 0xff;
          mBuffer.mData[aOffset + 3] = (aValue >> 32) & 0xff;
          mBuffer.mData[aOffset + 4] = (aValue >> 24) & 0xff;
          mBuffer.mData[aOffset + 5] = (aValue >> 16) & 0xff;
          mBuffer.mData[aOffset + 6] = (aValue >> 8) & 0xff;
          mBuffer.mData[aOffset + 7] = aValue & 0xff;
        });
  }

  Result<Ok, nsresult> WriteInt64(const int64_t& aValue) {
    auto value = static_cast<uint64_t>(aValue);
    return WriteUInt64(value);
  }

  Result<Ok, nsresult> WriteFloat(const float& aValue) {
    auto value = static_cast<uint32_t>(aValue);
    return WriteUInt32(value);
  }

  Result<Ok, nsresult> WriteDouble(const double& aValue) {
    auto value = static_cast<uint64_t>(aValue);
    return WriteUInt64(value);
  }

 private:
  template <typename T>
  Result<Ok, nsresult> WriteAt(
      const T& aValue, const std::function<void(size_t, const T&)>& aClosure) {
    CheckedInt<size_t> newOffset = mOffset;
    newOffset += sizeof(T);
    if (!newOffset.isValid() || int64_t(newOffset.value()) >= mBuffer.mLen) {
      return Err(NS_ERROR_ILLEGAL_VALUE);
    }
    aClosure(mOffset.value(), aValue);
    mOffset = newOffset;
    return Ok();
  }

  RustBuffer& mBuffer;
  CheckedInt<size_t> mOffset;
};

// A "trait" with specializations for types that can be read and written into
// a byte buffer.
template <typename T>
struct Serializable {
  static Result<T, nsresult> ReadFrom(Reader& aReader) = delete;
  static Result<Ok, nsresult> WriteInto(const T& aValue,
                                        Writer& aWriter) = delete;
  static size_t Size(const T& aValue) = delete;
};

// A "trait" with specializations for types that can be transferred back and
// forth over the FFI. This is analogous to the Rust trait of the same name.
template <typename T, typename FfiType>
struct ViaFfi {
  static Result<T, nsresult> Lift(const FfiType& aValue) = delete;
  static FfiType Lower(const T& aValue) = delete;
};

template <>
struct Serializable<uint8_t> {
  static Result<uint8_t, nsresult> ReadFrom(Reader& aReader) {
    return aReader.ReadUInt8();
  };

  static Result<Ok, nsresult> WriteInto(const uint8_t& aValue,
                                        Writer& aWriter) {
    return aWriter.WriteUInt8(aValue);
  }

  static size_t Size(const uint8_t& aValue) { return 1; }
};

template <>
struct ViaFfi<uint8_t, uint8_t> {
  static Result<uint8_t, nsresult> Lift(const uint8_t& aValue) {
    return aValue;
  };

  static uint8_t Lower(const uint8_t& aValue) { return aValue; }
};

template <typename T>
struct ViaFfi<T, RustBuffer> {
  // TODO: `const` and references might not be a good choice here...
  static Result<T, nsresult> Lift(const RustBuffer& aBuffer) {
    auto reader = Reader(aBuffer);
    T value;
    MOZ_TRY_VAR(value, Serializable<T>::ReadFrom(reader));
    if (reader.HasRemaining()) {
      return Err(NS_ERROR_ILLEGAL_VALUE);
    }
    {{ ci.ffi_bytebuffer_free().name() }}(aBuffer);
    return value;
  }

  static RustBuffer Lower(const T& aValue) {
    size_t size = Serializable<T>::Size(aValue);
    // TODO: Ensure `size` doesn't overflow.
    auto buffer = {{ ci.ffi_bytebuffer_alloc().name() }}(static_cast<uint32_t>(size));
    auto writer = Writer(buffer);
    // TODO: Remove errors for `WriteInto`.
    Serializable<T>::WriteInto(aValue, writer).unwrap();
    return buffer;
  }
};
