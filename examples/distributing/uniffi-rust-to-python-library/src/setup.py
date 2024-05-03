from setuptools import setup, Distribution, find_packages
from setuptools.command.install import install

from distutils.command.build import build as _build
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
import wheel.bdist_wheel

sys.dont_write_bytecode = True

if sys.version_info < (3, 6):
    print("the example project requires at least Python 3.6", file=sys.stderr)
    sys.exit(1)

from pathlib import Path  # noqa

# Path to the directory containing this file
PYTHON_ROOT = Path(__file__).parent.absolute()

# Relative path to this directory from cwd.
FROM_TOP = PYTHON_ROOT.relative_to(Path.cwd())

# Path to the root of the git checkout
SRC_ROOT = PYTHON_ROOT.parents[1]

requirements = [
    "wheel",
    "setuptools",
]

buildvariant = "release"


class BinaryDistribution(Distribution):
    def is_pure(self):
        return False

    def has_ext_modules(self):
        return True


def macos_compat(target):
    if target.startswith("aarch64-"):
        return "11.0"
    return "10.7"


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


class InstallPlatlib(install):
    def finalize_options(self):
        install.finalize_options(self)
        if self.distribution.has_ext_modules():
            self.install_lib = self.install_platlib


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


target = get_rustc_info()["host"]

extension = "" 
file_start = ""
if "-darwin" in target:
    shared_object = "libmath.dylib"
    extension = ".dylib"
    file_start = "lib"
elif "-windows" in target:
    shared_object = "mymath.dll"
    extension = ".dll"
    file_start = ""
else:
    # Anything else must be an ELF platform - Linux, *BSD, Solaris/illumos
    shared_object = "libmath.so"
    extension = ".so"
    file_start = "lib"

new_shared_object_name = file_start + "uniffi_mymath" + extension


class build(_build):
    def run(self):
        try:
            # Use `check_output` to suppress output
            subprocess.check_output(["cargo"])
        except subprocess.CalledProcessError:
            print("Install Rust and Cargo through Rustup: https://rustup.rs/.")
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
            #"--package",
            #"math",
            "--target-dir",
            "out",
            "--target",
            target,
        ]

        if buildvariant != "debug":
            command.append(f"--{buildvariant}")

        if "-darwin" in target:
            env["MACOSX_DEPLOYMENT_TARGET"] = macos_compat(target)
        
        subprocess.check_call(command, env=env)
        
        #os.makedirs(os.path.dirname(SRC_ROOT / "uniffi-rust-to-python-library" / "out"))#, exist_ok=True
        
        #print("root: {0}".format(SRC_ROOT))
        
        #print([name for name in os.listdir(".") if os.path.isdir(name) and "target" in os.path.isdir(name)][0])
        
        
        print("{0}".format(SRC_ROOT / "out" / target / buildvariant / "deps" / shared_object))
        
        
        shutil.copyfile(
            SRC_ROOT / "uniffi-rust-to-python-library" / "out" / target / buildvariant / "deps" / shared_object, #SRC_ROOT / "uniffi-rust-to-python-library" / "target" / target / buildvariant / "deps" / shared_object,
            SRC_ROOT / "uniffi-rust-to-python-library" / "out" / new_shared_object_name,
        )
        
        command = [
            "cargo",
            "run",
            "--features=uniffi/cli",
            "--bin",
            "uniffi-bindgen",
            "generate",
            "src/math.udl",
            "--language",
            "python",
            "--out-dir",
            SRC_ROOT / "uniffi-rust-to-python-library" / "target",
        ]
        
        subprocess.check_call(command, env=env)

        shutil.copyfile(
            SRC_ROOT / "uniffi-rust-to-python-library" / "target" / "mymath.py", SRC_ROOT / "uniffi-rust-to-python-library" / "out" / "mymath.py"
        )

        return _build.run(self)

setup(
    author="gogo2464",
    author_email="gogo246475@gmail.com",
    classifiers=[
        "Intended Audience :: Developers",
        "Natural Language :: English",
        "Programming Language :: Python :: 3"
    ],
    description="Example project in order to complete a uniffi-rs tutorial.",
    long_description="example",
    install_requires=requirements,
    long_description_content_type="text/markdown",
    include_package_data=True,
    keywords="example",
    name="mymath",
    version="0.1.0",
    packages=[
         "mymath"
    ],
    package_dir={
         "mymath": "out"
    },
    setup_requires=requirements,
    url="no_url",
    zip_safe=False,
    package_data={"mymath": [new_shared_object_name]},
    distclass=BinaryDistribution,
    cmdclass={"install": InstallPlatlib, "bdist_wheel": bdist_wheel, "build": build},
)
