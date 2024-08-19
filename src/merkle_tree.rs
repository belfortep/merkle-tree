use std::rc::Rc;

use sha3::{Digest, Sha3_256};

pub enum SiblingHash {
    Left(String),
    Right(String),
}

#[derive(Clone)]
struct InnerNode {
    hash_value: String,
    left_son: Rc<MerkleNode>,
    right_son: Rc<MerkleNode>,
}
#[derive(Clone)]
struct LeafNode {
    hash_value: String,
}

#[derive(Clone)]
enum MerkleNode {
    Inner(InnerNode),
    Leaf(LeafNode),
}
pub struct MerkleTree<H: AsRef<[u8]> + Clone> {
    merkle_root: MerkleNode,
    leaves: Vec<H>,
}

impl MerkleNode {
    pub fn get_hash_value(&self) -> String {
        match self {
            Self::Inner(node) => node.hash_value.clone(),
            Self::Leaf(node) => node.hash_value.clone(),
        }
    }
}

impl InnerNode {
    pub fn new(hash_value: String, left_son: Rc<MerkleNode>, right_son: Rc<MerkleNode>) -> Self {
        Self {
            hash_value,
            left_son,
            right_son,
        }
    }
}

impl LeafNode {
    pub fn new(hash_value: String) -> Self {
        Self { hash_value }
    }
}

impl<H: AsRef<[u8]> + Clone> MerkleTree<H> {
    pub fn new(transactions: Vec<H>) -> Result<Self, &'static str> {
        Self::create_tree(transactions)
    }

    // Fathers must have at least one son, if it does not have one, we clone the left one
    fn create_parent_from_siblings(
        left_son: MerkleNode,
        right_son: Option<MerkleNode>,
    ) -> MerkleNode {
        let mut hasher = Sha3_256::new();
        hasher.update(left_son.get_hash_value());
        let right_son = right_son.unwrap_or_else(|| left_son.clone());
        hasher.update(right_son.get_hash_value());

        MerkleNode::Inner(InnerNode::new(
            format!("{:X}", hasher.finalize()),
            Rc::new(left_son),
            Rc::new(right_son),
        ))
    }

    fn create_tree(transactions: Vec<H>) -> Result<MerkleTree<H>, &'static str> {
        if transactions.is_empty() {
            return Err("Can't create a tree without elements");
        }

        let mut nodes: Vec<MerkleNode> = transactions
            .iter()
            .map(|transaction| {
                let mut hasher = Sha3_256::new();
                hasher.update(transaction);
                MerkleNode::Leaf(LeafNode::new(format!("{:X}", hasher.finalize())))
            })
            .collect();

        // We loop all the elements and construct the next level of the tree, we stop once there is only one element (the root)
        let mut parents: Vec<MerkleNode> = Vec::with_capacity(nodes.len());
        while nodes.len() > 1 {
            let mut iter = nodes.into_iter();

            while let (Some(left_son), right_son) = (iter.next(), iter.next()) {
                let parent = Self::create_parent_from_siblings(left_son, right_son);
                parents.push(parent);
            }
            nodes = parents[0..parents.len()].to_vec();
            parents.clear();
        }

        Ok(Self {
            merkle_root: nodes.pop().unwrap(), // never panics, as 'nodes' have exactly one element at this point!
            leaves: transactions,
        })
    }

    pub fn verify(&mut self, transaction: H, proof: Vec<SiblingHash>) -> bool {
        let mut hasher = Sha3_256::new();
        hasher.update(transaction);
        let mut transaction = format!("{:X}", hasher.finalize());
        for sibling_hash in proof {
            let mut hasher = Sha3_256::new();
            match sibling_hash {
                SiblingHash::Left(left_hash) => {
                    hasher.update(left_hash);
                    hasher.update(transaction);
                }
                SiblingHash::Right(right_hash) => {
                    hasher.update(transaction);
                    hasher.update(right_hash);
                }
            }

            transaction = format!("{:X}", hasher.finalize());
        }

        transaction == self.merkle_root.get_hash_value()
    }

    fn recursive_get_proof(
        current_node: &MerkleNode,
        proof: &mut Vec<SiblingHash>,
        transaction_hash: String,
    ) -> bool {
        match current_node {
            MerkleNode::Inner(node) => {
                if node.left_son.get_hash_value() == transaction_hash {
                    proof.push(SiblingHash::Right(node.right_son.get_hash_value()));
                    return true;
                }
                if Self::recursive_get_proof(&node.left_son, proof, transaction_hash.clone()) {
                    proof.push(SiblingHash::Right(node.right_son.get_hash_value()));
                    return true;
                }

                if node.right_son.get_hash_value() == transaction_hash {
                    proof.push(SiblingHash::Left(node.left_son.get_hash_value()));
                    return true;
                }
                if Self::recursive_get_proof(&node.right_son, proof, transaction_hash) {
                    proof.push(SiblingHash::Left(node.left_son.get_hash_value()));
                    return true;
                }

                false
            }
            MerkleNode::Leaf(_) => false,
        }
    }

    pub fn get_proof(&mut self, transaction: H) -> Vec<SiblingHash> {
        let mut proof = Vec::new();
        let mut hasher = Sha3_256::new();
        hasher.update(transaction);
        Self::recursive_get_proof(
            &self.merkle_root,
            &mut proof,
            format!("{:X}", hasher.finalize()),
        );
        proof
    }

    pub fn add(&mut self, transaction: H) -> Result<(), &'static str> {
        self.leaves.push(transaction);
        self.merkle_root = Self::create_tree(self.leaves.clone())?.merkle_root;
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
        let transactions = vec![vec![10, 20, 30], vec![20, 30, 40], vec![100, 150, 200]];
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
