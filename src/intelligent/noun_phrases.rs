use nlprule::{
    types::{owned::Token, Sentence},
    Tokenizer,
};

#[derive(Debug)]
pub struct NounPhrase {
    pub words: Vec<String>,
    pub start: usize,
    pub end: usize,
}

impl std::fmt::Display for NounPhrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.words.join(" "))
    }
}

/// Given a text, extract all noun phrases from it
/// Noun phrases may overlap.
/// They may also allow ambiguous nouns to be bundled together. This is currently intentional. TODO: Is this the best option?
///     eg: "adult red dragon" -> ["adult red dragon", "adult", "red dragon"]
/// But  it may have bad results for others.
pub fn extract_noun_phrases_from_text(tokenizer: &Tokenizer, text: &str) -> Vec<NounPhrase> {
    // Tokenize into sentences, then concat them together from the pieces
    let mut all_phrases = vec![];
    for sentence in tokenizer.pipe(text) {
        let noun_phrases = concat_noun_phrases(sentence);
        all_phrases.extend(noun_phrases);
    }
    all_phrases
}

/// Concat noun phrases from chunks of individual words with assigned noun-phrase-ness
fn concat_noun_phrases(sentence: Sentence<'_>) -> Vec<NounPhrase> {
    let mut noun_phrases = vec![];
    let mut temp_noun_phrases: Vec<Vec<Token>> = vec![];
    let mut ready_to_clear = false;
    for token in sentence {
        let mut added = false;
        let word = token.to_owned_token();

        // No chunks- apostophes, random characters.
        // Allow these to pass, so stuff like "men's wives" can be caught
        if token.chunks().is_empty() {
            // Add to temp noun phrase
            for temp_noun_phrase in temp_noun_phrases.iter_mut() {
                temp_noun_phrase.push(word.clone());
            }
        }

        for chunk in token.chunks() {
            // Start of a new noun phrase
            if chunk.starts_with("B-NP") {
                // Append to each temp noun phrase
                for temp_noun_phrase in temp_noun_phrases.iter_mut() {
                    temp_noun_phrase.push(word.clone());
                }
                // Start new temp noun phrase
                temp_noun_phrases.push(vec![word.clone()]);

                added = true;
            }

            // Continue noun phrase
            if chunk.starts_with("I-NP") {
                // Append to each temp noun phrase
                for temp_noun_phrase in temp_noun_phrases.iter_mut() {
                    if !added {
                        temp_noun_phrase.push(word.clone());
                    }
                }
                added = true;
            }

            // End of noun phrase
            // This may not guarantee an end
            if chunk.starts_with("E-NP") {
                // Append to each temp noun phrase
                for temp_noun_phrase in temp_noun_phrases.iter_mut() {
                    if !added {
                        temp_noun_phrase.push(word.clone());
                    }

                    // Finalize temp noun phrase
                    noun_phrases.push(temp_noun_phrase.clone());
                }
                added = true;

                // Clear temp noun phrases
                ready_to_clear = true;
            }

            if !chunk.contains("-NP") && ready_to_clear {
                temp_noun_phrases.clear();
                ready_to_clear = false;
            }
        }
    }

    // Convert vecs of tokens to nounphrases
    let mut noun_phrases_final = vec![];
    for noun_phrase in noun_phrases {
        if noun_phrase.is_empty() {
            continue;
        }
        let first = noun_phrase.first().unwrap();
        let last = noun_phrase.last().unwrap();
        let np = NounPhrase {
            words: noun_phrase
                .iter()
                .map(|x| x.word.text.as_ref_id().as_str().to_string())
                .collect(),
            start: first.span.start().char,
            end: last.span.end().char,
        };
        noun_phrases_final.push(np);
    }

    noun_phrases_final
}
