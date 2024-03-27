# Intro

The main idea is to build the bindings with commands.

So you must create a setup.py file. This file will include the command to generate python .py bindings, link .dll and then build to pipy. Requires python version greater than 3.6.

The full example is available at this [address](https://github.com/mozilla/uniffi-rs/tree/main/examples/distributing/uniffi-rust-to-python-library/src/setup.py).

Once you reproducted the template on your project, fell free to run with:

## Windows Powershell

```powershell
python .\src\setup.py bdist_wheel ;
$wheelFile = Get-ChildItem -Path .\dist\ -Recurse -Include * ;
pip install $wheelFile --force-reinstall ;
```

## MacOs and Linux commands:

```bash
python3 ./src/setup.py bdist_wheel --verbose ;
pip3 install ./dist/* --force-reinstall ;
```