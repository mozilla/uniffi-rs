# Intro

The main idea is to build the bindings with:

python .\setup.py bdist_wheel --verbose ;
$wheelFile = Get-ChildItem -Path .\dist\ -Recurse -Include * ;
pip3 install $wheelFile --force-reinstall ;

Then you must create a setup.py file. This file will include the command to generate python .py bindings, link .dll and then build to pipy. Requires python version greater than 3.6.

# Create setup.py file.


## rustc

get rustc version from your setup.py file:

```python
def get_rustc_info():
    """
    Get the rustc info from `rustc --version --verbose`, parsed into a
    dictionary.
    """
    regex = re.compile(r"(?P<key>[^:]+)(: *(?P<value>\S+))")

    output = subprocess.check_output(["rustc", "--version", "--verbose"])

    data = {}
    for line in output.decode("utf-8").splitlines():
        match = regex.match(line)
        if match:
            d = match.groupdict()
            data[d["key"]] = d["value"]

    return data
```

## Macos compatibility

The macos compatibility depends of the files generated with the command. 

```python
def macos_compat(target):
    if target.startswith("aarch64-"):
        return "11.0"
    return "10.7"
```
	
	

## target

The uniffy-bindgen command will generate different output that you will need to guess for future operations:

```python
# The logic for specifying wheel tags in setuptools/wheel is very complex, hard
# to override, and is really meant for extensions that are compiled against
# libpython.so, not this case where we have a fairly portable Rust-compiled
# binary that should work across a number of Python versions. Therefore, we
# just skip all of its logic be overriding the `get_tag` method with something
# simple that only handles the cases we need.
class bdist_wheel(wheel.bdist_wheel.bdist_wheel):
    def get_tag(self):
        cpu, _, __ = target.partition("-")
        impl, abi_tag = "cp36", "abi3"
        if "-linux" in target:
            plat_name = f"linux_{cpu}"
        elif "-darwin" in target:
            compat = macos_compat(target).replace(".", "_")
            if cpu == "aarch64":
                cpu = "arm64"
            plat_name = f"macosx_{compat}_{cpu}"
        elif "-windows" in target:
            impl, abi_tag = "py3", "none"
            if cpu == "i686":
                plat_name = "win32"
            elif cpu == "x86_64":
                plat_name = "win_amd64"
            else:
                raise ValueError("Unsupported Windows platform")
        else:
            # Keep local wheel build on BSD/etc. working
            _, __, plat_name = super().get_tag()

        return (impl, abi_tag, plat_name)
```



## Get extension name

Guess extension from command target flag:

```python
target = os.environ.get("GLEAN_BUILD_TARGET")
if not target:
    target = get_rustc_info()["host"]

extension = "" 
file_start = ""
if "-darwin" in target:
    shared_object = "libcryptatools_core.dylib"
    extension = ".dylib"
    file_start = "lib"
elif "-windows" in target:
    shared_object = "cryptatools_core.dll"
    extension = ".dll"
    file_start = ""
else:
    # Anything else must be an ELF platform - Linux, *BSD, Solaris/illumos
    shared_object = "libcryptatools_core.so"
    extension = ".so"
    file_start = "lib"

new_shared_object_name = file_start + "uniffi_cryptatools" + extension
```



```python
class InstallPlatlib(install):
    def finalize_options(self):
        install.finalize_options(self)
        if self.distribution.has_ext_modules():
            self.install_lib = self.install_platlib
```

# Build

```python
class build(_build):
    def run(self):
        try:
            # Use `check_output` to suppress output
            subprocess.check_output(["cargo"])
        except subprocess.CalledProcessError:
            print("Install Rust and Cargo through Rustup: https://rustup.rs/.")
            print(
                "Need help installing your project?"
            )
            sys.exit(1)

        env = os.environ.copy()

        # For `musl`-based targets (e.g. Alpine Linux), we need to set a flag
        # to produce a shared object Python extension.
        if "-musl" in target:
            env["RUSTFLAGS"] = (
                env.get("RUSTFLAGS", "") + " -C target-feature=-crt-static"
            )
        if target == "i686-pc-windows-gnu":
            env["RUSTFLAGS"] = env.get("RUSTFLAGS", "") + " -C panic=abort"

        command = [
            "cargo",
            "build",
            "--package",
            "library",
            "--target",
            target,
        ]

        if buildvariant != "debug":
            command.append(f"--{buildvariant}")

        if "-darwin" in target:
            env["MACOSX_DEPLOYMENT_TARGET"] = macos_compat(target)
        
        subprocess.check_call(command, env=env)

        shutil.copyfile(
            SRC_ROOT / "project-root" / "target" / target / buildvariant / "deps" / shared_object,
            SRC_ROOT / "project-root" / "library" / "bindings" / "python3" / "loibrary" / new_shared_object_name,
        )
        
        command = [
            "cargo",
            "run",
            "--features=uniffi/cli",
            "--bin",
            "uniffi-bindgen",
            "generate",
            "src/file.udl",
            "--language",
            "python",
            "--out-dir",
            SRC_ROOT / "proj" / "target",
        ]
        subprocess.check_call(command, cwd=Path("cryptatools-core"), env=env)

        shutil.copyfile(
            SRC_ROOT / "project-root" / "target" / "cryptatools.py", SRC_ROOT / "project-root" / "library" / "bindings" / "python3" / "library" / "python3_bindings.py"
        )

        return _build.run(self)

long_description = (Path(__file__).parent.parent / "README.md").read_text()
```


## setup()

```python
setup(
    author="gogo2464",
    author_email="gogo246475@gmail.com",
    classifiers=[
        "Intended Audience :: Developers",
        "Natural Language :: English",
        "Programming Language :: Python :: 3"
    ],
    description="Python Binding of the library and cryptanalysis tool 'cryptatools'.",
    long_description=long_description,
    install_requires=requirements,
    long_description_content_type="text/markdown",
    include_package_data=True,
    keywords="cryptatools",
    name="cryptatools-python3",
    version="0.1.14",
    packages=[
         "library"
    ],
    package_dir={
         "library": "library/bindings/python3/library"
    },
    setup_requires=requirements,
    url="https://github.com/gogo2464/cryptatools-rs",
    zip_safe=False,
    package_data={"library": [new_shared_object_name]},
    distclass=BinaryDistribution,
    cmdclass={"install": InstallPlatlib, "bdist_wheel": bdist_wheel, "build": build},
)
```



## COngratulation.

It was not easy but you did it! In case of issue, fell free to consul the next chapter for real life example.