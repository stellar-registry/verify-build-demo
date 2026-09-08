# verify-build-demo

A disposable, minimal Soroban contract used to exercise the
[stellar-registry](https://github.com/stellar-registry) wasm-verified-build
feature end to end. Not a real product — just enough contract to publish,
deploy, and get verified by Stellar Expert.

## How this fits together

1. Pushing a `v*` tag triggers [`.github/workflows/release.yml`](.github/workflows/release.yml),
   which calls Stellar Expert's [`soroban-build-workflow`](https://github.com/stellar-expert/soroban-build-workflow).
   That workflow builds the contract, attaches the `.wasm` to a GitHub
   Release, and submits `{repository, commitHash, contractHash, packageName}`
   to Stellar Expert's `contract-validation/match` endpoint — this is what
   lets Stellar Expert later recognize a deployed contract with this exact
   wasm hash as coming from this repo/commit.
2. Someone downloads that exact release asset and publishes + deploys it to
   the stellar-registry (testnet, `unverified/` channel — see below). `deploy`
   (not `register-contract`) is required, since it's the only registry
   operation that emits an on-chain `Deploy` event, which the indexer's
   `v1.versions` view — and so the wasm-detail-page verified-build lookup —
   is keyed on.
3. The registry indexer picks up the resulting `register` event, asks
   Stellar Expert whether that contract_id is verified, and once it says
   yes, both the contract detail page and (pending `stellar-registry/ui#38`)
   the wasm detail page show a "Verified Build" badge.

## Publish & deploy (testnet)

```bash
# download the exact CI-built wasm — publish this one, not a local rebuild,
# so its hash matches what was already submitted to Stellar Expert
gh release download v0.1.0 -R stellar-registry/verify-build-demo -p '*.wasm' -D /tmp

stellar network use testnet
stellar keys use <your-funded-testnet-identity>

stellar registry publish \
  --wasm /tmp/verify_build_demo_v0.1.0.wasm \
  --wasm-name unverified/verify-build-demo \
  --binver 0.1.0

stellar registry deploy \
  --contract-name unverified/verify-build-demo \
  --wasm-name unverified/verify-build-demo \
  --version 0.1.0
```

Verification isn't instant — Stellar Expert has to index the on-chain deploy
and match it against the hash submitted in step 1, and the registry indexer
has to poll Stellar Expert after the `register` event, so expect some lag.
