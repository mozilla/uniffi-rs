#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

# Key points for these cmd-line args:
#  * run a transient image that deletes itself on successful completion
#  * `--init` fixes CTRL+C
#  * share cargo registry data with the image, so it doesn't have to re-fetch it each run
#  * mount the current working directory into the image, and run in that directory
#  * `--group-add` adds `circleci` user to the host group, allowing container to access host user files
#  * run via bash in interactive mode to get correct $PATH setup from .bashrc
docker run \
    -ti --rm --init \
    -v $HOME/.cargo/registry:/home/circleci/.cargo/registry \
    -v $ROOT_DIR:/mounted_workdir \
    -w /mounted_workdir/$(realpath -m --relative-to=$ROOT_DIR $PWD) \
    --group-add $(id -g) \
    rfkelly/uniffi-ci:latest bash -i -c "umask 0002 && $*"