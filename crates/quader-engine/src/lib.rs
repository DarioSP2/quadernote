//! Command engine and undo/redo session for QuaderNote.

pub fn engine_placeholder() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_placeholder() {
        assert!(engine_placeholder());
    }
}
