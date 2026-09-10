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

## Verification status: still unresolved as of v0.1.1

`v0.1.0`'s deployed contract (`CADWLKFTNAILABOEX2LH3XGPELA3JSX2R3HVJD4DVLSE5KJNSKSMKQUX`,
`unverified/verify-build-demo` on testnet) never verified. What's confirmed:
wasm hash matches exactly across our build, the deployed contract, and what
Stellar Expert itself reports; the wasm's `meta` correctly carries
`source_repo`; `stellar contract info build` shows a valid GitHub attestation
(correct repo/commit/workflow); and the `contract-validation/match` POST was
submitted 3 times over 39+ hours (`public` and `testnet` segments both) and
always got `200 {}` back. `v0.1.1` added a committed `Cargo.lock`, in case
Stellar Expert's matching does an independent reproducible rebuild rather
than trusting the attestation — plausible given the known-verified
`blend-contracts-v2` repo commits one — but that's unconfirmed; Stellar
Expert's own docs on the exact mechanism are thin/contradictory, and 16+
hours after that submission it's still `unverified`. Treat the Cargo.lock fix
as a reasonable guess, not a confirmed root cause.

Useful inspection commands, run against a downloaded release asset:
```bash
stellar contract info meta --wasm <path>    # embedded rsver/rssdkver/cliver/source_repo
stellar contract info build --wasm <path>   # looks up the GitHub attestation for the hash
stellar contract info hash --wasm <path>    # the wasm's own hash, to cross-check against
                                             # GET https://api.stellar.expert/explorer/testnet/contract/{id}
```

## `v0.2.0`: a contract that can actually be upgraded

`v0.1.x` had no `upgrade` entrypoint, so it could never be upgraded in place
— `stellar registry upgrade` works by calling the *deployed contract's own*
`upgrade` function, and there wasn't one. `v0.2.0` adds a minimal
admin-gated `upgrade` (plus `admin()`, which `upgrade_contract` looks for to
require the admin's auth at the top level) and a trivial `goodbye` function,
so it's both a fresh wasm hash to test verification against and a contract
that supports real upgrades from here on.

Because the existing `unverified/verify-build-demo` name is already taken by
the un-upgradable `v0.1.x` contract, `v0.2.0` deploys under a **new** contract
name, `unverified/verify-build-demo-2`, as its own instance with its own
constructor (`--admin=<address>`).

## Publish & deploy (testnet)

The `soroban-build-workflow` release job names its GitHub Release
`<tag>_<package>_cli<version>`, not the bare tag — check
[the releases page](https://github.com/stellar-registry/verify-build-demo/releases)
for the exact name (e.g. `v0.2.0_verify-build-demo_cli25.1.0`).

```bash
# download the exact CI-built wasm — publish this one, not a local rebuild,
# so its hash matches what was already submitted to Stellar Expert
gh release download v0.2.0_verify-build-demo_cli25.1.0 \
  -R stellar-registry/verify-build-demo -p '*.wasm' -D /tmp

stellar network use testnet
stellar keys use <your-funded-testnet-identity>

stellar registry publish \
  --wasm /tmp/verify-build-demo_v0.2.0.wasm \
  --wasm-name unverified/verify-build-demo \
  --binver 0.2.0

stellar registry deploy \
  --contract-name unverified/verify-build-demo-2 \
  --wasm-name unverified/verify-build-demo \
  --version 0.2.0 \
  -- \
  --admin <YOUR_G_ADDRESS>
```

From here, future versions can be published and rolled out with a real
upgrade (no new contract name needed):
```bash
stellar registry upgrade \
  --contract-name unverified/verify-build-demo-2 \
  --wasm-name unverified/verify-build-demo \
  --version <new-version>
```

Verification isn't instant — Stellar Expert has to index the on-chain deploy
and match it against the hash submitted in the release workflow, and the
registry indexer has to poll Stellar Expert after the `register` event, so
expect some lag before checking
`GET https://stellar-registry-testnet.fly.dev/v1/contracts/unverified/verify-build-demo-2`.
