use tiny_keccak::{Hasher, Keccak};

type Bytes32 = [u8; 32];

#[derive(Debug)]
struct Node {
    hash: Bytes32,
    transfer_id: Bytes32,
}

#[derive(Debug)]
pub struct Tree {
    scratch: Vec<Bytes32>,
    leaves: Vec<Node>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            scratch: Vec::new(),
            leaves: Vec::new(),
        }
    }
    // TODO: The transfer_id doesn't need to be passed in separately.
    // We know the encoding, so we know how to extract it from the data.
    pub fn insert(&mut self, data: &str, transfer_id: Bytes32) {
        let hash = keccak(data);
        let node = Node { hash, transfer_id };
        let index = match self
            .leaves
            .binary_search_by_key(&&node.transfer_id, |n| &n.transfer_id)
        {
            Ok(i) => i,
            Err(i) => i,
        };
        self.leaves.insert(index, node);
    }

    /// It is intentional that this method is separate from insert/delete.
    /// One expected use-case is to insert, calculate a new hash, propose an
    /// update, fail, and finally need to roll back. To roll back the best thing to
    /// do is just to delete without calculating the root.
    pub fn root(&mut self) -> (Bytes32, usize) {
        // TODO: (Performance)
        // This is another good case for second_stack to avoid the &mut self
        // and all the extra memory this requires to have separate scratches
        // per merkle tree.
        while self.scratch.len() < self.leaves.len() {
            self.scratch.push(Default::default())
        }

        let mut num_hashes = 0;

        // In principle this is about the same number of hashes.
        let mut scratch = &mut self.scratch[..self.leaves.len()];

        // TODO: In the final version can specialize the copy logic
        for i in 0..scratch.len() {
            scratch[i] = self.leaves[i].hash;
        }

        while scratch.len() > 1 {
            let mut write = 0;
            let mut read = 0;
            while read + 1 < scratch.len() {
                let a = scratch[read];
                let b = scratch[read + 1];
                read += 2;
                num_hashes += 1;
                scratch[write] = combine(&a, &b);
                write += 1;
            }

            scratch = &mut scratch[0..write];
        }

        (scratch[0], num_hashes)
    }
}

fn keccak(data: &str) -> Bytes32 {
    let mut hash = [0; 32];
    let mut hasher = Keccak::v256();
    hasher.update(data.as_bytes());
    hasher.finalize(&mut hash);
    hash
}

fn combine(a: &Bytes32, b: &Bytes32) -> Bytes32 {
    // TODO: Sort a, b correctly
    let (first, second) = if a > b { (a, b) } else { (b, a) };

    let mut keccak256 = Keccak::v256();
    keccak256.update(first);
    keccak256.update(second);
    let mut into = [0; 32];
    keccak256.finalize(&mut into);
    into
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn speed() {
        let start = Instant::now();
        let mut tree = Tree::new();
        let mut break_optimizer = [0; 32];
        let mut num_hashes = 0;
        for i in 0..2000 {
            // TODO: Encoded data of correct length.
            // This oversight shouldn't affect the result much,
            // because data is only hashed once.
            let data = i.to_string().repeat(1000);
            let mut app_id = [0; 32];
            app_id[0] = i as u8;
            tree.insert(&data, app_id);
            let (root, c) = tree.root();
            num_hashes += c;
            break_optimizer[i % 32] += root[i % 32];
        }
        dbg!(Instant::now() - start);
        dbg!(break_optimizer[0]);
        dbg!(num_hashes);
    }
}
