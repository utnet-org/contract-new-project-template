# unc-new-project-name

unc-new-project-description

## Quickstart Guide

You can start coding on the Utility Rust stack in less than a minute, thanks to [Utility Devcontainers](https://github.com/utnet-org/unc-devcontainers). How?

1. Click **Use this template** > **Create a new repository**

<img width="750" alt="Create a new repository" src="https://unc-s3.jongun2038.win/template.png">

2. In your newly created repo, click **Code** > **Codespaces** > **Create codespace on main**

<img width="750" alt="Create Codespace" src="https://unc-s3.jongun2038.win/new_project.png">

## Where to Get Started?

Start writing your contract logic in [src/lib.rs](src/lib.rs) and integration tests in [tests/test_basics.rs](tests/test_basics.rs).

## How to Build Locally?

Install [`unc`](https://github.com/utnet-org/utility-cli-rs) and run:

```bash
unc devtool build
```

## How to Test Locally?

```bash
cargo test
```

## How to Deploy?

Deployment is automated with GitHub Actions CI/CD pipeline.
To deploy manually, install [`unc`](https://github.com/utnet-org/utility-cli-rs) and run:

```bash
unc devtool deploy <account-id>
```

## Useful Links

- [unc CLI](https://unc.cli.rs) - Iteract with Utility blockchain from command line
