# The docs for the "manual".

The manual is what we publish to https://mozilla.github.io/uniffi-rs/ and
is generated from the content in this directory.

This README is not published as part of the docs - but almost everything
else in this directory is. This document describes how that publishing
works.

* This repo uses `mkdocs` - but **we must never deploy using that tool**.
  Deploying using mkdocs will publish into the root of the site, **damaging
  the versioning system we have in place.**

* We use a [`mike`](https://github.com/jimporter/mike) to manage
  the `mkdocs` build process and the deployment - it deploys to a versioned
  (eg, `./0.27`) directory and manages aliases (eg, `latest`) on the site.

* There are various HTML redirects and symlinks keeping things together. These
  are maintained by hand. See also any docs related scripts in `./tools`.

To put it another way: we never deploy the entire site, but instead use `mike`
to deploy just a single version.

## How to deploy a specific version.

While the main branch is published as a version called `next` by CI, the
steps to publish a specific version must be performed manually for now.

* Ensure you have the release branch checked out.

* Prepare your Python environment by executing:
> pip install -r tools/requirements_docs.txt

* From the root of your checkout, execute:
> mike deploy 0.XX latest --update-aliases --push

If you leave out `--push` your local `gh-pages` branch will be changed but it will not
be pushed, which can be helpful if you want to check the result. You can later push
the branch normally.

`--update-aliases` is only strictly necessary if this is the first time a
new version has been pushed and you want it to be the new "latest" version.

## Layout of the published site
The layout of the published pages is:

* `./versions.json` is a file maintained by `mike` and used in the generated
  html as a version selector.

* `./next` is the directory where `mike` publishes the docs on currrent git `main`.

* `./0.27`, `./0.28` etc are directories where `mike` publishes released versions.

* `./latest` is a symlink managed by `mike` which points to the most recently released version.

The other content is managed manually by the UniFFI maintainers by directly editing the `gh-pages` branch:

* Other `*.html` files are the entry-points to our docs which existed before we versioned the docs.
  They have been replaced with redirects to `0.27` - ie, `./Motivation.html` is content which redirects to `0.27/Motivation.html`.

* `./index.html` is a little shim that redirects to `./latest`.
