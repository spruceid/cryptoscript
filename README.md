# cryptoscript

cryptoscript is a non-turing complete DSL used for policy-as-code. It will
typically be embedded within an authorization capability as the primary caveat
representing the predicate conditions required for some action. It is currently
highly experimental and syntax/semantics are subject to change.

## Use Cases and Examples

For example, the following cryptoscript function can be used to control access
to a resource based on the ownership of an arbitrary Non-Fungible Token (NFT)
and countersigned capability:
```
define only_nft_owner(nft_addr, nft_idx) {
    let n = nft.resolve(nft_addr, nft_idx);
    n.push_owner_address;
    push_countersigner_address;
    assert_equal;
}
```

The following cryptoscript block is used for authentication that employment,
department, and access level represented as a W3C Verifiable Credential is
sufficient to proceed with an action.
```
let vp = pop<vc::W3CVerifiablePresentation>;
push_countersigner_address;
push vp.signer;
assert_equal;
let vc = vp.vc[0];
assert_elem vc.issuer [
    did:pkh:eip155:0xb9c5714089478a327f09197987f16f9e5d936e8a,
    did:ion:EiD3DIbDgBCajj2zCkE48x74FKTV9_Dcu1u_imzZddDKfg,
    did:web:credentials.corp.com,
];
assert_equal vc.department<str> "engineering";
assert_gte vc.access_level<uint64> 2;
```

cryptoscript can be mixed and matched. Scope and closures can be used to
further refine constrainted environments with attenuated permissions, similar
to the use of chroot in UNIX. It can also be used to explicit implement a
[verification method](https://www.w3.org/TR/did-core/#verification-methods),
including the set of cryptographic operations against data, to ensure there is
no ambiguity in contrast to specifying a data model alone.

## Design Goals
- Simplicity. Towards writing secure software, it's useful to reduce any
  unnecessary complexity. This results in a smaller implementation and attack
  surface area.
- Human-readability. Code-literate humans should be able to make sense of
  cryptoscript without much effort for ease of debug and review.
- Extensibility. Other systems will need to be integrated to maximize utility
  across a variety of environments, including the reading of blockchain data
  across different blockchain architectures, W3C Verifiable Credentials, X.509
  infrastructure, the variety of cryptosystems supported across different DID
  methods, in-production systems such as SAML2 and OIDC, JWTs, macaroons,
  biscuits, and more.
- Composability. Although cryptosript is meant to be non-turing complete, it
  should still incorporate advanced language features to improve the semantics
  of the language to prevent errors and lower attack surface area.
  Sophisticated type systems that do not get in the way should be incorporated
  to prevent entire classes of errors. Closures and S-expressions might be used
  to reduce permissioning scope, and a delegated capability could be
  implemented as a signed outer codeblock with bring-your-own-block semantics
  for custom functionality with attenuated permissions.

## Use with Capabilities

Due to the vast scale and variety of user data, we cannot specify all possible
permissions upfront with traditional approaches like RBAC. Capability-based
permission models are far more flexible and work like a hall pass. Users can
define new custom permissions with cryptoscript, and authorize with their keys.

There are infinite permutations of the possible ways to permission access with
on-chain and off-chain data, impossible to specify a priori. We introduce a
non-turing complete DSL inspired by bitcoin script called cryptoscript, where
the presenter "clears" a puzzle to get authorized. cryptoscript is extensible
with modules to support all blockchain networks and off-chain data. This
policy-as-code primitive can create infinite matching representations.

## Demo

There are two demos:
- Local: this demo is self-contained to not require any API keys
- Etherscan: this demo requires a free Etherscan API key, which you can get
  [here](https://docs.etherscan.io/getting-started/viewing-api-usage-statistics)

### Building

To build the `rest-api`:

```bash
cargo b --bin rest-api --features build-bin
```

To build the cryptoscript interpreter:

```bash
cargo b --bin cryptoscript --features build-bin
```

Building for the WASM target:

```bash
wasm-pack build --target web --no-default-features --features build-wasm
```

### Local Demo

The local demo requires running a tiny test server, which can be started with the following command:

```bash
cargo run --bin rest-api
```

Note: this API accepts PUT's of new GET "API's" for testing: each requires a
fixed `application/json` request body and returns a fixed `application/json`
response.

To run the demo itself, run:

```bash
cargo r --bin cryptoscript -- \
  --code examples/local_demo_code.json \
  --cache-location examples/local_cache.json \
  --input examples/input.json \
  --queries examples/local_query.json \
  --variables '{
    "contractaddress": "0x57d90b64a1a57749b0f932f1a3395792e12e7055",
    "address": "0xe04f27eb70e025b78871a2ad7eabe85e61212761",
    "apikey": "DUMMY_ETHERSCAN_API_KEY" }'
```

You'll see `successful!` if it completes without any errors.

### Etherscan Demo

*NOTE: this demo currently ignores any errors from Etherscan.*

This demo requires a free Etherscan API key, which you can get
[here](https://docs.etherscan.io/getting-started/viewing-api-usage-statistics)

Once you have an API key, replace `YOUR_ETHERSCAN_API_KEY` below with your API
key from Etherscan to run the demo:

```bash
cargo r --bin cryptoscript -- \
  --code examples/demo_code.json \
  --cache-location examples/cache.json \
  --input examples/input.json \
  --queries examples/query.json \
  --variables '{
    "contractaddress": "0x57d90b64a1a57749b0f932f1a3395792e12e7055",
    "address": "0xe04f27eb70e025b78871a2ad7eabe85e61212761",
    "apikey": "YOUR_ETHERSCAN_API_KEY" }'
```

### Troubleshooting Demo's

If you have any issues, make sure to clear any `cache.json` files to ensure
you're receiving fresh query responses.

