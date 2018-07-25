import unittest

import sys
import os


from graphannis.cs import CorpusStorageManager
from graphannis import util
import networkx as nx

class TestCorpusStorageManager(unittest.TestCase):
    def setUp(self):
        self.dataDir = os.path.normpath(os.path.realpath(__file__) + '/../../../data')
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

            # convert find results to salt URIs
            match_uris = []
            for m in find_result:
                match_uris.append(util.salt_uri_from_match(m))

            G = cs.subgraph('GUM', match_uris, 5, 5)
            print("\n".join(nx.generate_gml(G)))



if __name__ == '__main__': unittest.main()