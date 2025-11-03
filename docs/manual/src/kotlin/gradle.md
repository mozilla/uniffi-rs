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
See [Built-in types](../types/builtin_types.md).

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

## Advise on Rust -> Java interop

It's highly recommend to call `AttachCurrentThread` when spawning a Rust thread, and in that thread we need to call Java functions, maybe through UniFFI's foreign interface. Otherwise, [JNA] has to attach and detach a Java thread into the native thread in every function call, which is a heavy operation.

```rust
static VM: once_cell::sync::OnceCell<jni::JavaVM> = once_cell::sync::OnceCell::new();

// need call this function in java/kotlin first
// Before Rust 1.85 use #[export_name = "Java_com_xxx_xxx"]
// Since Rust 1.85 use #[unsafe(export_name = "Java_com_xxx_xxx")]
#[unsafe(export_name = "Java_com_xxx_xxx")]
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
}).build().unwrap();
```

To call `Java_com_xxx_xxx` you need to replace `_xxx_xxx` with the right package
name and the right class name from where the function will be called.
For example:

In Rust:

```rust
// Rust (see above for before / after Rust 1.85)
#[unsafe(export_name = "Java_com_my_package_MainActivity")]
pub extern "system" fn java_init(
// ...
```

And then in Kotlin:

```kotlin
// Kotlin
package my.package

class MainActivity : ComponentActivity() {
    external fun javaInit()

    override fun onCreate(savedInstanceState: Bundle?) {
        // Loads "libmy_library.so" so that it can be called by "external fun" above
        System.loadLibrary("my_library")
        // Calls the Java_com_my_package_MainActivity function in Rust
        javaInit()
    }
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
