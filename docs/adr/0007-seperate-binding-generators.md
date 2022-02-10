# Separate the binding generators into their own crates per target

* Status: proposed
* Deciders: TBD
* Date: 2022-01-16

Technical Story: [Issue 299](https://github.com/mozilla/uniffi-rs/issues/299)

outdated early prototype: [PR 997](https://github.com/mozilla/uniffi-rs/pull/997)
updated prototype: [PR 1157](https://github.com/mozilla/uniffi-rs/pull/1157)

## Context and Problem Statement
All the binding generators currently live in the [`uniffi_bindgen`](../../uniffi_bindgen/src/bindings) crate. This creates the following difficulties:

- All the bindings live in the `uniffi` repository, so the `uniffi` team has to maintain them (or at the very least review changes to them).
- Any change to a specific binding generator requires a new `uniffi_bindgen` release for it to be accessible by consumers. Even if it doesn't impact any of the other bindings.
- Some bindings require complex build systems to test. Including those build systems in `uniffi` would require developers to install those build systems, and CI to do the same. For example, any type of `gecko-js` bindings would require the mozilla-central build system to build and test.
- We currently run all the tests for the bindings in our CI and through `cargo test`. This means that if one binding target gets outdated and fails, or if a developer doesn't have the needed libraries installed for one of the targets, the tests would fail.
- It's currently difficult to support third-party developers writing bindings for languages the core team does not wish to maintain.

This ADR re-evaluates the architecture of how we consume the bindings in hope of improving the developer experience of adding new bindings, testing new and the current bindings and clarifying the ownership of the bindings.
## Decision Drivers

* Maintainability, we would like to improve the maintainability of our binding generators - this means clear ownership of the bindings.
* Testability, it should be easy for developers to test the bindings they care about. Without having to navigate and install unfamiliar libraries and build systems.
* Version compatibility, this refers to the same version of `uniffi` being used to generate both the scaffolding and the bindings. This includes:
    - How easy is it to accidentally have a version mismatch
    - How much damage is caused by the the version mismatch
    - The difficulty of debugging a version mismatch
* Developer experience, it should be easier to write and maintain a new binding generator than it currently is.
* Releases, cutting releases for changes in one binding generator shouldn't harm another.

## Considered Options

* **[Option 1] Do nothing**

    This means keeping everything as-is, and deciding that all binding generators (at least for now) should live under the `uniffi_bindgen` crate.

* **[Option 2] Publish a separate crate per binding generator and have users use the  `uniffi_bindgen` Command line interface to shell out to the specific binding generators**

    Users would have to install a binary of the crates of the binding language they would like to use **in addition** to installing `uniffi_bindgen`. For example, a user would need to:

    1. run `cargo install uniffi_bindgen_kotlin`.
    1. run `cargo install uniffi_bindgen`.
    1. Then a user will finally be able to run `uniffi_bindgen generate -l kotlin <ARGS>`. Users must install `uniffi_bindgen_kotlin` otherwise `uniffi_bindgen` would fail.

    This means that `uniffi_bindgen` would still handle generic tasks related to binding generation. This includes:

    1. Parsing the UDL, and passing the `ComponentInterface` to the specific binding generator.
    1. Parsing the `uniffi.toml` and passing the language specific configuration down to the binding generator.
    1. Calling a generic `BindingGenerator::write_bindings` function that is implemented by the specific binding generator.
    1. `uniffi_bindgen` would also expose its `CodeOracle`, `CodeType` and `CodeDeclaration` types to help developers create a standard way to interact with code generation. It shouldn't however, restrict developers to use those concepts if they don't fit a specific binding.
    But developers would still need a separate crate that implements the traits `uniffi_bindgen` exposes.

* **[Option 3] Publish a separate crate per binding generator and have them get consumed directly**

    Users would only need to install `uniffi_bindgen_{lang}` and run it like `uniffi_bindgen_kotlin <ARGS>`, and each `uniffi_bindgen_{lang}` could opt to support generating scaffolding as well.
## Pros and Cons of the Options

### **[Option 1] Do nothing**

* Good, because it makes it hard to accidentally use different versions of `uniffi` for scaffolding and bindings.
* Good, it makes it easier to make changes to multiple bindings at a time (in the case of a breaking change in `uniffi`, etc).
* Bad, because maintainability can grow to be difficult - especially if more bindings are added which the core `uniffi` team is not familiar with.
* Bad, because testability can also grow to be difficult - as more bindings are added (some type of `gecko-js` is expected in the future) the requirement to test all the bindings together in one repository is difficult to maintain.
* Bad, because releases of all the binding generators are tied to the release of `uniffi_bindgen`.

### **[Option 2] Publish a separate crate per binding generator and have users use the  `uniffi_bindgen` Command line interface to shell out to the specific binding generators**

* Good, because ownership will be clear, and members of the community can opt to maintain their own binding generators.
* Good, because testability can improve, where a developer can test a binding generator without building or testing the others.
* Good, because our CI would only need to test the core bindings we maintain, and others can be tested by their own maintainers (for example, a `gecko-js` binding generator should be tested in `mozilla-central` and not here).
* Good, because a release in one binding wouldn't have an impact on any other ones unless it changes internal `uniffi` behavior.
* Good, because it makes it easier to catch a version mismatch between scaffolding and bindings - since the version is passed between the binaries.
* Bad, because it's easier to accidentally have a version mismatch.
* Bad, because testability might increase in complexity. Bindings not in this repository will not have access to the extensive fixtures and examples we have. We might need to expose those if take this option.
* Bad because it increases the number of crates we manage. As we would manage the core crates for the bindings we depend on (i.e kotlin, swift, python, and soon a C++/gecko one). This will make the release process painful, but we can add automation and clear crate ownership to help.

Overall this option is preferred because:

- The tradeoffs are clear and manageable and because there is a raising need to create bindings for gecko-js, which can't be tested end-to-end without a complex build system.
- The benefit from losing that extra hop out of uniffi_bindgen isn't worth having our consumers try to enforce compatibility across different `uniffi_bindgen_{lang}` (especially like Glean and App Services where we have multiple bindings at a time).
- `uniffi_bindgen` staying as the source of truth (with it passing it's version to the other crates) has a good benefit of reducing mental load to keep track of which binary to call.


### **[Option 3] Publish a separate crate per binding generator and have them get consumed directly**

* Good, because maintainability will be clear, and members of the community can opt to maintain their own binding generators.
* Good, because testability can improve, where a developer can test a binding generator without building or testing the others.
* Good, because our CI would only need to test the core bindings we maintain, and others can be tested by their own maintainers (for example, a `gecko-js` binding generator should be tested in `mozilla-central` and not here).
* Good, because a release in one binding wouldn't have an impact on any other ones unless it changes internal `uniffi` behavior.
* Good, because users would not have to install multiple binaries. Just the ones they care about.
* Bad, because it's easier to accidentally have a version mismatch. And we have even less flexibility than `Option 2` to mitigate it since `uniffi_bindgen` won't shell out to the crate.
* Bad, because it's not clear how the users should generate the scaffolding, and if they use `uniffi_bindgen` there is a risk of version incompatibility.
* Bad, because testability might increase in complexity. Bindings not in this repository will not have access to the extensive fixtures and examples we have. We might need to expose those if take this option.
* Bad because it increases the number of crates we manage. As we would manage the core crates for the bindings we depend on (i.e kotlin, swift, python, and soon a C++/gecko one). This will make the release process painful, but we can add automation and clear crate ownership to help.


## Decision Outcome

Chosen option:

* **[Option 2] Publish a separate crate per binding generator and have users use the  `uniffi_bindgen` Command line interface to shell out to the specific binding generators**

## Changes Proposed
### Expose a trait `BindingGenerator`
The trait would have the following:
1. An associated type that implements `BindingGeneratorConfig`
    - `BindingGeneratorConfig` would be another trait, that binding generators can implement on their own configuration types. The purpose of this type is to carry any binding specific configuration parsed from the `uniffi.toml`
    - `BindingGeneratorConfig` requires implementing `serde::Deserialize`, `MergeWith`, `Default`, `Debug` and `From<&ComponentInterface>`.
    - `BindingGeneratorConfig` definition:
    ```rs
    pub trait BindingGeneratorConfig: for<'de> Deserialize<'de> + MergeWith + for<'a> From<&'a ComponentInterface> + Default + std::fmt::Debug {}
    ```
1. A function `write_bindings` that takes in the ComponentInterface and Config and writes the bindings into directory `out_dir`
    - Definition:
    ```rs
    fn write_bindings<P: AsRef<Path>>(&self, ci: &ComponentInterface, out_dir: P) -> Result<()>;
    ```
1. A function compile_bindings that compiles the bindings generated by the `write_bindings` function. Note: this is mostly only useful for **testing**. 
    - Definition:
    ```rs
    fn compile_bindings<P: AsRef<Path>>(&self, ci: &ComponentInterface, out_dir: P) -> Result<()>;
    ```
1. A function `run_script` that runs a script, against the bindings that were compiled with `compile_bindings`. Note: this is mostly only useful for **testing**
    - Definition:
    ```rs
    fn run_script<P: AsRef<Path>>(&self, out_dir: P, script_file: P) -> Result<()>;
    ```
#### Full Traits definition:
```rs
pub trait BindingGeneratorConfig: for<'de> Deserialize<'de> + MergeWith + for<'a> From<&'a ComponentInterface> + Default + std::fmt::Debug {}

pub trait BindingGenerator: Sized {
    type Config: BindingGeneratorConfig;
    fn new(config: Self::Config) -> anyhow::Result<Self>;

    fn write_bindings<P: AsRef<Path>>(&self, ci: &ComponentInterface, out_dir: P) -> anyhow::Result<()>;

   
    fn compile_bindings<P: AsRef<Path>>(
        &self,
        ci: &ComponentInterface,
        out_dir: P,
    ) -> anyhow::Result<()>;

    fn run_script<P: AsRef<Path>>(&self, out_dir: P, script_file: P) -> anyhow::Result<()>;
}
```
### Expose generic functions as entry points
1. The binding generator should call a generic function when generating bindings exposed by `uniffi_bindgen`. The generic function will:
    - Parse the UDL.
    - Parse the configuration from `uniffi.toml`, using the `BindingGeneratorConfig` trait the consumer implements.
    - Initialize a `BindingGenerator`, with the type a consumer provides.
    - Call `write_bindings` on the generic type.
    - Definition:
        ```rs
        pub fn generate_bindings<B: BindingGenerator, P: AsRef<Path>>(
            udl_file: P,
            config_file_override: Option<P>,
            out_dir_override: Option<P>,
            _try_format_code: bool
        ) -> Result<()>
        ```
1. The binding generator should call another generic function when running tests. This generic function will:
    - Parse the UDL.
    - Parse the configuration from `uniffi.toml`, using the `BindingGeneratorConfig` trait the consumer implements.
    - Initialize a `BindingGenerator`, with the type a consumer provides.
    - Call `write_bindings` on the generic type.
    - Compile the bindings using `compile_bindings`
    - Then run a script, using `run_script` and fail if the script returns an error.
    - Definition:
        ```rs
        pub fn run_test<B: BindingGenerator, P: AsRef<Path>>(
            udl_file: P,
            script_file: P,
            config_file_override: Option<P>,
            out_dir_override: Option<P>,
            _try_format_code: bool
        ) -> Result<()>

        ```

Using this approach, users would install both the `uniffi_bindgen` **binary** and the `uniffi_bindgen_kotlin` (using kotlin as an example) **binary**. Then, the user would run the following:
1. The user runs `uniffi-bindgen generate -l kotlin -o <OUT_PATH> <UDL_PATH>`
1. `uniffi-bindgen` gets its own version, let's say `0.16` as an example.
1. `uniffi-bindgen` shells out to `uniffi-bingen-kotlin -o <OUT_PATH> -v 0.16 <UDL_PATH>`
1. `uniffi-bindgen-kotlin` asserts that it was compiled using the same `0.16` version, otherwise it panics.
1. `uniffi-bindgen-kotlin` then calls a `generate_bindings` function in `uniffi-bindgen` (as a library dependency)
1. `uniffi-bindgen`'s `generate_bindings` takes a generic parameter type that implements a trait `BindingGenerator`.
1. `uniffi-bindgen` parses the `udl`, then calls the generic type's `write_bindings` function (the implementation lives in `uniffi-bingen-kotlin`)


## Example Consumer implementation for a Kotlin binding generator:
```rs
pub struct KotlinBackend {
    config: UniffiConfig
}

#[derive(Debug, Default, Deserialize)]
pub struct UniffiConfig {
    #[serde(default)]
    kotlin: Config
}

impl MergeWith for UniffiConfig {
    fn merge_with(&self, other: &Self) -> Self {
        Self {
            kotlin: self.kotlin.merge_with(&other.kotlin)
        }
    }
}

impl From<&ComponentInterface> for UniffiConfig {
    fn from(ci: &ComponentInterface) -> Self {
        Self {
            kotlin: ci.into()
        }
    }
}

impl BindingGenerator for KotlinBackend {
    type Config = UniffiConfig;
    fn new(config: UniffiConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config
        })
    }

    fn write_bindings<P: AsRef<Path>>(
        &self,
        ci: &uniffi_bindgen::interface::ComponentInterface,
        out_dir: P,
    ) -> anyhow::Result<()> {
        let mut kt_file = full_bindings_path(&self.config.kotlin, out_dir.as_ref())?;
        std::fs::create_dir_all(&kt_file)?;
        kt_file.push(format!("{}.kt", ci.namespace()));
        let mut f = File::create(&kt_file).context("Failed to create .kt file for bindings")?;
        write!(f, "{}", generate_bindings(&self.config.kotlin, ci)?)?;

        Ok(())
    }

    /// Generate kotlin bindings for the given namespace, then use the kotlin
    /// command-line tools to compile them into a .jar file.
    fn compile_bindings<P: AsRef<Path>>(
        &self,
        ci: &ComponentInterface,
        out_dir: P,
    ) -> anyhow::Result<()> {
        let mut kt_file = full_bindings_path(&self.config.kotlin, out_dir.as_ref())?;
        kt_file.push(format!("{}.kt", ci.namespace()));
        let mut jar_file = PathBuf::from(out_dir.as_ref());
        jar_file.push(format!("{}.jar", ci.namespace()));
        let status = Command::new("kotlinc")
            // Our generated bindings should not produce any warnings; fail tests if they do.
            .arg("-Werror")
            // Reflect $CLASSPATH from the environment, to help find `jna.jar`.
            .arg("-classpath")
            .arg(env::var("CLASSPATH").unwrap_or_else(|_| "".to_string()))
            .arg(&kt_file)
            .arg("-d")
            .arg(jar_file)
            .spawn()
            .context("Failed to spawn `kotlinc` to compile the bindings")?
            .wait()
            .context("Failed to wait for `kotlinc` when compiling the bindings")?;
        if !status.success() {
            bail!("running `kotlinc` failed")
        }
        Ok(())
    }

    /// Execute the specifed kotlin script, with classpath based on the generated
    /// artifacts in the given output directory.
    fn run_script<P: AsRef<Path>>(&self, out_dir: P, script_file: P) -> Result<()> {
        let mut classpath = env::var_os("CLASSPATH").unwrap_or_else(|| OsString::from(""));
        classpath.push(":");
        classpath.push(out_dir.as_ref());
        for entry in PathBuf::from(out_dir.as_ref())
            .read_dir()
            .context("Failed to list target directory when running Kotlin script")?
        {
            let entry = entry.context("Directory listing failed while running Kotlin script")?;
            if let Some(ext) = entry.path().extension() {
                if ext == "jar" {
                    classpath.push(":");
                    classpath.push(entry.path());
                }
            }
        }
        let mut cmd = Command::new("kotlinc");
        cmd.arg("-classpath").arg(classpath);
        cmd.arg("-Xopt-in=kotlin.ExperimentalUnsignedTypes");
        cmd.arg("-J-ea");
        cmd.arg("-Werror");
        cmd.arg("-script").arg(script_file.as_ref());
        let status = cmd
            .spawn()
            .context("Failed to spawn `kotlinc` to run Kotlin script")?
            .wait()
            .context("Failed to wait for `kotlinc` when running Kotlin script")?;
        if !status.success() {
            bail!("running `kotlinc` failed")
        }
        Ok(())
    }
}
```
<br />
<br />

## Releases and Breaking changes
With the chosen outcome, how crates are released is an important question. This mostly relevant for crates that would live in the `uniffi-rs` repository. Crates that would live outside the repository are not impacted.
- Breaking changes in any of the `uniffi` core crates (`uniffi`, `uniffi_bindgen` for the purpose of the bindings) should not be landed (merged onto the main branch) until the breaking changes are addressed in the binding generators that live in this repository.
- Releases for the core crates, should be followed (ideally at the same time) by releases for all the binding generators that live in this repository.

### Testing
An important consideration for splitting the binding generators is the testing story for crates that live outside of the main `uniffi-rs` repository. We have a high-level decision to make:
Should we expose our fixtures as testing standards? And if yes, how do we do that.
At the time of this ADR, [we created a ticket on Uniffi to discuss this as its own issue](https://github.com/mozilla/uniffi-rs/issues/1151)
In the meantime, we can have our own crates consume the fixtures as they do currently since they are all in the same repository.

