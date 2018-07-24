import sys
import os
sys.path.append(os.path.dirname(os.path.realpath(__file__)) + "/..")

from graphannis.cs import CorpusStorageManager

if __name__ == "__main__":
    with CorpusStorageManager('../data') as cs:
        print(cs.list())

        find_result = cs.find(['GUM'], '{"alternatives":[{"nodes":{"1":{"id":1,"nodeAnnotations":[{"name":"pos","value":"NN","textMatching":"EXACT_EQUAL","qualifiedName":"pos"}],"root":false,"token":false,"variable":"1"}},"joins":[]}]}')

        print(find_result)
