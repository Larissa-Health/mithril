---
sidebar_position: 4
---

# Mithril client library

:::info

Mithril client library can be used by Rust developers to use the Mithril network in their applications.

It is responsible for handling the different types of data certified by Mithril and available through a Mithril aggregator:

- [**Cardano database**](../../../glossary.md#cardano-database): list, get, download the tarball, and record statistics
- [**Mithril stake distribution**](../../../glossary.md#stake-distribution): list and get
- [**Cardano transaction**](../../../glossary.md#cardano-transaction): list and get snapshots, get proofs
- [**Cardano stake distribution**](../../../glossary.md#stake-distribution): list, get, and get by epoch
- [**Certificate**](../../../glossary.md#certificate): list, get, and chain validation.

:::

:::tip

- For more information about the **Mithril network**, please see the [architecture](../../../mithril/advanced/mithril-network/architecture.md) overview

- For more information about the **Mithril client** node, please see [this overview](../../../mithril/advanced/mithril-network/client.md)

- Check out the [`Bootstrap a Cardano node`](../../getting-started/bootstrap-cardano-node.md) guide.

:::

:::info

The Mithril network configurations are available in the [**Network configurations**](../../getting-started/network-configurations.md) section of the user manual.

:::

## Resources

|        Node        |                                     Source repository                                      |                                Rust documentation                                 |                         Network configurations                         |
| :----------------: | :----------------------------------------------------------------------------------------: | :-------------------------------------------------------------------------------: | :--------------------------------------------------------------------: |
| **Mithril client** | [:arrow_upper_right:](https://github.com/input-output-hk/mithril/tree/main/mithril-client) | [:arrow_upper_right:](https://mithril.network/rust-doc/mithril_client/index.html) | [:arrow_upper_right:](../../getting-started/network-configurations.md) |

## Prerequisites

- Install the latest stable version of the [correctly configured](https://www.rust-lang.org/learn/get-started) Rust toolchain

- Install build tools `build-essential` and `m4`; for example, on Ubuntu/Debian/Mint, run `sudo apt install build-essential m4`

- Install OpenSSL development libraries; for example, on Ubuntu/Debian/Mint, run `apt install libssl-dev`.

## Installation

In your project, use `cargo` to add [mithril-client](https://crates.io/crates/mithril-client) crate as a dependency:

```bash
cargo add mithril-client
```

:::info

Mithril client is an asynchronous library. You will need a runtime to execute your futures. We recommend using the crate [tokio](https://crates.io/crates/tokio), as the library has been tested with it.

:::

## Using the Mithril client library

Below is a basic example of how to use most of the functions exposed by the Mithril client library:

```rust title="/src/main.rs"
use mithril_client::{ClientBuilder, MessageBuilder};
use std::path::Path;

#[tokio::main]
async fn main() -> mithril_client::MithrilResult<()> {
    let client = ClientBuilder::aggregator("YOUR_AGGREGATOR_ENDPOINT", "YOUR_GENESIS_VERIFICATION_KEY").build()?;

    let snapshots = client.snapshot().list().await?;

    let last_digest = snapshots.first().unwrap().digest.as_ref();
    let snapshot = client.snapshot().get(last_digest).await?.unwrap();

    let certificate = client
        .certificate()
        .verify_chain(&snapshot.certificate_hash)
        .await?;

    // Note: the directory must already exist, and the user running this code must have read/write access to it.
    let target_directory = Path::new("YOUR_TARGET_DIRECTORY");
    client
        .snapshot()
        .download_unpack(&snapshot, target_directory)
        .await?;

    if let Err(e) = client.snapshot().add_statistics(&snapshot).await {
        println!("Could not increment snapshot download statistics: {:?}", e);
    }

    let message = MessageBuilder::new()
        .compute_snapshot_message(&certificate, target_directory)
        .await?;
    assert!(certificate.match_message(&message));

    Ok(())
}
```

:::info

Snapshot download and certificate chain validation can take quite some time, even with a fast computer and network. We have implemented a feedback mechanism for them; more details are available in the [feedback sub-module](https://mithril.network/rust-doc/mithril_client/feedback/index.html).

An example of implementation with the crate [indicatif](https://crates.io/crates/indicatif) is available in the [Mithril repository](https://github.com/input-output-hk/mithril/tree/main/examples/client-snapshot/src/main.rs). To run it, execute the following command:

```bash
cargo run -p client-snapshot
```

or directly from the example crate directory:

```bash
cargo run
```

:::

Here is a working example of the code using the configuration parameters of the `release-preprod` network:

```rust title="/src/main.rs"
use mithril_client::{ClientBuilder, MessageBuilder};
use std::path::Path;

#[tokio::main]
async fn main() -> mithril_client::MithrilResult<()> {
    let client = ClientBuilder::aggregator("https://aggregator.release-preprod.api.mithril.network/aggregator", "5b3132372c37332c3132342c3136312c362c3133372c3133312c3231332c3230372c3131372c3139382c38352c3137362c3139392c3136322c3234312c36382c3132332c3131392c3134352c31332c3233322c3234332c34392c3232392c322c3234392c3230352c3230352c33392c3233352c34345d").build()?;

    let snapshots = client.snapshot().list().await?;

    let last_digest = snapshots.first().unwrap().digest.as_ref();
    let snapshot = client.snapshot().get(last_digest).await?.unwrap();

    let certificate = client
        .certificate()
        .verify_chain(&snapshot.certificate_hash)
        .await?;

    // Note: the directory must already exist, and the user running this code must have read/write access to it.
    let target_directory = Path::new(".");
    client
        .snapshot()
        .download_unpack(&snapshot, target_directory)
        .await?;

    if let Err(e) = client.snapshot().add_statistics(&snapshot).await {
        println!("Could not increment snapshot download statistics: {:?}", e);
    }

    let message = MessageBuilder::new()
        .compute_snapshot_message(&certificate, target_directory)
        .await?;
    assert!(certificate.match_message(&message));

    Ok(())
}
```

:::info

The library now also allows the download of an incremental Cardano database snapshot, enabling the retrieval of a **specific range of immutable files**. This feature is still unstable.

To use it, you need to add the `unstable` feature in your project's `Cargo.toml` file.

```
mithril-client = { version = "0.11.X", features = ["fs", "unstable"] }
```

:::

Below is a basic example using the new `CardanoDatabase` functions. Make sure the target aggregator signs `CardanoDatabase` incremental snapshot.

:::tip

You can verify that the aggregator signs **CardanoDatabase** by running the command below:

```bash
wget -q -O - YOUR_AGGREGATOR_ENDPOINT | jq '.capabilities.signed_entity_types | contains(["CardanoDatabase"])'
```

For example, with the aggregator on the `testing-preview` Mithril network:

```bash
wget -q -O - https://aggregator.testing-preview.api.mithril.network/aggregator | jq '.capabilities.signed_entity_types | contains(["CardanoDatabase"])'
```

:::

```rust title="/src/main.rs"
use mithril_client::{ClientBuilder, MessageBuilder, DownloadUnpackOptions, ImmutableFileRange};
use std::path::Path;

#[tokio::main]
async fn main() -> mithril_client::MithrilResult<()> {
    let client = ClientBuilder::aggregator("YOUR_AGGREGATOR_ENDPOINT", "YOUR_GENESIS_VERIFICATION_KEY").build()?;

    let snapshots = client.cardano_database_v2().list().await?;

    let latest_hash = snapshots.first().unwrap().digest.as_ref();
    let snapshot = client.cardano_database_v2().get(latest_hash).await?.unwrap();

    let certificate = client
        .certificate()
        .verify_chain(&snapshot.certificate_hash)
        .await?;

    let immutable_file_range = ImmutableFileRange::From(15000);
    let download_unpack_options = DownloadUnpackOptions {
        allow_override: true,
        include_ancillary: false,
        ..DownloadUnpackOptions::default()
    };

    // Note: the directory must already exist, and the user running this code must have read/write access to it.
    let target_directory = Path::new("YOUR_TARGET_DIRECTORY");
    client
        .cardano_database_v2()
        .download_unpack(
            &snapshot,
            &immutable_file_range,
            &target_directory,
            download_unpack_options,
        )
        .await?;

    let merkle_proof = client
        .cardano_database_v2()
        .compute_merkle_proof(
            &certificate,
            &snapshot,
            &immutable_file_range,
            &target_directory,
        )
        .await?;
    merkle_proof
        .verify()?;


    let message = MessageBuilder::new()
        .compute_cardano_database_message(&certificate, &merkle_proof)
    assert!(certificate.match_message(&message));

    Ok(())
}
```

:::info

An full example of implementation is available in the [Mithril repository](https://github.com/input-output-hk/mithril/tree/main/examples/client-cardano-database/src/main.rs). To run it, execute the following command:

```bash
cargo run -p client-cardano-database
```

or directly from the example crate directory:

```bash
cargo run
```

:::

:::tip

You can read the complete [developer documentation](https://mithril.network/rust-doc/mithril_client/index.html).

:::
