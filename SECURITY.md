# Security policy

## Reporting a vulnerability

Please report a suspected vulnerability **privately**, not in a public issue or pull request.

Use GitHub's private vulnerability reporting for this repository: open the
[Security tab of MentalGear/shadcn-dioxus](https://github.com/MentalGear/shadcn-dioxus/security) and choose
**Report a vulnerability** (a private security advisory visible only to you and the maintainers). Include what is affected
(a component, a primitive, the CLI, a dependency, the docs site), how to reproduce it, and the version or commit.

If the **Report a vulnerability** button is not offered, private reporting is not switched on for the repository yet: open a
public issue that says only that you have a security report and asks for a private channel, and put **no details** in it.

This is a small project maintained by volunteers; we aim to acknowledge a report within a week but cannot promise a fixed
response time.

## Supported versions

There is no tagged release yet (the crates are at `dioxus-primitives` 0.0.1 and `shadcn-dioxus` 0.1.0 and are installed
from git). Security fixes go to **`main`** only. Once a release is tagged, this section will name the latest release as
supported.

## What this covers, in short

The project is a Dioxus component registry plus its primitives crate, a CLI that copies components into an app, and a docs
site. Its supply-chain defence is a set of offline gates run before every commit: lock and manifest consistency, exact pins
and version floors for the crates a user compiles, a fixed dependency set for the one component that parses untrusted text
(markdown), committed generated code checked against its source, one registry URL for every install path, no third-party
runtime fetches on the docs site, and JavaScript tooling that refuses freshly published versions and runs no install
scripts. It does **not** yet scan dependencies for known advisories, review dependency code, or control transitive build
scripts; those are planned and listed as such.

The threat model, the full controls table (what each control stops, the script that enforces it, how to adopt it in your own
project) and what is deliberately out of scope are in
[`dev-docs/supply-chain/README.md`](./dev-docs/supply-chain/README.md).
