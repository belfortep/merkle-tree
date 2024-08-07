use std::hash::{DefaultHasher, Hash, Hasher};

use merkle_tree::{self, merkle::merkle_tree::MerkleTree};

fn main() {
    let transactions = vec![String::from("A"), String::from("B")];
    let mut merkle_tree = MerkleTree::new(transactions.clone()).unwrap();
    let transaction = transactions[0].clone();
    let another_transaction = transactions[1].clone();
    let proof = merkle_tree.get_proof(another_transaction.clone()).unwrap();
    let mut hasher = DefaultHasher::new();

    for p in &proof {
        println!("en proof: {}", p);
    }

    let mut hasher = DefaultHasher::new();
    transaction.hash(&mut hasher);
    let hash_de_a = hasher.finish();
    println!("A solo vale: {}", hash_de_a);

    let mut hasher = DefaultHasher::new();
    another_transaction.hash(&mut hasher);
    let hash_de_b = hasher.finish();
    println!("B solo vale: {}", hash_de_b);

    let mut hasher = DefaultHasher::new();
    hash_de_a.hash(&mut hasher);
    hasher.write_u64(hash_de_b);
    println!("Dou? : {}", hasher.finish());

    let mut hasher = DefaultHasher::new();
    hash_de_a.hash(&mut hasher);
    hash_de_b.hash(&mut hasher);
    println!("hashear A y B da {}", hasher.finish());

    println!("{}", merkle_tree.verify(String::from("B"), proof))
}
