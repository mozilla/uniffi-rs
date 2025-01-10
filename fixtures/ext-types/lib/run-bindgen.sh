#!/usr/bin/env bash

set -eou pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_DIR="$SCRIPT_DIR/../../.."

export MANIFEST_FILE="$REPO_DIR/Cargo.toml"
export RUST_PACKAGE="uniffi-fixture-ext-types"
export LIBRARY_NAME="libuniffi_ext_types_lib.a"

# clean before generating
rm -rf "$REPO_DIR/target/swift" \
    "$REPO_DIR/target/swift-package" \
    "$REPO_DIR/target/apple-ios-sim" \
    "$REPO_DIR/target/apple-macos"

"$REPO_DIR/examples/app/swift-package/bindgen.sh"

(cd "$REPO_DIR/target/swift-package/UniffiFixtureExtTypesFFI"; swift build)
