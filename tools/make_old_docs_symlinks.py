# Make redirects for our old html files relative to the root to version specific content.
# See also ../docs/manual/src/README.md
#
# This is a "migration" script, so not run regularly and not by CI,
#
# We used to publish the entire site in the "root" - ie, we had/have
# `./Motiviation.html` - but we've moved to a versioned site, so this page is now
# at, eg, `./0.XX/Motiviation.html`.
# This script locates all such HTML files and replaces them with HTML which redirects
# to 0.27.
#
# Note:
# * We don't want a symlink as we want the browser to see the actual new URL.
# * We pin the redirects to 0.27 to try and be more future-proof for future doc refactors.
#   It could be run again to update to a later version.
#
# It is designed to be run on a directory with the `gh-pages` branch checked out,
# and the result is then checked in and pushed.

import argparse
import os

paths = r"""
    ./Motivation.md
    ./Getting_started.md
    ./tutorial/Prerequisites.md ./tutorial/udl_file.md ./tutorial/Rust_scaffolding.md ./tutorial/foreign_language_bindings.md
    ./udl_file_spec.md ./udl/namespace.md ./udl/builtin_types.md ./udl/enumerations.md ./udl/structs.md
    ./udl/functions.md ./udl/errors.md ./udl/interfaces.md ./udl/callback_interfaces.md ./udl/ext_types.md
    ./udl/ext_types_external.md ./udl/custom_types.md ./udl/docstrings.md
    ./proc_macro/index.md
    ./futures.md
    ./bindings.md
    ./foreign_traits.md
    ./kotlin/configuration.md ./kotlin/gradle.md ./kotlin/lifetimes.md
    ./swift/overview.md ./swift/configuration.md ./swift/module.md ./swift/xcode.md
    ./python/configuration.md
    ./internals/design_principles.md ./internals/crates.md ./internals/lifting_and_lowering.md
    ./internals/object_references.md ./internals/rendering_foreign_bindings.md"""

redirect_template = r"""
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Redirecting.</title>
  <noscript>
    <meta http-equiv="refresh" content="1; url={target}/{path}" />
  </noscript>
  <script>
    window.location.replace("{target}/{path}" + window.location.hash);
  </script>
</head>
<body>
  Redirecting to <a href="{target}/{path}">{target}/{path}</a>...
</body>
</html>
"""

def redirects(args):
    target = "0.27"
    for spec in paths.split():
        path = os.path.splitext(os.path.normpath(spec))[0] + ".html"
        fq = os.path.join(args.target, path)
        if os.path.exists(fq):
            new_content = redirect_template.format(path=path, target=target)
            open(fq, "w").write(new_content)
            print("Updated", path)
        else:
            print("html file does not exist:", path)

def main():
    parser = argparse.ArgumentParser(description='UniFFI docs redirect helper')

    subparsers = parser.add_subparsers(required=True)

    parser_redirect = subparsers.add_parser('redirects', help='check redirects etc')
    parser_redirect.set_defaults(func=redirects)
    parser_redirect.add_argument('--target', type=str, required=True)

    args = parser.parse_args()
    args.func(args)

if __name__=='__main__':
    main()
