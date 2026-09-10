# verify-build-demo

A disposable, minimal Soroban contract used to exercise the
[stellar-registry](https://github.com/stellar-registry) wasm-verified-build
feature end to end: build → attest → publish → deploy → verify.

## How it works

1. Pushing a `v*` tag triggers [`.github/workflows/release.yml`](.github/workflows/release.yml),
   which runs Stellar Expert's `soroban-build-workflow`: builds the contract,
   attaches the `.wasm` to a GitHub Release, and submits
   `{repository, commitHash, contractHash, packageName}` to Stellar Expert so
   it can later recognize a deployed contract with this wasm hash as coming
   from this repo/commit.
2. Download that exact release asset and `stellar registry publish` +
   `deploy` it (testnet, `unverified/` channel — see below). `deploy` (not
   `register-contract`) is required — it's the only op that emits the
   `Deploy` event the indexer's verified-build lookup depends on.
3. The registry indexer picks up the resulting `register` event, asks
   Stellar Expert whether the contract is verified, and shows a "Verified
   Build" badge once it is.

`Cargo.lock` is committed deliberately (not gitignored) so a rebuild
resolves the same dependency graph CI used.

## Build

A local build is for development only — its hash won't match what CI
submitted to Stellar Expert, so don't publish it:

```bash
cargo test --workspace
stellar contract build
```

The wasm you actually publish must come from a tagged CI release (below).

## Cut a release

```bash
git tag vX.Y.Z && git push origin vX.Y.Z
```

CI publishes a GitHub Release named `<tag>_verify-build-demo_cli<cli-version>`
(check the [releases page](https://github.com/stellar-registry/verify-build-demo/releases)
for the exact name) with the `.wasm` attached.

## Publish & deploy (testnet)

```bash
gh release download <release-name> -R stellar-registry/verify-build-demo -p '*.wasm' -D /tmp

stellar network use testnet
stellar keys use <your-funded-testnet-identity>

stellar registry publish \
  --wasm /tmp/verify-build-demo_vX.Y.Z.wasm \
  --wasm-name unverified/verify-build-demo \
  --binver X.Y.Z
```

Only the original author of `unverified/verify-build-demo` can publish new
versions under that name.

First deploy of a contract name needs `--admin` (constructor arg; also who
can call `upgrade` later):

```bash
stellar registry deploy \
  --contract-name unverified/<contract-name> \
  --wasm-name unverified/verify-build-demo \
  --version X.Y.Z \
  -- \
  --admin <G_ADDRESS>
```

Later versions on an already-deployed instance use `upgrade` instead (no new
contract name):

```bash
stellar registry upgrade \
  --contract-name unverified/<contract-name> \
  --wasm-name unverified/verify-build-demo \
  --version X.Y.Z
```

## Checking verification

Not instant — Stellar Expert has to index and match the deploy, then the
registry indexer polls Stellar Expert after the `register` event.

```bash
curl -s https://stellar-registry-testnet.fly.dev/v1/contracts/unverified/<contract-name>

stellar contract info meta --wasm <path>    # embedded rsver/rssdkver/cliver/source_repo
stellar contract info build --wasm <path>   # GitHub attestation for the hash
stellar contract info hash --wasm <path>    # cross-check against the API's reported hash
```

## Current deployments

| Versions | Contract name | Notes |
|---|---|---|
| v0.1.x | `unverified/verify-build-demo` | No `upgrade` entrypoint — stuck permanently at whatever version is deployed. Verification was never confirmed working. |
| v0.2.0+ | `unverified/verify-build-demo-2` | Has `admin()`/`upgrade()` — upgradeable via `stellar registry upgrade` going forward. |
