use annis::stringstorage::StringStorage;
use annis::annostorage::AnnoStorage;
use annis::{NodeID, StringID};
use annis::AnnoKey;

const ANNIS_NS : &str = "annis";
const NODE_NAME : &str = "node_name";
const TOK : &str = "tok";
const NODE_TYPE : &str = "node_type";

pub struct GraphDB {
    pub strings: StringStorage,
    pub node_annos: AnnoStorage<NodeID>,

    id_annis_ns: StringID,
    id_node_name: StringID,
    id_tok : StringID,
    id_node_type : StringID,
}

impl GraphDB {
    /**
     * Create a new and empty instance.
     */
    pub fn new() -> GraphDB {
        let mut strings = StringStorage::new();

        GraphDB {
            id_annis_ns : strings.add(ANNIS_NS),
            id_node_name : strings.add(NODE_NAME),
            id_tok : strings.add(TOK),
            id_node_type : strings.add(NODE_TYPE),
            
            strings,
            node_annos: AnnoStorage::<NodeID>::new(),
        }
    }

    pub fn get_token_key(&self) -> AnnoKey {
        AnnoKey{ns : self.id_annis_ns, name: self.id_tok}
    }
}
