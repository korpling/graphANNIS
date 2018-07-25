import unittest

import sys
import os


from graphannis.cs import CorpusStorageManager
import networkx as nx

class TestCorpusStorageManager(unittest.TestCase):
    def setUp(self):
        self.dataDir = os.path.normpath(os.path.realpath(__file__) + '/../../../data')
        print(self.dataDir)
        # TODO load data if not test corpus does not exist yet
        
    def test_list(self):
        with CorpusStorageManager(self.dataDir) as cs:
            corpora = cs.list()
            assert(isinstance(corpora, list))

    def test_find(self):
        with CorpusStorageManager(self.dataDir) as cs:
            find_result = cs.find(['GUM'], '{"alternatives":[{"nodes":{"1":{"id":1,"nodeAnnotations":[{"name":"pos","value":"NN","textMatching":"EXACT_EQUAL","qualifiedName":"pos"}],"root":false,"token":false,"variable":"1"}},"joins":[]}]}')
            assert(isinstance(find_result, list))

            assert(len(find_result) > 0)

            G = cs.subgraph('GUM', find_result, 5, 5)
            print("\n".join(nx.generate_gml(G)))



if __name__ == '__main__': unittest.main()