use nlprule::{tokenizer_filename, Tokenizer};

pub mod noun_phrases;

/// Loads the tokenizer from the binary file for small nlprules NLP crate
/// Panics if the tokenizer binary is invalid
pub fn load_nlprules_tokenizer() -> Tokenizer {
    let mut tokenizer_bytes: &'static [u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/", tokenizer_filename!("en")));

    Tokenizer::from_reader(&mut tokenizer_bytes).expect("tokenizer binary is valid")
}
