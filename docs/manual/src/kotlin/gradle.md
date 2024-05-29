# Integrating with Gradle

It is possible to generate Kotlin bindings at compile time for Kotlin Android projects. We'd like to make a gradle plugin for that, but until then you can add to your `build.gradle` the following:

```groovy
android.libraryVariants.all { variant ->
    def t = tasks.register("generate${variant.name.capitalize()}UniFFIBindings", Exec) {
        workingDir "${project.projectDir}"
        // Runs the bindings generation, note that you must have uniffi-bindgen installed and in your PATH environment variable
        commandLine 'uniffi-bindgen', 'generate', '<PATH TO .udl FILE>', '--language', 'kotlin', '--out-dir', "${buildDir}/generated/source/uniffi/${variant.name}/java"
    }
    variant.javaCompileProvider.get().dependsOn(t)
    def sourceSet = variant.sourceSets.find { it.name == variant.name }
    sourceSet.java.srcDir new File(buildDir, "generated/source/uniffi/${variant.name}/java")
    // XXX: I've been trying to make this work but I can't, so the compiled bindings will show as "regular sources" in Android Studio.
    idea.module.generatedSourceDirs += file("${buildDir}/generated/source/uniffi/${variant.name}/java/uniffi")
}
```

The generated bindings should appear in the project sources in Android Studio.

## Using experimental unsigned types

Unsigned integers in the defined API are translated to their equivalents in the foreign language binding, e.g. `u32` becomes Kotlin's `UInt` type.
See [Built-in types](../udl/builtin_types.md).

However unsigned integer types are experimental in Kotlin versions prior to 1.5.
As such they require explicit annotations to suppress warnings.
Uniffi is trying to add these annotations where necessary,
but currently misses some places, see [PR #977](https://github.com/mozilla/uniffi-rs/pull/977) for details.

To suppress all warnings for experimental unsigned types add this to your project's `build.gradle` file:

```groovy
allprojects {
   tasks.withType(org.jetbrains.kotlin.gradle.tasks.KotlinCompile).all {
        kotlinOptions {
            freeCompilerArgs += [
                "-Xuse-experimental=kotlin.ExperimentalUnsignedTypes",
            ]
        }
    }
}
```

> ## Update
>
> As of [PR #993](https://github.com/mozilla/uniffi-rs/pull/993), the Kotlin backend was refactored, and it became harder to support the
> `@ExperimentalUnsignedTypes` annotation. Uniffi's Android customers are rapidly moving toward Kotlin 1.5, so adding this compiler arg is no longer necessary.

## JNA dependency

UniFFI relies on [JNA] for the ability to call native methods.
JNA 5.12.0 or greater is required.

Set the dependency in your `build.gradle`:

```groovy
dependencies {
    implementation "net.java.dev.jna:jna:5.12.0@aar"
}
```

## Advise on rust->java interop

It's highly recommend to call `AttachCurrentThread` when spawning a rust thread, and in that thread we need to call java functions, maybe through uniffi's foreign interface. Otherwise, [JNA] has to attach and detach java thread into native thread in every function call, which is obviously a heavy operation.

```rust
static VM: once_cell::sync::OnceCell<jni::JavaVM> = once_cell::sync::OnceCell::new();

// need call this function in java/kotlin first
#[export_name = "Java_com_xxx_xxx"]
pub extern "system" fn java_init(
    env: jni::JNIEnv,
    _class: jni::objects::JClass,
    app: jni::objects::JObject,
) {
    let vm = env.get_java_vm().unwrap();
    _ = VM.set(vm);
}

// take tokio for example, and we need to call uniffi's callback in the tokio worker threads -
tokio::runtime::Builder::new_multi_thread().on_thread_start(|| {
    let vm = VM.get().expect("init java vm");
    vm.attach_current_thread_permanently().unwrap();
})
```

[JNA]: https://github.com/java-native-access/jna

## Coroutines dependency

UniFFI relies on [kotlinx coroutines core] for future and async support. Version 1.6 or greater is required.

Set the dependency in your `build.gradle`:

```groovy
dependencies {
    implementation "org.jetbrains.kotlinx:kotlinx-coroutines-core:1.6.4"
}
```

[kotlinx coroutines core]: https://github.com/Kotlin/kotlinx.coroutines
