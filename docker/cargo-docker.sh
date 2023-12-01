#!/usr/bin/env bash

# Key points for these cmd-line args:
#  * run a transient image that deletes itself on successful completion
#  * share cargo registry data with the image, so it doesn't have to re-fetch it each run
#  * mount the current working directory into the image, and run in that directory
#  * run via bash in interactive mode to get correct $PATH setup from .bashrc
docker run \
    -ti --rm \
    -v $HOME/.cargo/registry:/usr/local/cargo/registry \
    -v $PWD:/mounted_workdir \
    -w /mounted_workdir \
    rfkelly/uniffi-ci:latest bash -i -c "cargo $*"
