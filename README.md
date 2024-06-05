# contract-new-project-name

## contracts template

- [Hello / OnBoarding contract](./hello-utility/)
- [Drop / Base contract](./airdrop/)
- [Social / Web3 contract](./social-web3/)
- [Lockup / Vesting contract](./lockup/)
- [Lockup Factory](./lockup-factory/)
- [Multisig contract](./multisig/)
- [Multisig Factory](./multisig-factory/)
- [Staking Pool / Delegation contract](./staking-pool/)
- [Staking Pool Factory](./staking-pool-factory/)
- [Voting Contract](./voting/)
- [Whitelist Contract](./whitelist/)

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
# use root account if dev-containers environment
sudo su -
cd <contracts template> # cd social-web3

unc dev-tool build
```

## How to Test Locally?

```bash
cargo nextest run --nocapture  # cargo test
```

## How to Deploy?

Deployment is automated with GitHub Actions CI/CD pipeline.
To deploy manually, install [`unc`](https://github.com/utnet-org/utility-cli-rs) and run:

```bash
unc dev-tool deploy <account-id>
```

## Useful Links

- [unc CLI](https://unc.cli.rs) - Iteract with Utility blockchain from command line
