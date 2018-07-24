import sys
import os

sys.path.append(os.path.dirname(os.path.realpath(__file__)) + "/..")

from graphannis.cs import CorpusStorageManager
import networkx as nx

if __name__ == "__main__":
    with CorpusStorageManager('../data') as cs:
        print(cs.list())

        find_result = cs.find(['GUM'], '{"alternatives":[{"nodes":{"1":{"id":1,"nodeAnnotations":[{"name":"pos","value":"NN","textMatching":"EXACT_EQUAL","qualifiedName":"pos"}],"root":false,"token":false,"variable":"1"}},"joins":[]}]}')

        G = cs.subgraph('GUM', find_result, 5, 5)

        print("\n".join(nx.generate_gml(G)))
