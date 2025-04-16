# ZCash Wallet Export/Import Format (ZeWIF) Guidelines

## Project Overview

This project provides a universal format for migrating wallet data between different ZCash wallet implementations:

- `zewif`: Core library defining the ZeWIF interchange format
- `zmigrate`: Command line tool for wallet migrations
- `zewif-zcashd`: ZCashd-specific integration for migration
- `zewif-zingo`: Zingo-specific integration for migration (future)

🚨 NOTE: Only make changes to these four crates and no others in the workspace.

The ZeWIF format is designed for **data interchange**, not operational wallet functionality:

- **Purpose**: Enable migration between different ZCash wallet implementations
- **Focus**: Data preservation and format conversion
- **Approach**: Uses optional fields to handle differences between wallet implementations

## Development Environment

### Build/Test Commands

```bash
# Build a specific crate
cargo build -p <crate_name>

# Run the zmigrate tool
cargo run -p zmigrate -- zcashd zmigrate/tests/fixtures/zcashd/wallet0.dat

# Run tests
cargo test -p <crate_name>
cargo test -p <crate_name> --test <test_name>

# Check code quality
cargo clippy -p <crate_name> -- -D warnings

# Build documentation
cargo doc -p zewif --no-deps --target-dir zewif/cargo-docs
```

### Development Guidelines

- **Use accessors, not pub fields** - All struct fields should be private with accessor methods
- **Production quality** - Write code as you would for a real-world implementation
- **Proper error handling** - Use `Result<T>` with `anyhow::Context` for all functions that can fail
- **Clean code** - Fix all compiler errors and Clippy lints

### Testing

- Add tests to `zmigrate/tests` directory
- Test fixtures are stored in `zmigrate/tests/fixtures/`
- Don't mark tasks as complete until all tests pass

## Core Reference: ZeWIF Types and Concepts

### Key Data Types

| Type          | Description                                               |
| ------------- | --------------------------------------------------------- |
| `Zewif`       | Top-level container for wallet data                       |
| `ZewifWallet` | Complete wallet representation                            |
| `Blob<N>`     | Fixed-size byte array (wrapper around `[u8; N]`)          |
| `Data`        | Variable-size byte array (wrapper around `Vec<u8>`)       |
| `u256`        | 256-bit unsigned integer, little-endian byte order        |
| `u252`        | 252-bit unsigned integer for Orchard note commitments     |
| `u160`        | 160-bit unsigned integer for transparent addresses        |
| `TxId`        | Transaction ID (32 bytes representing a transaction hash) |

### Useful Macros

| Macro | Description |
|-------|-------------|
| `blob!` | Creates a new type wrapping a fixed-size byte array (`Blob<N>`) with common methods and trait implementations (Parse, Debug, etc.) |
| `data!` | Creates a new type wrapping a variable-size byte array (`Data`) with common methods and trait implementations |
| `string!` | Creates a new type wrapping a String with common methods and trait implementations for string handling |
| `mod_use!` | Simplifies module imports by both declaring a module and re-exporting its contents in a single statement |
| `impl_attachable!` | Automatically implements the Attachable trait for a type that contains an `attachments` field |
| `parse!` | Provides context-aware parsing for types that implement the `Parse` trait with error messages for various parser operations |

### Other Important Types

| Type  | Description |
| ----- | ----------- |
| `bc_components::ARID` | Apparently Random Identifier (ARID), used where structs themselves aren't simple or unique enough to be uniquely identifying. |

### Reference Materials

- Original `zcashd` code: `reference-code/zcash-master` directory
- Transaction assignment details: `zewif-zcashd/docs/TransactionAssignment.md`
- Key preservation strategy: `zewif-zcashd/docs/KeyPreservation.md`

## Current Status and Roadmap

### 🟢 COMPLETED: ZCashd to ZeWIF Migration

We have successfully implemented the complete migration pathway from ZCashd wallet.dat files to the ZeWIF format. All critical data is now properly preserved:

- **Address Migration**: All address types (transparent, sapling, unified) with metadata
- **Transaction Migration**: Complete transaction data with inputs, outputs, and witness data
- **Key Preservation**: All key materials are properly extracted and preserved
- **Metadata Handling**: Purpose strings, names, and other metadata are preserved
- **Unified Address Support**: Diversifier indices and receiver types preserved
- **Encrypted Memos**: Ciphertexts preserved without decryption (by design)
- **Account Structure**: Multi-account wallet structure correctly maintained

