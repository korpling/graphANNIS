# Gives access to a (sub)-graph stored in graphANNIS and allows to map it to networkX

import networkx as nx
from graphannis import CAPI

def map_graph(db):
    G = nx.DiGraph()

    # TODO: do the mapping
    G.add_node(1)
    G.add_node(2)
    G.add_edge(1,2)

    return G