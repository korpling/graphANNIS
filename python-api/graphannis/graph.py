# Gives access to a (sub)-graph stored in graphANNIS and allows to map it to networkX

import networkx as nx
from graphannis import CAPI, ffi

def map_node(n_id):
    n = dict()
    n["test"] = "example"
    return n

def map_graph(db):
    G = nx.DiGraph()
    if db == ffi.NULL:
        return G
        
    # create all new nodes
    it_nodes = CAPI.annis_graph_nodes_by_type(db, b'nodes')
    print(it_nodes)
    if it_nodes != ffi.NULL:
        n_id = CAPI.annis_iter_nodeid_next(it_nodes)
        print(it_nodes)
        while n_id != ffi.NULL:

            G.add_node(map_node(n_id))

            n_id = CAPI.annis_iter_nodeid_next(it_nodes)

    return G