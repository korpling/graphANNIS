use crate::annis::types::{Edge, NodeID};
use rocksdb::{DBRawIterator};
use std::convert::TryInto;

pub struct OutgoingEdgesIterator<'a> {
    raw: DBRawIterator<'a>,
    upper_bound: Vec<u8>,
    exhausted: bool,
}

impl<'a> OutgoingEdgesIterator<'a> {
    pub fn new(
        gs: &'a super::DiskAdjacencyListStorage,
        cf: &'a rocksdb::ColumnFamily,
        source: NodeID,
    ) -> OutgoingEdgesIterator<'a> {
        let mut opts = rocksdb::ReadOptions::default();
        // Create a forward-only iterator
        opts.set_tailing(true);
        opts.set_verify_checksums(false);

        // restrict search to source node prefix
        let prefix: Vec<u8> = source.to_be_bytes().to_vec();
        let it = gs.prefix_iterator(&cf, &prefix);

        let lower_bound = Edge {
            source,
            target: NodeID::min_value(),
        };

        let upper_bound = Edge {
            source,
            target: NodeID::min_value(),
        };

        let lower_bound = super::create_key(&lower_bound);
        let upper_bound = super::create_key(&upper_bound);

        let mut raw: DBRawIterator = it.into();

        raw.seek(lower_bound);

        OutgoingEdgesIterator {
            raw,
            upper_bound,
            exhausted: false,
        }
    }
}

impl<'a> Iterator for OutgoingEdgesIterator<'a> {
    type Item = NodeID;

    fn next(&mut self) -> Option<NodeID> {
        if !self.exhausted {
            if self.raw.valid() {
                // get the current item
                if let Some(key) = self.raw.key() {
                    // check if item has reached the upper bound
                    if key < &self.upper_bound[..] {
                        // parse the node ID from this item
                        let outgoing_id = NodeID::from_be_bytes(
                            key[(key.len() - super::NODE_ID_SIZE)..]
                                .try_into()
                                .expect(&format!(
                                    "Key data must at least have length {}",
                                    super::NODE_ID_SIZE
                                )),
                        );
                        // set iterator to next item
                        self.raw.next();

                        return Some(outgoing_id);
                    } else {
                        // iterator is exhausted: make sure that raw.next() is not called again
                        self.exhausted = true;
                    }
                }
            } else {
                self.exhausted = true;
            }
        }
        None
    }
}
