use crate::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn from_idx(index: usize) -> Self {
        match index % 2 {
            0 => Self::Left,
            _ => Self::Right,
        }
    }

    pub fn sibling(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Default)]
pub struct MerkleTree {
    layers: Vec<Vec<Hash>>,
}

pub struct MerkleProof {
    leaf_index: usize,
    siblings: Vec<(Hash, Side)>,
}

impl MerkleTree {
    pub fn root(&self) -> Hash {
        self.layers
            .last()
            .expect("MerkleTree always has at least one layer")[0]
    }

    fn hash_pair(left: &Hash, right: &Hash) -> Hash {
        let mut concatenated_hash = [0u8; 64];
        concatenated_hash[..32].copy_from_slice(left.as_bytes());
        concatenated_hash[32..].copy_from_slice(right.as_bytes());
        return Hash::of(&concatenated_hash);
    }

    fn next_layer(layer: &[Hash]) -> Vec<Hash> {
        let mut new_layer = Vec::new();

        for chunk in layer.chunks(2) {
            match chunk {
                [left, right] => new_layer.push(Self::hash_pair(left, right)),
                [single] => new_layer.push(single.clone()),
                _ => unreachable!(),
            }
        }

        return new_layer;
    }

    pub fn build(leaves: &[Hash]) -> Self {
        // If empty return sentinel tree
        if leaves.is_empty() {
            return Self {
                layers: vec![vec![Hash::of(b"")]],
            };
        }

        let mut layers = vec![leaves.to_vec()];

        while layers.last().unwrap().len() != 1 {
            let cur_layer = layers.last().unwrap();
            let next_layer = Self::next_layer(cur_layer);
            layers.push(next_layer);
        }

        Self { layers }
    }

    pub fn prove(&self, leaf_index: usize) -> MerkleProof {
        assert!(leaf_index < self.layers[0].len(), "Leaf is out of index");
        let mut siblings: Vec<(Hash, Side)> = Vec::new();

        let mut cur_leaf_idx = leaf_index;
        for cur_layer in &self.layers[..self.layers.len() - 1] {
            let cur_leaf_side = Side::from_idx(cur_leaf_idx);
            let sibling_leaf_idx = match cur_leaf_side {
                Side::Left => cur_leaf_idx + 1,
                Side::Right => cur_leaf_idx - 1,
            };

            if sibling_leaf_idx >= cur_layer.len() {
                cur_leaf_idx /= 2;
                continue;
            }

            siblings.push((cur_layer[sibling_leaf_idx], cur_leaf_side.sibling()));
            cur_leaf_idx /= 2;
        }

        MerkleProof {
            leaf_index,
            siblings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaves_from(strs: &[&str]) -> Vec<Hash> {
        strs.iter().map(|s| Hash::of(s.as_bytes())).collect()
    }

    #[test]
    fn test_merkle_tree_empty_leaves_returns_default() {
        let empty_leaves: Vec<Hash> = Vec::new();
        let merkle_tree = MerkleTree::build(&empty_leaves);
        assert_eq!(merkle_tree.root(), Hash::of(b""));
    }

    #[test]
    fn test_merkle_tree_single_leaf_root_is_the_leaf() {
        let leaf = Hash::of(b"tx1");
        let merkle_tree = MerkleTree::build(&[leaf]);
        assert_eq!(merkle_tree.root(), leaf);
    }

    #[test]
    fn test_merkle_tree_four_leaves_root_differs_from_any_leaf() {
        let leaves = leaves_from(&["tx1", "tx2", "tx3", "tx4"]);

        let merkle_tree = MerkleTree::build(&leaves);
        let root = merkle_tree.root();

        for leaf in &leaves {
            assert_ne!(root, *leaf);
        }
    }

    #[test]
    fn test_merkle_tree_changing_one_leaf_changes_root() {
        let leaves_a = leaves_from(&["tx1", "tx2", "tx3", "tx4"]);
        let leaves_b = leaves_from(&["tx1", "tx2", "tx3", "tampered"]);

        let root_a = MerkleTree::build(&leaves_a).root();
        let root_b = MerkleTree::build(&leaves_b).root();

        assert_ne!(root_a, root_b);
    }

    #[test]
    fn test_merkle_tree_same_leaves_same_root() {
        let leaves = leaves_from(&["tx1", "tx2", "tx3"]);

        let root_a = MerkleTree::build(&leaves).root();
        let root_b = MerkleTree::build(&leaves).root();

        assert_eq!(root_a, root_b);
    }

    #[test]
    fn test_prove_single_leaf_has_no_siblings() {
        let leaves = leaves_from(&["tx1"]);
        let tree = MerkleTree::build(&leaves);

        let proof = tree.prove(0);

        assert!(proof.siblings.is_empty());
    }

    #[test]
    fn test_prove_four_leaves_proof_length_matches_tree_height() {
        let leaves = leaves_from(&["tx1", "tx2", "tx3", "tx4"]);
        let tree = MerkleTree::build(&leaves);

        for i in 0..leaves.len() {
            let proof = tree.prove(i);
            assert_eq!(proof.siblings.len(), 2, "leaf {i} should need 2 siblings");
        }
    }

    #[test]
    fn test_prove_four_leaves_returns_expected_siblings_for_index_2() {
        let leaves = leaves_from(&["tx1", "tx2", "tx3", "tx4"]);
        let tree = MerkleTree::build(&leaves);
        let layer1_left_sibling = MerkleTree::hash_pair(&leaves[0], &leaves[1]);

        let proof = tree.prove(2);

        assert_eq!(
            proof.siblings,
            vec![(leaves[3], Side::Right), (layer1_left_sibling, Side::Left)]
        );
    }

    #[test]
    fn test_prove_promoted_leaf_of_odd_tree_has_one_sibling() {
        let leaves = leaves_from(&["tx1", "tx2", "tx3"]);
        let tree = MerkleTree::build(&leaves);
        let layer1_left_sibling = MerkleTree::hash_pair(&leaves[0], &leaves[1]);

        // leaves[2] is the lone unpaired leaf, promoted unchanged into layer 1.
        let proof = tree.prove(2);

        assert_eq!(proof.siblings, vec![(layer1_left_sibling, Side::Left)]);
    }
}
