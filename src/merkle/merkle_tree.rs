use std::hash::{DefaultHasher, Hash, Hasher};

struct MerkleTree {
    transactions_hash: Vec<u64>,
}

impl MerkleTree {
    pub fn new<H: Hash>(transactions: Vec<H>) -> Self {
        let mut transactions_hash = Vec::new();
        for transaction in transactions {
            let mut hasher = DefaultHasher::new();
            transaction.hash(&mut hasher);
            let hash = hasher.finish();
            transactions_hash.push(hash);
        }

        Self { transactions_hash }
    }

    pub fn contains<H: Hash>(&mut self, transaction: H) -> bool {
        let mut hasher = DefaultHasher::new();

        transaction.hash(&mut hasher);
        let hash = hasher.finish();

        self.transactions_hash.contains(&hash)
    }
}

#[cfg(test)]
pub mod test {

    use crate::merkle::merkle_tree::MerkleTree;

    #[test]
    fn test_001_a_new_merkle_tree_contains_nothing() {
        let transactions: Vec<String> = Vec::new();
        let mut merkle_tree = MerkleTree::new(transactions);
        let transaction_not_in_tree = String::from("Hi");
        assert!(!merkle_tree.contains(transaction_not_in_tree));
    }

    #[test]
    fn test_002_a_merkle_tree_can_contains_one_transaction() {
        let transactions = vec![String::from("hi")];
        let mut merkle_tree = MerkleTree::new(transactions.clone());
        let transaction = transactions[0].clone();
        assert!(merkle_tree.contains(transaction));
    }

    #[test]
    fn test_003_a_merkle_tree_can_contains_one_transaction() {
        let transactions = vec![String::from("hi")];
        let mut merkle_tree = MerkleTree::new(transactions.clone());
        let transaction = transactions[0].clone();
        assert!(merkle_tree.contains(transaction));
    }
}
