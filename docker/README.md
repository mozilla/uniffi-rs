This directory contains a Dockerfile for building the
`uniffi-ci` docker image that we use for running tests
in CI. To build a new version of this docker image, use:

```
docker build -t rfkelly/uniffi-ci -f Dockerfile-build .
docker push rfkelly/uniffi-ci
```

That only works if you're `rfkelly`; we need to figure out
a better strategy for maintainership of said docker image.
