#!/usr/bin/env bash

set -eou pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_DIR="$SCRIPT_DIR/../../.."

if [[ "${MANIFEST_FILE:-}" == "" ]]; then
    printf "MANIFEST_FILE env var not set!\n" 1>&2
    exit 1
fi

if [[ "${RUST_PACKAGE:-}" == "" ]]; then
    printf "RUST_PACKAGE env var not set!\n" 1>&2
    exit 1
fi

DEFAULT_XCFRAMEWORK_MODULE_NAME="$(echo -n "$RUST_PACKAGE" | sed -e 's/-/_/g' | python3 -c "print(''.join(x.title() for x in input().split('_')) + 'FFI')")"
XCFRAMEWORK_MODULE_NAME="${XCFRAMEWORK_MODULE_NAME:-$DEFAULT_XCFRAMEWORK_MODULE_NAME}"

DEFAULT_OUTPUT_DIR="$REPO_DIR/target/swift-package/$XCFRAMEWORK_MODULE_NAME"
OUTPUT_DIR="${OUTPUT_DIR:-$DEFAULT_OUTPUT_DIR}"
XCFRAMEWORK_DIR="$OUTPUT_DIR/$XCFRAMEWORK_MODULE_NAME.xcframework"

if [[ -e "$OUTPUT_DIR" ]]; then
    printf "The OUTPUT_DIR '%s' must not exist before running\n" "$OUTPUT_DIR" 1>&2
    exit 1
fi

DEFAULT_LIBRARY_NAME="lib${RUST_PACKAGE}.a"
LIBRARY_NAME="${LIBRARY_NAME:-$DEFAULT_LIBRARY_NAME}"

if [[ "${UNIFFI_BINDGEN_SWIFT:-}" == "" ]]; then
    UNIFFI_BINDGEN_SWIFT="$REPO_DIR/target/release/uniffi-bindgen-swift"
    cargo build --manifest-path "$REPO_DIR/Cargo.toml" --release --bin uniffi-bindgen-swift
fi

rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

cargo build --manifest-path "$REPO_DIR/Cargo.toml" --release --package "$RUST_PACKAGE" \
    --target x86_64-apple-ios \
    --target aarch64-apple-ios-sim \
    --target aarch64-apple-ios \
    --target x86_64-apple-darwin \
    --target aarch64-apple-darwin

# Generate Swift files based on sample library
SWIFT_FILES_DIR="$REPO_DIR/target/swift/$RUST_PACKAGE"
mkdir -p "$SWIFT_FILES_DIR"
"$UNIFFI_BINDGEN_SWIFT" \
    --swift-sources "$REPO_DIR/target/aarch64-apple-darwin/release/$LIBRARY_NAME" \
    "$SWIFT_FILES_DIR"

# Generate headers based on sample library
HEADER_FILES_DIR="$REPO_DIR/target/include/$RUST_PACKAGE"
"$UNIFFI_BINDGEN_SWIFT" --module-name "$XCFRAMEWORK_MODULE_NAME" \
    --headers --modulemap --modulemap-filename module.modulemap \
    "$REPO_DIR/target/aarch64-apple-darwin/release/$LIBRARY_NAME" \
    "$HEADER_FILES_DIR"

mkdir -p "$XCFRAMEWORK_DIR"

SNAKE_NAMES=()
CAMEL_NAMES=()
while IFS= read -r -d $'\0'; do
    FILENAME="$REPLY"
    SNAKE_NAME="$(basename "$FILENAME" | sed -e 's/\.swift//g')"
    CAMEL_NAME="$(echo -n "$SNAKE_NAME" | python3 -c "print(''.join(x.title() for x in input().split('_')) + 'FFI')")"
    SNAKE_NAMES+=("$SNAKE_NAME")
    CAMEL_NAMES+=("$CAMEL_NAME")
done < <(find "$SWIFT_FILES_DIR" -name '*.swift' -print0)
MODULE_COUNT="${#SNAKE_NAMES[@]}"

# Create fat library for iOS Simulator
IOS_SIM_FAT_LIBRARY="$REPO_DIR/target/apple-ios-sim/release/$LIBRARY_NAME"
printf "Creating iOS sim fat library at path '%s'\n" "$IOS_SIM_FAT_LIBRARY"
mkdir -p "$(dirname "$IOS_SIM_FAT_LIBRARY")"
lipo -create "$REPO_DIR/target/aarch64-apple-ios-sim/release/$LIBRARY_NAME" \
    "$REPO_DIR/target/x86_64-apple-ios/release/$LIBRARY_NAME" \
    -output "$IOS_SIM_FAT_LIBRARY"

