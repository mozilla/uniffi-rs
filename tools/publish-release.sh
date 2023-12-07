#!/bin/sh

if [[ `git status --porcelain` ]]; then
  echo "Uncommited changes."
  echo "Please commit changes and run this command again"
  echo
  echo "See docs/release.md for details"
  exit 2
fi

echo "Publishing current revision to crates.io"
echo "See docs/release.md for our release process"

while true; do
    read -p "Are you sure (yn)? " yn
    case $yn in
        [Yy]* ) break;;
        [Nn]* ) exit;;
        * ) echo "Please answer y or n.";;
    esac
done

set -ex

# Note: make sure these are ordered so that dependencies come before the crates that depend on them
cargo publish -p uniffi_checksum_derive
cargo publish -p uniffi_meta
cargo publish -p uniffi_core
cargo publish -p uniffi_testing
cargo publish -p uniffi_udl
cargo publish -p uniffi_bindgen
cargo publish -p uniffi_build
cargo publish -p uniffi_macros
cargo publish -p uniffi
