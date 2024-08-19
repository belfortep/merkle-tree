use std::rc::Rc;

use sha3::Digest;

pub enum SiblingHash {
    Left(Vec<u8>),
    Right(Vec<u8>),
}

#[derive(Clone)]
struct InnerNode {
    hash_value: Vec<u8>,
    left_son: Rc<MerkleNode>,
    right_son: Rc<MerkleNode>,
}
#[derive(Clone)]
struct LeafNode {
    hash_value: Vec<u8>,
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
    pub fn get_hash_value(&self) -> Vec<u8> {
        match self {
            Self::Inner(node) => node.hash_value.clone(),
            Self::Leaf(node) => node.hash_value.clone(),
        }
    }
}

impl InnerNode {
    pub fn new(hash_value: Vec<u8>, left_son: Rc<MerkleNode>, right_son: Rc<MerkleNode>) -> Self {
        Self {
            hash_value,
            left_son,
            right_son,
        }
    }
}

impl LeafNode {
    pub fn new(hash_value: Vec<u8>) -> Self {
        Self { hash_value }
    }
}

impl<H: AsRef<[u8]> + Clone> MerkleTree<H> {
    pub fn new<D: Digest>(transactions: Vec<H>) -> Result<Self, &'static str> {
        Self::create_tree::<D>(transactions)
    }

    // Fathers must have at least one son, if it does not have one, we clone the left one
    fn create_parent_from_siblings<D: Digest>(
        left_son: MerkleNode,
        right_son: Option<MerkleNode>,
    ) -> MerkleNode {
        let mut hasher = D::new();
        hasher.update(left_son.get_hash_value());
        let right_son = right_son.unwrap_or_else(|| left_son.clone());
        hasher.update(right_son.get_hash_value());

        MerkleNode::Inner(InnerNode::new(
            hasher.finalize().to_ascii_lowercase(),
            Rc::new(left_son),
            Rc::new(right_son),
        ))
    }

    fn create_tree<D: Digest>(transactions: Vec<H>) -> Result<MerkleTree<H>, &'static str> {
        if transactions.is_empty() {
            return Err("Can't create a tree without elements");
        }

        let mut nodes: Vec<MerkleNode> = transactions
            .iter()
            .map(|transaction| {
                let mut hasher = D::new();
                hasher.update(transaction);
                MerkleNode::Leaf(LeafNode::new(hasher.finalize().to_ascii_lowercase()))
            })
            .collect();

        // We loop all the elements and construct the next level of the tree, we stop once there is only one element (the root)
        let mut parents: Vec<MerkleNode> = Vec::with_capacity(nodes.len());
        while nodes.len() > 1 {
            let mut iter = nodes.into_iter();

            while let (Some(left_son), right_son) = (iter.next(), iter.next()) {
                let parent = Self::create_parent_from_siblings::<D>(left_son, right_son);
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

    pub fn verify<D: Digest>(&mut self, transaction: H, proof: Vec<SiblingHash>) -> bool {
        let mut hasher = D::new();
        hasher.update(transaction);
        let mut transaction = hasher.finalize().to_ascii_lowercase();
        for sibling_hash in proof {
            let mut hasher = D::new();
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

            transaction = hasher.finalize().to_ascii_lowercase();
        }

        transaction == self.merkle_root.get_hash_value()
    }

    fn recursive_get_proof(
        current_node: &MerkleNode,
        proof: &mut Vec<SiblingHash>,
        transaction_hash: Vec<u8>,
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

    pub fn get_proof<D: Digest>(&mut self, transaction: H) -> Vec<SiblingHash> {
        let mut proof = Vec::new();
        let mut hasher = D::new();
        hasher.update(transaction);

        Self::recursive_get_proof(
            &self.merkle_root,
            &mut proof,
            hasher.finalize().to_ascii_lowercase(),
        );
        proof
    }

    pub fn add<D: Digest>(&mut self, transaction: H) -> Result<(), &'static str> {
        self.leaves.push(transaction);
        self.merkle_root = Self::create_tree::<D>(self.leaves.clone())?.merkle_root;
        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use sha3::{Sha3_256, Sha3_384};

    use crate::merkle_tree::MerkleTree;

    #[test]
    fn cant_create_a_merkle_tree_without_transactions() {
        let transactions: Vec<String> = Vec::new();
        let merkle_tree = MerkleTree::new::<Sha3_256>(transactions);

        assert!(merkle_tree.is_err());
    }

    #[test]
    fn a_merkle_tree_can_contain_one_transaction() {
        let transactions = vec![String::from("A")];
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());

        assert!(merkle_tree.verify::<Sha3_256>(transaction, proof))
    }

    #[test]
    fn a_merkle_tree_can_contain_one_level_of_transactions() {
        let transactions = vec![String::from("A"), String::from("B")];
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());

        assert!(merkle_tree.verify::<Sha3_256>(transaction, proof));
    }
    #[test]
    fn a_merkle_tree_can_contain_two_level_of_transactions() {
        let transactions = vec![
            String::from("A"),
            String::from("B"),
            String::from("C"),
            String::from("D"),
        ];
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());

        assert!(merkle_tree.verify::<Sha3_256>(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_can_contain_an_odd_number_of_transactions() {
        let transactions = vec![String::from("A"), String::from("B"), String::from("C")];
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());

        assert!(merkle_tree.verify::<Sha3_256>(transaction, proof));
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
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());

        assert!(merkle_tree.verify::<Sha3_256>(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_can_add_new_elements() {
        let transactions = vec![String::from("A")];
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());

        assert_eq!(proof.len(), 0);
        assert!(merkle_tree.verify::<Sha3_256>(transaction, proof));

        merkle_tree.add::<Sha3_256>(String::from("B")).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());
        assert_eq!(proof.len(), 1);
        assert!(merkle_tree.verify::<Sha3_256>(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_cant_verify_a_transaction_if_not_present() {
        let transactions = vec![String::from("A"), String::from("B")];
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = String::from("C");
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());

        assert!(!merkle_tree.verify::<Sha3_256>(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_cant_verify_a_transaction_with_different_hasher() {
        let transactions = vec![String::from("A"), String::from("B")];
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());

        assert!(!merkle_tree.verify::<Sha3_384>(transaction, proof));
    }

    #[test]
    fn a_merkle_tree_can_have_generic_transactions() {
        let transactions = vec![vec![10, 20, 30], vec![20, 30, 40], vec![100, 150, 200]];
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = transactions[0].clone();
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction.clone());

        assert!(merkle_tree.verify::<Sha3_256>(transaction, proof));

        let transactions = vec![
            "De aquel amor",
            "De musica ligera",
            "Nada nos libra,",
            "Nada mas queda",
        ];
        let mut merkle_tree = MerkleTree::new::<Sha3_256>(transactions.clone()).unwrap();
        let transaction = transactions[0];
        let proof = merkle_tree.get_proof::<Sha3_256>(transaction);

        assert!(merkle_tree.verify::<Sha3_256>(transaction, proof));
    }
}
