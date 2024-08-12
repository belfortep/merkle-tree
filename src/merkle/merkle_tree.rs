use std::hash::{DefaultHasher, Hash, Hasher};

pub enum SiblingHash {
    Left(u64),
    Right(u64),
}

#[derive(Clone)]
struct MerkleNode {
    hash_value: u64,
    left_son: Option<Box<MerkleNode>>,
    right_son: Option<Box<MerkleNode>>,
}
pub struct MerkleTree<H: Hash + Clone> {
    merkle_root: MerkleNode,
    leafs: Vec<H>,
}

impl MerkleNode {
    pub fn new(
        hash_value: u64,
        left_son: Option<Box<MerkleNode>>,
        right_son: Option<Box<MerkleNode>>,
    ) -> Self {
        Self {
            hash_value,
            left_son,
            right_son,
        }
    }
}

impl<H: Hash + Clone> MerkleTree<H> {
    pub fn new(transactions: Vec<H>) -> Self {
        Self::create_tree(transactions)
    }

    fn create_parent_from_siblings(
        left_son: Option<Box<MerkleNode>>,
        mut right_son: Option<Box<MerkleNode>>,
    ) -> MerkleNode {
        let mut hasher = DefaultHasher::new();

        if let Some(left_sibling) = &left_son {
            left_sibling.hash_value.hash(&mut hasher);
            if let Some(right_sibling) = &right_son {
                right_sibling.hash_value.hash(&mut hasher);
            } else {
                right_son = left_son.clone();
                left_sibling.hash_value.hash(&mut hasher);
            }
        }
        MerkleNode::new(hasher.finish(), left_son, right_son)
    }

    fn create_tree(transactions: Vec<H>) -> MerkleTree<H> {
        if transactions.is_empty() {
            return Self {
                merkle_root: MerkleNode::new(0, None, None),
                leafs: transactions,
            };
        }

        let transactions_hash = Self::get_hashes_of_transactions(&transactions);

        let mut nodes = Vec::new();
        for hash in transactions_hash {
            nodes.push(Box::new(MerkleNode::new(hash, None, None)));
        }

        while nodes.len() > 1 {
            let mut parents = Vec::new();

            for _ in (0..nodes.len()).step_by(2) {
                let parent = Self::create_parent_from_siblings(nodes.pop(), nodes.pop());
                parents.push(Box::new(parent));
            }

            nodes = parents;
        }

        Self {
            merkle_root: *nodes[0].clone(),
            leafs: transactions,
        }
    }

    fn get_hashes_of_transactions(transactions: &Vec<H>) -> Vec<u64> {
        let mut transactions_hash = Vec::new();
        for transaction in transactions {
            let mut hasher = DefaultHasher::new();
            transaction.hash(&mut hasher);
            let transaction_hash = hasher.finish();
            transactions_hash.push(transaction_hash);
        }
        transactions_hash
    }

    pub fn verify(&mut self, transaction: H, proof: Vec<SiblingHash>) -> bool {
        let mut hasher = DefaultHasher::new();
        transaction.hash(&mut hasher);
        let mut transaction = hasher.finish();
        for sibling_hash in proof {
            hasher = DefaultHasher::new();
            match sibling_hash {
                SiblingHash::Left(left_hash) => {
                    left_hash.hash(&mut hasher);
                    transaction.hash(&mut hasher);
                }
                SiblingHash::Right(right_hash) => {
                    transaction.hash(&mut hasher);
                    right_hash.hash(&mut hasher);
                }
            }

            transaction = hasher.finish();
        }

        transaction == self.merkle_root.hash_value
    }

    fn recursive_get_proof(
        current_node: &MerkleNode,
        proof: &mut Vec<SiblingHash>,
        transaction_hash: u64,
    ) -> bool {
        if let Some(left) = &current_node.left_son {
            if left.hash_value == transaction_hash {
                if let Some(right_sibling) = &current_node.right_son {
                    proof.push(SiblingHash::Right(right_sibling.hash_value));
                }
                return true;
            }
            if Self::recursive_get_proof(left, proof, transaction_hash) {
                if let Some(right_sibling) = &current_node.right_son {
                    proof.push(SiblingHash::Right(right_sibling.hash_value));
                }
                return true;
            }
        }

        if let Some(right) = &current_node.right_son {
            if right.hash_value == transaction_hash {
                if let Some(left_sibling) = &current_node.left_son {
                    proof.push(SiblingHash::Left(left_sibling.hash_value));
                }
                return true;
            }

            if Self::recursive_get_proof(right, proof, transaction_hash) {
                if let Some(left_sibling) = &current_node.left_son {
                    proof.push(SiblingHash::Left(left_sibling.hash_value));
                }
                return true;
            }
        }
        false
    }

    pub fn get_proof(&mut self, transaction: H) -> Vec<SiblingHash> {
        let mut proof = Vec::new();
        let mut hasher = DefaultHasher::new();
        transaction.hash(&mut hasher);
        Self::recursive_get_proof(&self.merkle_root, &mut proof, hasher.finish());

        proof
    }

    pub fn add(&mut self, transaction: H) {
        self.leafs.push(transaction);
        let mut leafs = Vec::new();
        for leaf in &self.leafs {
            leafs.push(leaf.clone());
        }
        self.merkle_root = Self::create_tree(leafs.clone()).merkle_root;
    }
}

#[cfg(test)]
pub mod test {

    use crate::merkle::merkle_tree::MerkleTree;

    #[test]
    fn a_new_merkle_tree_contains_nothing() {
        let transactions: Vec<String> = Vec::new();
        let merkle_tree = MerkleTree::new(transactions);

        assert!(merkle_tree.leafs.is_empty());
    }

    #[test]
    fn a_merkle_tree_can_contains_one_transaction() {
        let transactions = vec![String::from("A")];
        let mut merkle_tree = MerkleTree::new(transactions.clone());
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof))
    }

    #[test]
    fn a_merkle_tree_can_contains_one_level_of_transactions() {
        let transactions = vec![String::from("A"), String::from("B")];
        let mut merkle_tree = MerkleTree::new(transactions.clone());
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof));
    }
    #[test]
    fn a_merkle_tree_can_contains_two_level_of_transactions() {
        let transactions = vec![
            String::from("A"),
            String::from("B"),
            String::from("C"),
            String::from("D"),
        ];
        let mut merkle_tree = MerkleTree::new(transactions.clone());
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_can_contains_an_odd_number_of_transactions() {
        let transactions = vec![String::from("A"), String::from("B"), String::from("C")];
        let mut merkle_tree = MerkleTree::new(transactions.clone());
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof));
    }
    #[test]
    fn a_merkle_tree_can_contains_multiple_levels_of_transactions() {
        let transactions = vec![
            String::from("A"),
            String::from("B"),
            String::from("C"),
            String::from("D"),
            String::from("E"),
            String::from("F"),
        ];
        let mut merkle_tree = MerkleTree::new(transactions.clone());
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_can_add_new_elements() {
        let transactions = vec![String::from("A")];
        let mut merkle_tree = MerkleTree::new(transactions.clone());
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert_eq!(proof.len(), 0);
        assert!(merkle_tree.verify(transaction, proof));

        merkle_tree.add(String::from("B"));
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());
        assert_eq!(proof.len(), 1);
        assert!(merkle_tree.verify(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_cant_verify_a_transaction_if_not_present() {
        let transactions = vec![String::from("A"), String::from("B")];
        let mut merkle_tree = MerkleTree::new(transactions.clone());
        let transaction = String::from("C");
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(!merkle_tree.verify(transaction, proof));
    }
}