#### Key Design Insights

- **Unified Address Handling**: Unified addresses are not stored directly in wallet.dat. Instead, we preserve the metadata (diversifier indices, receiver types, key references) needed to derive them.
- **Encrypted Memos**: Memos are kept in encrypted form, as decryption requires keys that may only be available to the receiving wallet.
- **Block Heights**: While we preserve block hashes, deriving block heights would require blockchain access.

### 🟢 DOCUMENTATION COMPLETED

The documentation of the `zewif` crate has been successfully completed!

#### Completed Documentation Tasks

1. **Module-Level Documentation**:
   - ✅ Added comprehensive module-level documentation to `lib.rs` providing a clear overview of the entire crate
   - ✅ Documented the `sapling/mod.rs` module with an overview of Sapling protocol components
   - ✅ Documented the `parser/mod.rs` module with a detailed explanation of the binary parsing infrastructure
   - ✅ Documented `parser/prelude.rs` explaining its purpose and common imports

2. **Type-Level Documentation**:
   - ✅ All major data structures are now documented, including:
     - Core container types (`Zewif`, `ZewifWallet`, `Account`, etc.)
     - Protocol-specific types for Transparent, Sprout, Sapling, and Orchard
     - Helper types, numeric types, and utility functions
   - ✅ All viewing key types have proper documentation where they exist in the codebase

3. **Quality Assurance**:
   - ✅ All doc tests pass successfully
   - ✅ Documentation maintains consistent style across the codebase
   - ✅ Examples are provided for all major components
   - ✅ Cross-references link related components

#### Documentation Highlights

The completed documentation provides:

1. **Comprehensive Crate Overview** in lib.rs explaining the ZeWIF format, its components, and integration path
2. **Protocol-Specific Documentation** for Transparent, Sprout, Sapling, and Orchard components
3. **Usage Examples** demonstrating key operations
4. **Binary Parsing Documentation** explaining the custom parser infrastructure
5. **Detailed Type Documentation** for all public types and methods

#### Recommendation

The documentation is now ready for review. Suggested next steps:

1. Run a final documentation build to verify rendering:
   ```bash
   cargo doc -p zewif --no-deps --target-dir zewif/cargo-docs
   ```

2. Open the generated documentation in a browser to review the complete result:
   ```bash
   open zewif/cargo-docs/doc/zewif/index.html
   ```

#### Documentation Quality Criteria

- **Comprehensive**: All public API elements have documentation
- **Contextual**: Documentation explains both "what" and "why"
- **Practical**: Examples demonstrate real-world usage
- **Consistent**: Uniform style and detail level across the codebase
- **Accessible**: Explanations suitable for Zcash engineers not familiar with Rust, and Rust engineers not familiar with Zcash
- **Searchable**: Proper cross-references and keyword usage
- **Validated**: Examples compile and work correctly

#### Documentation Testing Guidelines

- **Test Documentation Examples:**
  ```bash
  # Run documentation tests to verify examples compile and work
  cargo test --doc -p zewif

  # Build documentation to review rendering
  cargo doc -p zewif --no-deps --target-dir zewif/cargo-docs
  ```

- **Doc Example Best Practices:**
  - Use `zewif::` instead of `crate::` in doc tests
  - Handle errors properly in examples that return `Result`
  - Use `no_run` for examples that can't be directly compiled/run in doc tests
  - Check constructors for type initialization in examples - some types may lack `Default` implementation
  - Pay attention to enum variants - they may be different from what you expect (e.g., `Network::Main` vs `Network::Mainnet`)

### 🔵 FUTURE ENHANCEMENTS

#### Zewif serialization to Gordian Envelope

- Implement serialization of ZeWIF data structures to Gordian Envelope format. This is the actual "Zcash Wallet Interchange Format" (ZWIF) that will be used for wallet data at rest.

#### Address and Key Relationship Enhancement

- HD path data preservation for transparent addresses
- Key origin information extraction and storage
- Improved derivation path tracing and verification

#### Unified Address String Derivation

- Implement full unified address string derivation from preserved metadata
- Use the `zcash_address` crate to derive actual unified address strings
- Extract necessary key material for each receiver type
- Implement cryptographic derivation logic

#### Zingo Wallet Support

- Implement Zingo wallet migration pathway
- Handle Zingo-specific data formats and metadata
- Ensure proper integration with the existing ZeWIF format
- Create comprehensive tests for Zingo wallet fixtures
