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
JNA 5.7 or greater is required.

Set the dependency in your `build.gradle`:

```groovy
dependencies {
    implementation "net.java.dev.jna:jna:5.7.0@aar"
}
```

[JNA]: https://github.com/java-native-access/jna
