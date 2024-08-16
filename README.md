# merkle-tree
Implementation of a Merkle Tree data structure in Rust

## Primitives
- Can be built from a list of elements that implements `AsRef<[u8]>` and `Clone`
- Can generate a proof and validate that an element is in the tree with that proof
- Can add new elementss to the tree
