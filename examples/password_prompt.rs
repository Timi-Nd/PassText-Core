use std::error::Error;

use passtext_core::{process_password, NormalizationMode, PassTextConfig};

fn main() -> Result<(), Box<dyn Error>> {
    let config = PassTextConfig::new(4096, 1024, NormalizationMode::Nfc)?;
    let password = rpassword::prompt_password("Enter password: ")?;
    let prepared = process_password(&password, config)?;
    let metadata = prepared.metadata();

    println!("Original code points: {}", metadata.original_codepoints());
    println!("Original UTF-8 bytes: {}", metadata.original_utf8_bytes());
    println!("Processed code points: {}", metadata.processed_codepoints());
    println!("Processed UTF-8 bytes: {}", metadata.processed_utf8_bytes());
    println!(
        "Normalization changed: {}",
        metadata.normalization_changed()
    );
    Ok(())
}
