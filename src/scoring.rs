use std::collections::HashMap;

/// Simple tokenizer: split on whitespace and common punctuation, lowercase.
/// Does not stem — preserves exact tokens for code-heavy content.
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '`' | ':' | '!' | '?'))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub struct Bm25Scorer {
    query_terms: Vec<String>,
    doc_freq: HashMap<String, usize>,
    total_docs: usize,
    avg_doc_len: f64,
    k1: f64,
    b: f64,
}

impl Bm25Scorer {
    /// Build a BM25 scorer from query text and a corpus of document texts.
    /// `corpus` is an iterator of (doc_index, text) pairs.
    pub fn new(query: &str, corpus: &[&str]) -> Self {
        let query_terms = tokenize(query);
        let total_docs = corpus.len().max(1);

        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        let mut total_tokens: usize = 0;

        for text in corpus {
            let tokens = tokenize(text);
            total_tokens += tokens.len();

            // Count unique terms per document
            let mut seen: HashMap<&str, bool> = HashMap::new();
            for token in &tokens {
                if !seen.contains_key(token.as_str()) {
                    *doc_freq.entry(token.clone()).or_insert(0) += 1;
                    seen.insert(token.as_str(), true);
                }
            }
        }

        let avg_doc_len = if total_tokens > 0 {
            total_tokens as f64 / total_docs as f64
        } else {
            1.0
        };

        Self {
            query_terms,
            doc_freq,
            total_docs,
            avg_doc_len,
            k1: 1.2,
            b: 0.75,
        }
    }

    /// Check if a document matches the query.
    /// `require_all`: true = AND mode, false = OR mode.
    pub fn matches(&self, text: &str, require_all: bool) -> bool {
        let text_lower = text.to_lowercase();
        if require_all {
            self.query_terms.iter().all(|t| text_lower.contains(t.as_str()))
        } else {
            self.query_terms.iter().any(|t| text_lower.contains(t.as_str()))
        }
    }

    /// Compute BM25 score for a single document.
    pub fn score(&self, text: &str) -> f64 {
        let tokens = tokenize(text);
        let doc_len = tokens.len() as f64;

        // Build term frequency map for this document
        let mut tf_map: HashMap<&str, usize> = HashMap::new();
        for token in &tokens {
            *tf_map.entry(token.as_str()).or_insert(0) += 1;
        }

        let mut score = 0.0;
        let n = self.total_docs as f64;

        for term in &self.query_terms {
            let df = *self.doc_freq.get(term.as_str()).unwrap_or(&0) as f64;
            let tf = *tf_map.get(term.as_str()).unwrap_or(&0) as f64;

            if tf == 0.0 {
                continue;
            }

            // IDF: log((N - df + 0.5) / (df + 0.5) + 1)
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();

            // TF component: (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * dl / avgdl))
            let tf_norm = (tf * (self.k1 + 1.0))
                / (tf + self.k1 * (1.0 - self.b + self.b * doc_len / self.avg_doc_len));

            score += idf * tf_norm;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("Hello, World! fn main()");
        assert_eq!(tokens, vec!["hello", "world", "fn", "main"]);
    }

    #[test]
    fn test_tokenize_code() {
        let tokens = tokenize("impl Provider for ClaudeProvider");
        assert_eq!(tokens, vec!["impl", "provider", "for", "claudeprovider"]);
    }

    #[test]
    fn test_bm25_basic() {
        let corpus = vec![
            "the auth middleware has a bug in token validation",
            "please fix the CSS styling on the homepage",
            "auth token refresh is failing with 401 errors",
        ];
        let scorer = Bm25Scorer::new("auth bug", &corpus);

        let s0 = scorer.score(corpus[0]);
        let s1 = scorer.score(corpus[1]);
        let s2 = scorer.score(corpus[2]);

        // doc 0 has both "auth" and "bug" → highest
        assert!(s0 > s2, "doc0 ({s0}) should score higher than doc2 ({s2})");
        assert!(s0 > s1, "doc0 ({s0}) should score higher than doc1 ({s1})");
        // doc 2 has "auth" but not "bug"
        assert!(s2 > s1, "doc2 ({s2}) should score higher than doc1 ({s1})");
    }

    #[test]
    fn test_bm25_matches_and_mode() {
        let corpus = vec!["auth bug fix", "auth only", "bug only"];
        let scorer = Bm25Scorer::new("auth bug", &corpus);

        assert!(scorer.matches("auth bug fix", true));
        assert!(!scorer.matches("auth only", true));
        assert!(scorer.matches("auth only", false));
        assert!(scorer.matches("bug only", false));
        assert!(!scorer.matches("nothing here", false));
    }

    #[test]
    fn test_bm25_empty_corpus() {
        let corpus: Vec<&str> = vec![];
        let scorer = Bm25Scorer::new("test query", &corpus);
        let s = scorer.score("test query here");
        assert!(s > 0.0, "score should be positive, got {s}");
    }

    #[test]
    fn test_bm25_single_term() {
        let corpus = vec!["apple banana", "apple apple apple", "banana only"];
        let scorer = Bm25Scorer::new("apple", &corpus);

        let s0 = scorer.score(corpus[0]);
        let s1 = scorer.score(corpus[1]);
        let s2 = scorer.score(corpus[2]);

        // doc with more "apple" occurrences should score higher
        assert!(s1 > s0, "doc1 ({s1}) should > doc0 ({s0})");
        assert!(s0 > s2, "doc0 ({s0}) should > doc2 ({s2})");
        assert_eq!(s2, 0.0, "doc2 has no 'apple'");
    }
}
