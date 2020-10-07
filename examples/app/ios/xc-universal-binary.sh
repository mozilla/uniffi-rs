#!/usr/bin/env bash
set -eEuvx

function error_help()
{
    ERROR_MSG="It looks like something went wrong building the Example App Universal Binary."
    echo "error: ${ERROR_MSG}"
}
trap error_help ERR

# XCode tries to be helpful and overwrites the PATH. Reset that.
PATH="$(bash -l -c 'echo $PATH')"

# This should be invoked from inside xcode, not manually
if [[ "${#}" -ne 4 ]]
then
    echo "Usage (note: only call inside xcode!):"
    echo "path/to/build-scripts/xc-universal-binary.sh <STATIC_LIB_NAME> <FFI_TARGET> <SRC_ROOT_PATH> <buildvariant>"
    exit 1
fi
# e.g. liblogins_ffi.a
STATIC_LIB_NAME=${1}
# what to pass to cargo build -p, e.g. logins_ffi
FFI_TARGET=${2}
# path to app services root
SRC_ROOT=${3}
# buildvariant from our xcconfigs 
BUILDVARIANT=$(echo "${4}" | tr '[:upper:]' '[:lower:]') 

RELFLAG=
RELDIR="debug"
if [[ "${BUILDVARIANT}" != "debug" ]]; then
    RELFLAG=--release
    RELDIR=release
fi

TARGETDIR=${SRC_ROOT}/target

# We can't use cargo lipo because we can't link to universal libraries :(
# https://github.com/rust-lang/rust/issues/55235
LIBS_ARCHS=("x86_64" "arm64")
IOS_TRIPLES=("x86_64-apple-ios" "aarch64-apple-ios")
for i in "${!LIBS_ARCHS[@]}"; do
    env -i PATH="${PATH}" \
    "${HOME}"/.cargo/bin/cargo build --locked -p "${FFI_TARGET}" --lib ${RELFLAG} --target "${IOS_TRIPLES[${i}]}"
done

UNIVERSAL_BINARY=${TARGETDIR}/universal/${RELDIR}/${STATIC_LIB_NAME}
NEED_LIPO=

# if the universal binary doesnt exist, or if it's older than the static libs,
# we need to run `lipo` again.
if [[ ! -f "${UNIVERSAL_BINARY}" ]]; then
    NEED_LIPO=1
elif [[ "$(stat -f "%m" "${TARGETDIR}/x86_64-apple-ios/${RELDIR}/${STATIC_LIB_NAME}")" -gt "$(stat -f "%m" "${UNIVERSAL_BINARY}")" ]]; then
    NEED_LIPO=1
elif [[ "$(stat -f "%m" "${TARGETDIR}/aarch64-apple-ios/${RELDIR}/${STATIC_LIB_NAME}")" -gt "$(stat -f "%m" "${UNIVERSAL_BINARY}")" ]]; then
    NEED_LIPO=1
fi
if [[ "${NEED_LIPO}" = "1" ]]; then
    mkdir -p "${TARGETDIR}/universal/${RELDIR}"
    lipo -create -output "${UNIVERSAL_BINARY}" \
        "${TARGETDIR}/x86_64-apple-ios/${RELDIR}/${STATIC_LIB_NAME}" \
        "${TARGETDIR}/aarch64-apple-ios/${RELDIR}/${STATIC_LIB_NAME}"
fi
