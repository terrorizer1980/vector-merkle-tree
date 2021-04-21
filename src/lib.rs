type Bytes32 = [u8; 32];

mod error;
mod format;
mod hash;

//#[cfg(test)]
mod test_utils;

pub use error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Node {
    hash: Bytes32,
    transfer_id: Bytes32,
}

#[derive(Debug, Clone)]
pub struct Tree {
    // TODO: (Performance)
    // This scratch is way too big. If using depth-first traversal
    // only 2 * log2(n) values would be needed at any time.
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

    fn insert_node(&mut self, node: Node) -> Result<(), Error> {
        let index = match self
            .leaves
            .binary_search_by_key(&&node.transfer_id, |n| &n.transfer_id)
        {
            // This structure cannot handle a duplicate transfer ID because there
            // would not be one canonical ordering.
            Ok(_) => return Err(Error::DuplicateTransferID),
            Err(i) => i,
        };
        self.leaves.insert(index, node);
        Ok(())
    }

    pub fn insert_hex(&mut self, core_transfer_state: &str) -> Result<(), Error> {
        let node = format::hex_to_node(core_transfer_state)?;
        self.insert_node(node)
    }

    /// It is intentional that this method is separate from insert/delete.
    /// One expected use-case is to insert, calculate a new hash, propose an
    /// update, fail, and finally need to roll back. To roll back the best thing to
    /// do is just to delete without calculating the root.
    pub fn root(&mut self) -> Bytes32 {
        if self.leaves.len() == 0 {
            return Default::default();
        }

        // TODO: (Performance)
        // This is another good case for second_stack to avoid the &mut self
        // and all the extra memory this requires to have separate scratches
        // per merkle tree.
        while self.scratch.len() < self.leaves.len() {
            self.scratch.push(Default::default())
        }

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
                scratch[write] = hash::combine(&a, &b);
                write += 1;
            }
            if read < scratch.len() {
                scratch[write] = scratch[read];
                write += 1;
            }

