#!/usr/bin/env bash

set -ex

case "$OSTYPE" in
  darwin*)  DLL_EXT=".dylib" ;;
  msys*)    DLL_EXT=".dll" ;;
  *)        DLL_EXT=".so" ;;
esac

CRATE_ROOT=$(dirname $0)
TARGET_DIR=${CRATE_ROOT}/../../target/
LIBRARY_PATH=${TARGET_DIR}/debug/libuniffi_fixture_version_mismatch${DLL_EXT}
WORK_DIR=${TARGET_DIR}/version-mismatch-workdir

# Setup the work dir
if test -e "${WORK_DIR}"; then rm -r ${WORK_DIR}; fi
mkdir -p ${WORK_DIR}

# Build the library and generate the bindings
cargo build
cargo run -p uniffi-fixture-version-mismatch --bin bindgen -- generate src/api_v1.udl --lib-file $LIBRARY_PATH --language kotlin --out-dir ${WORK_DIR} --no-format

# Rebuild library using features to change the API wrapped with proc-macros
cargo build --features=proc_macro_v2

# Try to run the two together
cp ${LIBRARY_PATH} ${WORK_DIR}
cp ${CRATE_ROOT}/bindings/kotlin_test.kts ${WORK_DIR}
cd ${WORK_DIR}

kotlinc -Werror -d ${WORK_DIR}/uniffi-example-arithmetic.jar -classpath ${CLASSPATH} ${WORK_DIR}/uniffi/fixture_version_mismatch/fixture_version_mismatch.kt
kotlinc -classpath ${WORK_DIR}:${WORK_DIR}/uniffi-example-arithmetic.jar:${CLASSPATH} -J-ea -Werror -script ${WORK_DIR}/kotlin_test.kts
