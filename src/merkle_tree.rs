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
    pub fn new(transactions: Vec<H>) -> Result<Self, &'static str> {
        Self::create_tree(transactions)
    }

    // Fathers must have at least one son, if it does not have one, we clone the left one
    fn create_parent_from_siblings(
        left_son: Box<MerkleNode>,
        mut right_son: Option<Box<MerkleNode>>,
    ) -> MerkleNode {
        let mut hasher = DefaultHasher::new();
        left_son.hash_value.hash(&mut hasher);

        match &right_son {
            Some(right_sibling) => {
                right_sibling.hash_value.hash(&mut hasher);
            }
            None => {
                right_son = Some(left_son.clone());
                left_son.hash_value.hash(&mut hasher);
            }
        }

        MerkleNode::new(hasher.finish(), Some(left_son), right_son)
    }

    fn create_tree(transactions: Vec<H>) -> Result<MerkleTree<H>, &'static str> {
        if transactions.is_empty() {
            return Err("Can't create a tree without elements");
        }

        let transactions_hash = Self::get_hashes_of_transactions(&transactions);
        let mut nodes: Vec<Box<MerkleNode>> = transactions_hash
            .into_iter()
            .map(|hash| Box::new(MerkleNode::new(hash, None, None)))
            .collect();

        // We loop all the elements and construct the next level of the tree, we stop once there is only one element (the root)
        while nodes.len() > 1 {
            let mut parents = Vec::new();
            let mut iter = nodes.into_iter();

            while let (Some(left_son), right_son) = (iter.next(), iter.next()) {
                let parent = Self::create_parent_from_siblings(left_son, right_son);
                parents.push(Box::new(parent));
            }

            nodes = parents;
        }

        Ok(Self {
            merkle_root: *nodes[0].clone(),
            leafs: transactions,
        })
    }

    fn get_hashes_of_transactions(transactions: &[H]) -> Vec<u64> {
        transactions
            .iter()
            .map(|transaction| {
                let mut hasher = DefaultHasher::new();
                transaction.hash(&mut hasher);
                hasher.finish()
            })
            .collect()
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
        let mut added_proof = false;
        current_node.left_son.as_ref().inspect(|left| {
            if left.hash_value == transaction_hash {
                current_node.right_son.as_ref().inspect(|right_sibling| {
                    proof.push(SiblingHash::Right(right_sibling.hash_value))
                });
                added_proof = true;
            }
            if Self::recursive_get_proof(left, proof, transaction_hash) {
                current_node.right_son.as_ref().inspect(|right_sibling| {
                    proof.push(SiblingHash::Right(right_sibling.hash_value))
                });
                added_proof = true;
            }
        });

        current_node.right_son.as_ref().inspect(|right| {
            if right.hash_value == transaction_hash {
                current_node
                    .left_son
                    .as_ref()
                    .inspect(|left_sibling| proof.push(SiblingHash::Left(left_sibling.hash_value)));
                added_proof = true;
            }
            if Self::recursive_get_proof(right, proof, transaction_hash) {
                current_node
                    .right_son
                    .as_ref()
                    .inspect(|left_sibling| proof.push(SiblingHash::Left(left_sibling.hash_value)));
                added_proof = true;
            }
        });

        added_proof
    }

    pub fn get_proof(&mut self, transaction: H) -> Vec<SiblingHash> {
        let mut proof = Vec::new();
        let mut hasher = DefaultHasher::new();
        transaction.hash(&mut hasher);
        Self::recursive_get_proof(&self.merkle_root, &mut proof, hasher.finish());
        proof
    }

    pub fn add(&mut self, transaction: H) -> Result<(), &'static str> {
        self.leafs.push(transaction);
        self.merkle_root = Self::create_tree(self.leafs.clone())?.merkle_root;
        Ok(())
    }
}

#[cfg(test)]
pub mod test {

    use crate::merkle_tree::MerkleTree;

    #[test]
    fn cant_create_a_merkle_tree_without_transactions() {
        let transactions: Vec<String> = Vec::new();
        let merkle_tree = MerkleTree::new(transactions);

        assert!(merkle_tree.is_err());
    }

    #[test]
    fn a_merkle_tree_can_contain_one_transaction() {
        let transactions = vec![String::from("A")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof))
    }

    #[test]
    fn a_merkle_tree_can_contain_one_level_of_transactions() {
        let transactions = vec![String::from("A"), String::from("B")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof));
    }
    #[test]
    fn a_merkle_tree_can_contain_two_level_of_transactions() {
        let transactions = vec![
            String::from("A"),
            String::from("B"),
            String::from("C"),
            String::from("D"),
        ];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_can_contain_an_odd_number_of_transactions() {
        let transactions = vec![String::from("A"), String::from("B"), String::from("C")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof));
    }
    #[test]
    fn a_merkle_tree_can_contain_multiple_levels_of_transactions() {
        let transactions = vec![
            String::from("A"),
            String::from("B"),
            String::from("C"),
            String::from("D"),
            String::from("E"),
            String::from("F"),
        ];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_can_add_new_elements() {
        let transactions = vec![String::from("A")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert_eq!(proof.len(), 0);
        assert!(merkle_tree.verify(transaction, proof));

        merkle_tree.add(String::from("B")).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());
        assert_eq!(proof.len(), 1);
        assert!(merkle_tree.verify(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_cant_verify_a_transaction_if_not_present() {
        let transactions = vec![String::from("A"), String::from("B")];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = String::from("C");
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(!merkle_tree.verify(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_can_have_generic_transactions() {
        let transactions = vec![1000, 1500, 2000, 3000, 4000, 5500, 7000, 8700];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof(transaction.clone());

        assert!(merkle_tree.verify(transaction, proof));

        let transactions = vec![
            "De aquel amor",
            "De musica ligera",
            "Nada nos libra,",
            "Nada mas queda",
        ];
        let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
        let transaction = transactions[0];
        let proof = merkle_tree.get_proof(transaction);

        assert!(merkle_tree.verify(transaction, proof));
    }
}