# Create fat library for macOS
MACOS_FAT_LIBRARY="$REPO_DIR/target/apple-macos/release/$LIBRARY_NAME"
printf "Creating macOS fat library at path '%s'\n" "$MACOS_FAT_LIBRARY"
mkdir -p "$(dirname "$MACOS_FAT_LIBRARY")"
lipo -create "$REPO_DIR/target/aarch64-apple-darwin/release/$LIBRARY_NAME" \
    "$REPO_DIR/target/x86_64-apple-darwin/release/$LIBRARY_NAME" \
    -output "$MACOS_FAT_LIBRARY"

printf "Creating XCFramework at path '%s'\n" "$XCFRAMEWORK_DIR"
xcodebuild -create-xcframework \
    -library "$IOS_SIM_FAT_LIBRARY" \
    -headers "$HEADER_FILES_DIR" \
    -library "$MACOS_FAT_LIBRARY" \
    -headers "$HEADER_FILES_DIR" \
    -library "$REPO_DIR/target/aarch64-apple-ios/release/$LIBRARY_NAME" \
    -headers "$HEADER_FILES_DIR" \
    -output "$XCFRAMEWORK_DIR"

PKG_FILE="$OUTPUT_DIR/Package.swift"

cat >"$PKG_FILE" <<EOF
// swift-tools-version:6.0
import PackageDescription

let package = Package(
    name: "$XCFRAMEWORK_MODULE_NAME",
    platforms: [.iOS(.v18), .macOS(.v15)],
EOF
printf >>"$PKG_FILE" "    products: ["

for ((i = 0; i < "$MODULE_COUNT"; ++i)); do
    if [[ "$i" != "0" ]]; then
        printf >>"$PKG_FILE" ",\n"
    else
        printf >>"$PKG_FILE" "\n"
    fi
    printf >>"$PKG_FILE" "        .library(\n"
    printf >>"$PKG_FILE" "            name: \"%s\", targets: [\"%s\"]\n" "${CAMEL_NAMES[$i]}" "${CAMEL_NAMES[$i]}"
    printf >>"$PKG_FILE" "        )"
done
printf >>"$PKG_FILE" "\n"

cat >>"$PKG_FILE" <<EOF
    ],
    targets: [
EOF

for ((i = 0; i < "$MODULE_COUNT"; ++i)); do
    MODULE_SRC_DIR="$OUTPUT_DIR/Sources/${CAMEL_NAMES[$i]}"
    mkdir -p "$MODULE_SRC_DIR"
    sed >"$MODULE_SRC_DIR/${CAMEL_NAMES[$i]}.swift" \
        -e "s/${SNAKE_NAMES[$i]}FFI/$XCFRAMEWORK_MODULE_NAME/g" \
        -e "s/private var initializationResult/private nonisolated(unsafe) var initializationResult/g" \
        -e "s/static var vtable:/static nonisolated(unsafe) var vtable:/g" \
        -e "s/fileprivate static var handleMap =/fileprivate static nonisolated(unsafe) var handleMap =/g" \
        -e "s/fileprivate let uniffiContinuationHandleMap =/fileprivate nonisolated(unsafe) let uniffiContinuationHandleMap =/g" \
        -e "s/\(public struct [[:alnum:]]\{1,\}\)/\1: Sendable/g" \
        -e "s/\(public enum [[:alnum:]]\{1,\}\)/\1: Sendable/g" \
        -e "s/\(public struct FfiConverter[[:alnum:]]\{1,\}\): Sendable/\1/g" \
        "$SWIFT_FILES_DIR/${SNAKE_NAMES[$i]}.swift"
    cat >>"$PKG_FILE" <<EOF
        .target(
            name: "${CAMEL_NAMES[$i]}",
            dependencies: [
                .target(name: "$XCFRAMEWORK_MODULE_NAME")
                
            ]
        ),
EOF
done

cat >>"$PKG_FILE" <<EOF
        .binaryTarget(
            name: "$XCFRAMEWORK_MODULE_NAME",
            path: "./$XCFRAMEWORK_MODULE_NAME.xcframework"
            
        )
    ]
)    
EOF
