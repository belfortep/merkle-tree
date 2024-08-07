use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone)]
struct MerkleNode {
    hash_value: u64,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
}
struct MerkleTree {
    merkle_root: MerkleNode,
}

impl MerkleNode {
    pub fn new(hash_value: u64) -> Self {
        Self {
            hash_value,
            left: None,
            right: None,
        }
    }
}

impl MerkleTree {
    pub fn new<H: Hash>(transactions: Vec<H>) -> Result<Self, String> {
        if transactions.is_empty() {
            return Err("Empty transactions vector".into());
        }

        let transactions_hash = Self::get_hashes(transactions);

        Ok(Self::create_tree_from_hashes(transactions_hash))
    }

    fn create_tree_from_hashes(transactions_hash: Vec<u64>) -> MerkleTree {
        let mut leafs: Vec<Box<MerkleNode>> = Vec::new();
        for hash in transactions_hash {
            leafs.push(Box::new(MerkleNode::new(hash)));
        }

        while leafs.len() > 1 {
            let mut parents = Vec::new();

            for _ in (0..leafs.len()).step_by(2) {
                let mut hasher = DefaultHasher::new();
                let left = leafs.pop();
                let right = leafs.pop();

                if let Some(left) = &left {
                    left.hash_value.hash(&mut hasher);
                }

                if let Some(right) = &right {
                    right.hash_value.hash(&mut hasher);
                }

                let mut parent = MerkleNode::new(hasher.finish());
                parent.left = left;
                parent.right = right;
                parents.push(Box::new(parent));
            }

            leafs = parents;
        }

        Self {
            merkle_root: *leafs[0].clone(),
        }
    }

    fn get_hashes<H: Hash>(transactions: Vec<H>) -> Vec<u64> {
        let mut transactions_hash = Vec::new();
        for transaction in transactions {
            let mut hasher = DefaultHasher::new();
            transaction.hash(&mut hasher);
            let hash = hasher.finish();
            transactions_hash.push(hash);
        }
        transactions_hash
    }

    pub fn contains<H: Hash>(&mut self, transaction: H) -> bool {
        let mut hasher = DefaultHasher::new();

        transaction.hash(&mut hasher);
        let hash = hasher.finish();

        self.merkle_root.hash_value == hash
    }
}

#[cfg(test)]
pub mod test {

    use crate::merkle::merkle_tree::MerkleTree;

    #[test]
    fn test_001_a_new_merkle_tree_contains_nothing() {
        let transactions: Vec<String> = Vec::new();
        let mut merkle_tree = MerkleTree::new(transactions);

        assert!(merkle_tree.is_err());
    }

    #[test]
    fn test_002_a_merkle_tree_can_contains_one_transaction() {
        let transactions = vec![String::from("hi")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        assert!(merkle_tree.contains(transaction));
    }

    #[test]
    fn test_003_a_merkle_tree_can_contains_multiple_transactions() {
        let transactions = vec![String::from("hi"), String::from("bye")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let another_transaction = transactions[1].clone();
        assert!(merkle_tree.contains(transaction));
        assert!(merkle_tree.contains(another_transaction));
    }
}
