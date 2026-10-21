//! Document model for QuaderNote.

pub fn doc_placeholder() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_placeholder() {
        assert!(doc_placeholder());
    }
}
