# UniFFI Benchmarks Android App

Android application for running UniFFI benchmark tests on Android devices.

## Setup

### 1. Install Rust Android Targets

```bash
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

### 2. Configure Android HOME

Make sure you have the Android NDK installed via Android Studio or the command line tools.

Set the `ANDROID_HOME` environment variable to the android `Sdk` directory

## Building the App

Build and install on device connected to ADB

```bash
./gradlew installDebug
```

## Interpreting results

The benchmark variance is very high so you need to use some judgement when interpreting the results.
In particular, the first run seems to be consistently higher than subsequent runs.
