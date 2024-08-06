use std::hash::Hash;

struct MerkleTree {}

impl MerkleTree {
    pub fn new() -> Self {
        Self {}
    }

    pub fn contains<H: Hash>(&self, data: H) -> bool {
        false
    }
}

#[cfg(test)]
pub mod test {
    use crate::merkle::merkle_tree::MerkleTree;

    #[test]
    fn test_001_xxx() {
        let merkle_tree = MerkleTree::new();
        let transaction_not_in_tree = "hi";
        assert!(!merkle_tree.contains(transaction_not_in_tree));
    }
}
