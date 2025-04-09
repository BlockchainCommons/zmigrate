use std::path::Path;

use anyhow::{Ok, Result};
use zewif::Data;

// Import shared test utilities
mod test_utils;
use test_utils::fixtures_path;
use zewif_zwl::ZwlParser;

/// Attempts to decrypt a zecwallet wallet file with the given password, and compares the seed phrase
/// to the expected phrase.
fn test_parser_encryption(wallet_path: &[&str], expected_phrase: &str, password: &str) {
    let file_path = fixtures_path(wallet_path);

    let file_data = Data::from_vec(
        std::fs::read(Path::new(&file_path))
            .expect(&format!("Failed to read wallet file: {:?}", wallet_path)),
    );
    let mut parser = ZwlParser::new(&file_data);
    let wallet = parser.parse();

    let mut real_wallet = wallet.unwrap();

    let phrase = real_wallet
        .keys
        .get_phrase(String::from(password))
        .unwrap()
        .into_phrase();

    assert_eq!(phrase, expected_phrase, "Expected phrase does not match");

    real_wallet
        .keys
        .unlock_wallet(password.to_string())
        .unwrap();
}

#[test]
fn test_zwl_decryption() -> Result<()> {
    let paths = vec![vec!["zwl", "mainnet", "zwl-encrypted.dat"]];
    for path in &paths {
        test_parser_encryption(
            path,
            "basket decorate ivory office buddy embark country office trophy speak cupboard mixture crazy agent lemon permit build situate omit spider bridge panda rather chuckle",
            "hello world",
        );
    }
    Ok(())
}