            scratch = &mut scratch[0..write];
        }

        scratch[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::Instant;
    use test_utils::*;

    #[test]
    #[ignore]
    fn speed() {
        let start = Instant::now();
        let mut tree = Tree::new();
        let mut break_optimizer = [0u32; 32];
        for i in 0u32..2000 {
            let mut data = "0x000000000000000000000000ccc00000000000000000000000000000000000000de5846e2e915d4bad7b8fbb3822b932367a24691960d57cd983257037553411000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000".to_owned();
            for c in 66..130 {
                let mut hasher = DefaultHasher::new();
                (i, c).hash(&mut hasher);
                let h = (hasher.finish() % 10) + 48;
                unsafe { data.as_bytes_mut()[c] = h as u8 };
            }
            tree.insert_hex(&data)
                .expect("Duplicate transfer ids unlikely");
            let root = tree.root();
            let index = (i % 32) as usize;
            let opt = &mut break_optimizer[index];
            *opt = opt.wrapping_add(root[index] as u32);
        }
        dbg!(Instant::now() - start);
        dbg!(break_optimizer);
    }

    // Ensures that the result is the same as before.
    #[test]
    fn backward_compatible() {
        let encoded_transfers = [
            "0x000000000000000000000000ccc000000000000000000000000000000000000005549d00942c85d5004b75e5cd02acce4f330a7be6a8f6c5a1fabbf5b4cdd828000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc000000000000000000000000000000000000007ace82c0553bb5ca2aa65a36575e03c7f828862331d1c46dfb6670e4c4df42e000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc000000000000000000000000000000000000009f144821123db1bfc74c7ee6cbc8d165f5811c1e3e7ba7fe712c94fd8269c48000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc00000000000000000000000000000000000000dfe90e4da4d7751ed146dda5ddab63b79e9b289a1e5a8df3043c6f310d048e4000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc00000000000000000000000000000000000003299fc09866576090f3aac16947ac1d5609643de11bac44e1078f46c1d3e2d88000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc000000000000000000000000000000000000046e175ab10c6eda80a359597463ddab7964cde65aab637841ca4dae99cc70a03000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc00000000000000000000000000000000000004c58e396ee023abc1527ed314f5a8d5f400a3368db05fe94912052739bcc76c7000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc00000000000000000000000000000000000005dcb4070cac9caf98dc6d52e8c894d66ca56a604fddc5997e12b68ef3d157601000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc0000000000000000000000000000000000000601d0c7b09a5497afb3b40af416db9849b5aedccc28db446e5a4c820febe9f1e000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc0000000000000000000000000000000000000801577b9f75b982587cf3990dfc64f470737e8b541ea584d1a5f5256b4a93142000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc00000000000000000000000000000000000008aabe587d75461d49fa307817b2d8838a491f2c05651df8524ccd92bf452872d000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc00000000000000000000000000000000000009137e2f6cb80b96f774f8ed2373994541f66a8689f447087e69206427da05515000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc00000000000000000000000000000000000009552d4fead25e7e1e3b5ade019dcd6af4512f88872c6969b4d46022dbcc78284000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc00000000000000000000000000000000000009f85649dfee68d6fa25ec750fc8a0acb8249a35f2ec6fd9636d1d1ef03eaf2b3000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc0000000000000000000000000000000000000a3dcf8776fa5875dc396ab38acfe7281fa9ecf436f5b2726842c4b2d5fbcadc4000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc0000000000000000000000000000000000000a57a4368df0abc7d96913bc3f17979db404218797cc821ccaf290ee1fe585cbf000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc0000000000000000000000000000000000000aa29aab59197f98b58737c083159bf03c601671341d9022681ba17c4db88fb32000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc0000000000000000000000000000000000000ac5dc7ee2f3ee59658fdf9566663329ac102e66d19d629b5c3ba9af21aec9ffe000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc0000000000000000000000000000000000000dcd3135bc272cec12f4247fa064251e200c53c9ed6be8e644f8a1ba650c1a676000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
            "0x000000000000000000000000ccc0000000000000000000000000000000000000e3efadc64aeb650945b72bac219c1633a632dc7d77925359b7e546cb88fa7f43000000000000000000000000def0000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000bbb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001abcdef0000000000000000000000000000000000000000000000000000000000",
        ];

        let results = [
            "f904cda0a4dfdbe1e6e2d48a131810dc8d62f2af02f04a940f792fea44b7bdb2",
            "6ac2b38459206338b7da7f5c85578e5a7ca8c02d5e541b17a0a4769b792c76ab",
            "f0636436ea54a4bb432238f704252cc2f73d45c801ae7aecd10b105d32bc7827",
            "d3691afa76c335f1e8016a9efad192a7c4d3d9931958b07dc1482e2d2fd191c8",
            "6cf598cc2d7be3ade6ac4182661a3de96f4242537d9a9573847ecc1ea9363eb4",
            "04df001aad685cd1a6538e28a62d12d5e5af759a70ffda3c053f8a6aa7f997fd",
            "5e299452c6ecfe23f19f89d3eb9348fd271bdaa1292fb92941a488a1dfa8120e",
            "94ffc901bfe33c04a3877277fc443ef0787e80734fa6d48a5261175d6bf2b4e5",
            "2745ecce64da66937b26ecc9a4927709ed72f2b0c59c0137519f3d9abe277e39",
            "17aaf4493bfb1371236dff66f52f98aed31a8fe2a1c03a8b3b6cb97a64038fff",
        ];

        let mut tree = Tree::new();
        for i in 0..10 {
            tree.insert_hex(encoded_transfers[i])
                .expect("Transfer id should be unique");
            let root = hex_encode(tree.root());
            assert_eq!(root, results[i]);
        }
    }

    #[test]
    fn empty_set() {
        let mut tree = Tree::new();
        let root = tree.root();
        assert_eq!(
            hex_encode(root),
            "0000000000000000000000000000000000000000000000000000000000000000"
        )
    }
}
