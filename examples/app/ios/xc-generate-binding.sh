#!/usr/bin/env bash
set -eEuvx

function error_help()
{
    ERROR_MSG="It looks like something went wrong building the Example App Universal Binary."
    echo "error: ${ERROR_MSG}"
}
trap error_help ERR

EXAMPLES="$SRCROOT/../../../examples/"
for UDL in "$EXAMPLES/arithmetic/src/arithmetic.udl" "$EXAMPLES/todolist/src/todolist.udl"; do
  echo "Generating files for $UDL"
  "$SRCROOT/../../../target/debug/uniffi-bindgen" generate "$UDL" --language swift --out-dir "$SRCROOT/Generated"
done
