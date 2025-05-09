# How To Use Attachments with ZeWIF

The ZeWIF data format is build on [Gordian Envelope](https://developer.blockchaincommons.com/envelope/). This provides it with access to many additional features that might not be utilized in the first iteration of the ZeWIF standard but which may be of use in the future, including compression, elision, and encryption.

Another special feature of Envelope is the "attachment", which is fully detailed in [BCR-2023-006](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-006-envelope-attachment.md) and summarized here. Attachments _are_ used in ZeWIF, specifically to store content for certain data classes. Attachment points are available in many parts of the ZeWIF structure, including the top-level structure.

## Data Classes in ZeWIF

ZeWIF organizes data into three classes:

* **Class I.** Data that is stored by two or more different wallets and is considered important and up-to-date.
* **Class II.** Data that is stored by less than two wallets or that is considered less important or less up-to-date.
* **Class III.** Data that is considered unimportant, not up-to-date, and is typically not in use by multiple wallets.

Class I data is defined in the ZeWIF spec. It can be encoded by the `zewif` crate and may be marked through CBOR tags, known values, or other means. Class I data structure also included attachment points so that all data of a specific type is kept together. Class II data should still be stored as part of a ZeWIF file, but is not specified, so the storage is done through attachments. Class III data is not considered important enough to store, but should still be maintained as part of an attachment containing the entirety of the original wallet data file that is being migrated.

To clarify attachments may be found in two places in a ZeWIF data file:

* **Top-Level Attachments.** The Zewif structure includes an attachment point for top-level content. This includes a copy of the original data file. Additional data not relevant to a structure attachment might also be included here.
* **Structure Attachments.** Individual lower-level structures also include attachments points when there was thought to be reason to include additional data or metadata on the topic that nonetheless was not Class I.

After a ZeWIF data file is output, additional attachments could also be made using the [`envelope-cli-rust` tool](https://github.com/BlockchainCommons/bc-envelope-cli-rust) or other means to adjust a Gordian Envelope file.

## Envelope Technical Overview

The [Gordian Envelope](https://developer.blockchaincommons.com) is fully defined in an [IETF Internet-Draft](https://datatracker.ietf.org/doc/draft-mcnally-envelope/). Understanding of the Envelope data format is _not_ required to use ZeWIF, even if input is occurring from ZeWIF files or output is occurring to ZeWIF files. The `zewif` crates, `zmigrate` tool, and associated docs should be all that are required to make use of ZeWIF.

Nonetheless, the following major points are useful as an overview of Gordian Envelope, the data-file format that is used for an intermediary for ZeWIF transfer (or storage).

* **Envelope Uses CBOR.** Gordian Envelope is built using CBOR, a [well-specified](https://cbor.io/) and mature binary data representation format. Every Envelope is not only [legal CBOR](https://datatracker.ietf.org/doc/html/rfc8949), but also [legal dCBOR](https://datatracker.ietf.org/doc/draft-mcnally-deterministic-cbor/), a deterministic version of CBOR. Every Envelope, and therefore every ZeWIF file, can be read using CBOR tools such as [cbor.me](https://cbor.me/). (But don't read ZeWIF files containing private keys in an online site!)
* **Envelope Stores Data in a Merkle-Like Tree.** Gordian Envelope is a branching hierarchical structure. Central nodes lead to multiple branches and eventually to leaves. This allows for the organized storage of data. The tree is Merkle-like because branches can be hashed and that hash can be stored in a node to prove the data that lies under it (which may not be relevant for the first iteration of ZeWIF, but which allows for powerful elision and signatures).
* **Envelope is Built on Semantic Triples.** Data is stored in a Gordian Envelope as a semantic triple of subject-predicate-object. Each predicate-object pair is called an assertion, which applies to a subject. A node connects together a subject and zero or more assertions about that subject.

```mermaid
graph LR
    1("node")
    2["subject"]
    3["assertion"]
    4["predicate"]
    5["object"]
    6["assertion"]
    7["predicate"]
    8["object"]

    1 --> 2
    1 --> 3
    3 --> 4
    3 --> 5
    1 --> 6
    6 --> 7
    6 --> 8
```
* **Envelope Can Be Recursive.** Any subject, predicate, object, or assertion can itself be an Envelope (a semantic triple). This allows for fully recursive structures to improve organization.

```mermaid
graph LR
    1("node")
    2["subject"]
    3["assertion"]
    4["predicate"]
    5["object"]
    6["assertion"]
    7["predicate"]
    8["node"]
    9["subject"]
    10["assertion"]
    11["predicate"]
    12["object"]
    1 --> 2
    1 --> 3
    3 --> 4
    3 --> 5
    1 --> 6
    6 --> 7
    6 --> 8
    8 --> 9
    8 --> 10
    10 --> 11
    10 --> 12
```

* **Envelope Can Be Stored as a UR.** Blockchain Commons [Uniform Resources (UR) format](https://developer.blockchaincommons.com/ur/) supports the text encoding of CBOR data. It can be used to store a ZeWIF Envelope file and to encode it as a QR or an [Animated QR](https://developer.blockchaincommons.com/animated-qrs/). This can resolve issues of storage and transmission.
* **Envelopes Can Be Signed.** Envelopes can be "wrapped" and then signed. This is not currently supported by ZeWIF, but could be used in the future to guarantee the authenticity of a ZeWIF file.
* **Envelope Hashes Allow Data to Be Elided.** Some or all data in an Envelope can be elided. Hashes remain allowing for proof of content and proof of signature. This is another Envelope feature not currently supported by ZeWIF, but that might have future usefulness.

### Wrapped Envelopes

A "wrapped" envelope is an Envelope that has been turned into a subject for a new envelope. This allows assertions to refer to the entire content of the envelope.

```mermaid
graph LR
    1["node"]
    2["subject"]
    3["assertion"]
    4["predicate"]
    5["object"]
    6["assertion"]
    7["predicate"]
    8["object"]
    W1["node"]
    W2["wrapped (subject)"]
    W3["assertion"]
    W4["predicate"]
    W5["object"]

    W1 --> W2
    W2 --> 1
    W1 --> W3
    W3 --> W4
    W3 --> W5
    1 --> 2
    1 --> 3
    3 --> 4
    3 --> 5
    1 --> 6
    6 --> 7
    6 --> 8
```

This feature is a crucial element of an attachment.

## Attachments Technical Overview

Attachments are fully described in [BCR-2023-006](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-006-envelope-attachment.md). They make use of Envelope wrapping (described above) and of [known values](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-002-known-value.md), which are unique unsigned 64-bit integers that are registered with Blockchain Commons and used to refer to specific, repeating concepts in a precise, compact way; known values are encoded in CBOR with the `#6.40000` tag.

* An attachment is an assertion in an envelope, meaning that it contains a predicate and an object.
* The predicate of an attachment is known value #50, which is registered as `attachment`.
* The object of an attachment is the payload of attachment data.

When creating a payload for a ZeWIF attachment you might simply dump in the otherwise unspecified data as a binary blob, matching whatever format the data was held in in the originating wallet. It's better to simplify and standardize it as much as possible as discussed in the [ZeWIF best practices](https://github.com/BlockchainCommons/zmigrate/blob/master/docs/bestpractices.md#attachments). Because an object can be a full Envelope, it's also possible to create an entire recursive data structure as the object of an attachment. However, the `zewif` crate currently creates an attachment as simply an attachment with `vendor` and (optionally) `conformsTo` assertions:

* Once an attachment has been created, the Envelope is wrapped, turning it into the subject of a new Envelope.
* A `vendor` assertion must be added. This is a predicate of known value #51 with the object of a string for the vendor name.
* A `conformsTo` assertion may be added. (It's [highly suggested](https://github.com/BlockchainCommons/zmigrate/blob/master/docs/bestpractices.md#attachments)) for ZeWIF attachments.) This is a predicate of known value #52 with the object being a string that identifies the format of the attachment.

That's it! You have an attachment! And, if you're using the `zewif` crate you don't even need to know these specifics, you just use the `attachments.add` function:
```
attachments.add(payload,vendor,conforms_to);
```

You should create attachments for all of the data in your wallet that is not defined by the ZeWIF spec.

### Defining `vendor` and `conformsTo`

**`vendor`** must be a unique name identifying the vendor of the wallet. It's suggested as a reverse domain name, such as `org.zingolabs`.

**`conformsTo`** is not required, but it is highly recommended, especially for ZeWIF data, since it's crucial that it be understandable in the  future. It's recommended as a URL that contains the specification describing what the data is and how it's encoded. By storing this info at a URL, you can sure it's recoverable in the far future. Even if your site is gone, archive.org or like services may have a copy.

### Versioning `conformsTo`

Your `conformsTo` should be versioned so that a user or importer can look up the precise version that data was encoded with. One way to do so is to ensure that the URL referred to in `conformsTo` includes a version, with different versions going to different URLs. This will allow for the future lookup of a specific version of the `conformsTo` URL using a service such as archive.org.

### Adding Other Metadata

Other assertions can be added to the wrapped Envelope. This might include notes about the contents, warnings, or even instructions for their use to supplement the `conformsTo` information. Anything that might be helpful to later recovery of the data should be considered for addition.

[[It looks like this may not be possible in current version because `add_attachment` only allows conformsTo and vendor? Need to update and refer it for the future if so.]]

## Encoding an Attachment

Attachments are defined in `zewif/attachments.rs`.

Attachments can be added with the `attachments.add` function, which has arguments of an Envelope payload, a string `vendor`, and an optional string `conforms_to`.

[[example]]

### Encoding a Structure Attachment

Some standardized data structures in ZeWIF contain spaces for attachments. This example shows how an attachment is incorporated into the `Account` while in memory:
```
pub struct Account {
    id: ARID,
    name: String, // May not be unique.
    zip32_account_id: Option<u32>,
    addresses: HashMap<String, Address>,
    relevant_transactions: HashSet<TxId>, // Subset of the global transaction history.
    // The following are intended for storage of information that may not be
    // recoverable from the chain.
    sapling_sent_outputs: Vec<SaplingSentOutput>,
    orchard_sent_outputs: Vec<OrchardSentOutput>,
    attachments: HashMap<Digest, Envelope>,
}
```
The hash map allows for a number of attachments to be added to this standard data structure, each of which is intended to hold metadata related to the Account.

When output as ZeWIF, the attachment will then be part of that branch of the data tree.
```
[EXAMPLE OF ZeWIF OUTPUT]
```
An attachment can be added to a standard data structure as follows:
```
[example]
```

### Encoding a Top-Level Attachment

Attaching something to the top-level `Zewif` structure works just the same. As seen in [`zewif_impl.rs`](https://github.com/BlockchainCommons/zewif/blob/master/src/zewif_impl.rs), `attachments` is one of the top-level structures:
```
pub struct Zewif {
    wallets: HashMap<ARID, ZewifWallet>,
    transactions: HashMap<TxId, Transaction>,
    attachments: Attachments,
}
```

This allows for the attachment of data that doesn't fit in any particular structure, including the backup of the data file of the original wallet:
```
[example of attaching data file]
```
The biggest challenge here may actually be converting the data file into a serializable format. Though this will be trivial for a flat file, database driven wallets will likely need to dump a number of tables in a way that is meaningfully readable.
