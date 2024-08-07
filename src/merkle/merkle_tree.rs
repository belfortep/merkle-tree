use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone)]
struct MerkleNode {
    hash_value: u64,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
}
pub struct MerkleTree<H: Hash + Clone> {
    merkle_root: MerkleNode,
    leafs: Vec<H>,
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

impl<H: Hash + Clone> MerkleTree<H> {
    pub fn new(transactions: Vec<H>) -> Result<Self, String> {
        if transactions.is_empty() {
            return Err("Empty transactions vector".into());
        }

        Ok(Self::create_tree(transactions))
    }

    fn create_tree(transactions: Vec<H>) -> MerkleTree<H> {
        let transactions_hash = Self::get_hashes(&transactions);

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
                let hash = hasher.finish();
                let mut parent = MerkleNode::new(hash);
                parent.left = left;
                parent.right = right;
                parents.push(Box::new(parent));
            }

            leafs = parents;
        }

        Self {
            merkle_root: *leafs[0].clone(),
            leafs: transactions,
        }
    }

    fn get_hashes(transactions: &Vec<H>) -> Vec<u64> {
        let mut transactions_hash = Vec::new();
        for transaction in transactions {
            let mut hasher = DefaultHasher::new();
            transaction.hash(&mut hasher);
            let hash = hasher.finish();
            transactions_hash.push(hash);
        }
        transactions_hash
    }

    pub fn verify(&mut self, transaction: H, proof: Vec<u64>) -> bool {
        if proof.is_empty() {
            let mut hasher = DefaultHasher::new();
            transaction.hash(&mut hasher);
            return hasher.finish() == self.merkle_root.hash_value;
        }

        let mut hasher = DefaultHasher::new();

        transaction.hash(&mut hasher);
        let mut transaction = hasher.finish();
        for p in proof {
            hasher = DefaultHasher::new();
            p.hash(&mut hasher);
            transaction.hash(&mut hasher);
            transaction = hasher.finish();
        }

        transaction == self.merkle_root.hash_value
    }

    fn recursive_get_proof(
        actual_node: &MerkleNode,
        mut proof: &mut Vec<u64>,
        transaction_hash: u64,
    ) -> bool {
        if let Some(left) = &actual_node.left {
            if left.hash_value == transaction_hash {
                proof.push(actual_node.right.as_ref().unwrap().hash_value);
                return true;
            }
            if Self::recursive_get_proof(&left, &mut proof, transaction_hash) {
                proof.push(actual_node.right.as_ref().unwrap().hash_value);
                return true;
            }
        }

        if let Some(right) = &actual_node.right {
            if right.hash_value == transaction_hash {
                proof.push(actual_node.left.as_ref().unwrap().hash_value);
                return true;
            }

            if Self::recursive_get_proof(&right, &mut proof, transaction_hash) {
                proof.push(actual_node.left.as_ref().unwrap().hash_value);
                return true;
            }
        }
        false
    }

    pub fn get_proof(&mut self, transaction: H) -> Vec<u64> {
        let mut proof = Vec::new();
        let mut hasher = DefaultHasher::new();
        transaction.hash(&mut hasher);
        Self::recursive_get_proof(&self.merkle_root, &mut proof, hasher.finish());

        proof
    }
    pub fn add(&mut self, transaction: H) {
        self.leafs.push(transaction);

        self.merkle_root = Self::create_tree(self.leafs.clone()).merkle_root;
    }
}

#[cfg(test)]
pub mod test {

    use crate::merkle::merkle_tree::MerkleTree;

    #[test]
    fn test_001_a_new_merkle_tree_contains_nothing() {
        let transactions: Vec<String> = Vec::new();
        let merkle_tree = MerkleTree::new(transactions);

        assert!(merkle_tree.is_err());
    }

    #[test]
    fn test_002_a_merkle_tree_can_contains_one_transaction() {
        let transactions = vec![String::from("A")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());
        assert!(merkle_tree.verify(transaction, proof))
    }

    #[test]
    fn test_003_a_merkle_tree_can_contains_one_level_of_transactions() {
        let transactions = vec![String::from("A"), String::from("B")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());
        assert!(merkle_tree.verify(transaction, proof));
    }
    #[test]
    fn test_004_a_merkle_tree_can_contains_two_level_of_transactions() {
        let transactions = vec![
            String::from("A"),
            String::from("B"),
            String::from("C"),
            String::from("D"),
        ];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[2].clone();
        let proof = merkle_tree.get_proof(transaction.clone());
        assert!(merkle_tree.verify(transaction, proof));
    }

    #[test]
    fn test_005_a_merkle_tree_can_contains_an_odd_number_of_transactions() {
        let transactions = vec![String::from("A"), String::from("B"), String::from("C")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[1].clone();
        let proof = merkle_tree.get_proof(transaction.clone());
        assert!(merkle_tree.verify(transaction, proof));
    }
    #[test]
    fn test_006_a_merkle_tree_can_contains_multiple_levels_of_transactions() {
        let transactions = vec![
            String::from("A"),
            String::from("B"),
            String::from("C"),
            String::from("D"),
            String::from("E"),
            String::from("F"),
        ];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[2].clone();
        let proof = merkle_tree.get_proof(transaction.clone());
        assert!(merkle_tree.verify(transaction, proof));
    }

    #[test]
    fn test_007_a_merkle_tree_can_add_new_elements() {
        let transactions = vec![String::from("A")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());
        assert!(merkle_tree.verify(transaction, proof));
        merkle_tree.add(String::from("B"));
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());
        assert_eq!(proof.len(), 1);
        assert!(merkle_tree.verify(transaction, proof));
    }
}
