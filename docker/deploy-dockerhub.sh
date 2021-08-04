#!/bin/bash

# THIS IS MEANT TO BE RUN BY CI

set -e

# Usage: retry MAX CMD...
# Retry CMD up to MAX times. If it fails MAX times, returns failure.
# Example: retry 3 docker push "$DOCKERHUB_REPO:$TAG"
function retry() {
    max=$1
    shift
    count=1
    until "$@"; do
        count=$((count + 1))
        if [[ $count -gt $max ]]; then
            return 1
        fi
        echo "$count / $max"
    done
    return 0
}

# configure docker creds
retry 3 echo "$DOCKER_PASS" | docker login -u="$DOCKER_USER" --password-stdin

# docker tag and push to dockerhub
if [ -n "$1" ]; then
    TAG="$1"
    docker tag uniffi-ci:build "$DOCKERHUB_REPO:$TAG" ||
        (echo "Couldn't tag uniffi-ci:build as $DOCKERHUB_REPO:$TAG" && false)
    retry 3 docker push "$DOCKERHUB_REPO:$TAG" ||
        (echo "Couldn't push $DOCKERHUB_REPO:$TAG" && false)
    echo "Pushed $DOCKERHUB_REPO:$TAG"
fi
