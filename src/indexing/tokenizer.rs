pub fn tokenize(text: &str) -> Vec<String> {
    // 1. Convert to lowercase once (allocates one String for the text)
    let lowercase = text.to_lowercase();

    // 2. Stream through whitespace, trim, filter, and only then allocate final Strings
    lowercase
        .split_whitespace()
        .map(|word| word.trim_matches(|ch: char| !ch.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .map(|word| word.to_string()) // Only allocate for words that actually survive!
        .collect()
}
