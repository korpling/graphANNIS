use crate::annis::errors::*;
use crate::annis::types::{Edge, NodeID};
use rocksdb::DBRawIterator;
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
        // restrict search to source node prefix
        let prefix: Vec<u8> = source.to_be_bytes().to_vec();
        let it = gs.prefix_iterator(&cf, &prefix);

        let lower_bound = Edge {
            source,
            target: NodeID::min_value(),
        };

        let upper_bound = Edge {
            source,
            target: NodeID::max_value(),
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

    pub fn try_new(
        gs: &'a super::DiskAdjacencyListStorage,
        cf: &'a rocksdb::ColumnFamily,
        source: NodeID,
    ) -> Result<OutgoingEdgesIterator<'a>> {
        // restrict search to source node prefix
        let prefix: Vec<u8> = source.to_be_bytes().to_vec();
        let it = gs.db.prefix_iterator_cf(&cf, &prefix)?;

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

        Ok(OutgoingEdgesIterator {
            raw,
            upper_bound,
            exhausted: false,
        })
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
                                .expect("Key data must be large enough"),
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

pub struct SourceIterator<'a> {
    raw: DBRawIterator<'a>,
    exhausted: bool,
}

impl<'a> SourceIterator<'a> {
    pub fn new(
        gs: &'a super::DiskAdjacencyListStorage,
        cf: &'a rocksdb::ColumnFamily,
    ) -> SourceIterator<'a> {
        let mut opts = rocksdb::ReadOptions::default();
        // Create a forward-only iterator
        opts.set_tailing(true);
        opts.set_verify_checksums(false);

        // restrict search to source node prefix
        let it = gs.iterator_cf_opt_from_start(&cf, &opts);

        let raw: DBRawIterator = it.into();

        SourceIterator {
            raw,
            exhausted: false,
        }
    }
}

impl<'a> Iterator for SourceIterator<'a> {
    type Item = NodeID;

    fn next(&mut self) -> Option<NodeID> {
        while !self.exhausted {
            if self.raw.valid() {
                // get the current item
                let current_source = NodeID::from_be_bytes(
                    self.raw
                        .key()
                        .expect("Valid iterator needs to return valid key")[0..super::NODE_ID_SIZE]
                        .try_into()
                        .expect("Key data must be large enough"),
                );
                // find the next item with a different source node
                let mut at_least_one_outgoing = false;
                self.raw.next();
                while let Some(key) = self.raw.key() {
                    let next_source = NodeID::from_be_bytes(
                        key[0..super::NODE_ID_SIZE]
                            .try_into()
                            .expect("Key data must be large enough"),
                    );
                    if next_source != current_source {
                        break;
                    }
                    at_least_one_outgoing = true;
                    self.raw.next();
                }

                if at_least_one_outgoing {
                    return Some(current_source);
                } else {
                    return None;
                }
            } else {
                self.exhausted = true;
            }
        }
        None
    }
}
