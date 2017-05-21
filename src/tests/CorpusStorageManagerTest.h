/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#pragma once

#include <gtest/gtest.h>
#include <annis/db.h>
#include <annis/api/graphupdate.h>
#include <annis/api/corpusstoragemanager.h>

#include <annis/annosearch/exactannokeysearch.h>

#include <memory>
#include <boost/filesystem.hpp>

#include "testlogger.h"

using namespace annis;

class CorpusStorageManagerTest : public ::testing::Test {
protected:
  std::string dataDir;
  boost::filesystem::path tmpDBPath;
  std::unique_ptr<api::CorpusStorageManager> storageEmpty;
  std::unique_ptr<api::CorpusStorageManager> storageTest;

  CorpusStorageManagerTest()
    : dataDir("data")
  {
  }

  virtual ~CorpusStorageManagerTest() {
    // You can do clean-up work that doesn't throw exceptions here.
  }

  // If the constructor and destructor are not enough for setting up
  // and cleaning up each test, you can define the following methods:

  virtual void SetUp() {
    char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
    if(testDataEnv != NULL)
    {
      dataDir = testDataEnv;
    }

    tmpDBPath = boost::filesystem::unique_path(
            boost::filesystem::temp_directory_path().string() + "/annis-temporary-workspace-%%%%-%%%%-%%%%-%%%%");

    boost::filesystem::create_directories(tmpDBPath);
    HL_INFO(logger, "Using " + tmpDBPath.string() + " as temporary path");


    storageEmpty = std::unique_ptr<api::CorpusStorageManager>(new api::CorpusStorageManager(tmpDBPath.string()));
    ASSERT_EQ(true, (bool) storageEmpty);

    storageTest = std::unique_ptr<api::CorpusStorageManager>(new api::CorpusStorageManager(dataDir));
    ASSERT_EQ(true, (bool) storageTest);

  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(CorpusStorageManagerTest, AddNodeLabel) {



  api::GraphUpdate u;
  u.addNode("node1");
  u.addNodeLabel("node1", "test", "anno", "testVal");

  ASSERT_EQ(2, u.getDiffs().size());


  storageEmpty->applyUpdate("testCorpus", u);


  auto numOfNodes = storageEmpty->count({"testCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"}},\"joins\":[]}]}");
  ASSERT_EQ(1, numOfNodes);

  auto numOfTestAnnos = storageEmpty->count({"testCorpus"},
                                       "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"nodeAnnotations\":[{\"namespace\":\"test\",\"name\":\"anno\",\"value\":\"testVal\",\"textMatching\":\"EXACT_EQUAL\",\"qualifiedName\":\"test:anno\"}],\"root\":false,\"token\":false,\"variable\":\"1\"}},\"joins\":[]}]}");
  ASSERT_EQ(1, numOfTestAnnos);
}

TEST_F(CorpusStorageManagerTest, DeleteNode) {

  api::GraphUpdate updateInsert;
  updateInsert.addNode("node1");
  updateInsert.addNodeLabel("node1", "test", "anno", "testVal");

  storageEmpty->applyUpdate("testCorpus", updateInsert);

  api::GraphUpdate updateDelete;
  updateDelete.deleteNode("node1");
  storageEmpty->applyUpdate("testCorpus", updateDelete);


  auto numOfNodes = storageEmpty->count({"testCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"}},\"joins\":[]}]}");
  ASSERT_EQ(0, numOfNodes);

  auto numOfTestAnnos = storageEmpty->count({"testCorpus"},
                                       "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"nodeAnnotations\":[{\"namespace\":\"test\",\"name\":\"anno\",\"value\":\"testVal\",\"textMatching\":\"EXACT_EQUAL\",\"qualifiedName\":\"test:anno\"}],\"root\":false,\"token\":false,\"variable\":\"1\"}},\"joins\":[]}]}");
  ASSERT_EQ(0, numOfTestAnnos);
}

TEST_F(CorpusStorageManagerTest, AddEdge) {

  api::GraphUpdate updateInsert;
  updateInsert.addNode("node1");
  updateInsert.addNode("node2");
  updateInsert.addEdge("node1", "node2", "", "POINTING", "dep");

  storageEmpty->applyUpdate("testCorpus", updateInsert);

  auto depEdges = storageEmpty->count({"testCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"},\"2\":{\"id\":2,\"root\":false,\"token\":false,\"variable\":\"2\"}},\"joins\":[{\"op\":\"Pointing\",\"name\":\"dep\",\"minDistance\":1,\"maxDistance\":1,\"left\":1,\"right\":2}]}]}");
  ASSERT_EQ(1, depEdges);

  // make sure no edge is found
  auto depEdgesWithAnno = storageEmpty->count({"testCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"},\"2\":{\"id\":2,\"root\":false,\"token\":false,\"variable\":\"2\"}},\"joins\":[{\"op\":\"Pointing\",\"name\":\"dep\",\"minDistance\":1,\"maxDistance\":1,\"edgeAnnotations\":[{\"namespace\":\"ns\",\"name\":\"anno\",\"value\":\"testval\",\"textMatching\":\"EXACT_EQUAL\",\"qualifiedName\":\"ns:anno\"}],\"left\":1,\"right\":2}]}]}");
  ASSERT_EQ(0, depEdgesWithAnno);


}

TEST_F(CorpusStorageManagerTest, AddEdgeLabel) {

  api::GraphUpdate updateInsert;
  updateInsert.addNode("node1");
  updateInsert.addNode("node2");
  updateInsert.addEdge("node1", "node2", "", "POINTING", "dep");
  updateInsert.addEdgeLabel("node1", "node2", "", "POINTING", "dep", "ns", "anno", "testVal");

  storageEmpty->applyUpdate("testCorpus", updateInsert);

  auto depEdgesWithAnno = storageEmpty->count({"testCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"},\"2\":{\"id\":2,\"root\":false,\"token\":false,\"variable\":\"2\"}},\"joins\":[{\"op\":\"Pointing\",\"name\":\"dep\",\"minDistance\":1,\"maxDistance\":1,\"edgeAnnotations\":[{\"namespace\":\"ns\",\"name\":\"anno\",\"value\":\"testVal\",\"textMatching\":\"EXACT_EQUAL\",\"qualifiedName\":\"ns:anno\"}],\"left\":1,\"right\":2}]}]}");
  ASSERT_EQ(1, depEdgesWithAnno);

}

TEST_F(CorpusStorageManagerTest, DeleteEdge) {

  api::GraphUpdate updateInsert;
  updateInsert.addNode("n1");
  updateInsert.addNode("n2");
  updateInsert.addEdge("n1", "n2", "", "POINTING", "dep");
  updateInsert.addNode("n3");
  updateInsert.addNode("n4");
  updateInsert.addEdge("n3", "n4", "", "POINTING", "dep");

  storageEmpty->applyUpdate("testCorpus", updateInsert);

  api::GraphUpdate updateDelete;
  updateDelete.deleteEdge("n1", "n2", "", "POINTING", "dep");

  storageEmpty->applyUpdate("testCorpus", updateDelete);

  auto depEdges = storageEmpty->count({"testCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"},\"2\":{\"id\":2,\"root\":false,\"token\":false,\"variable\":\"2\"}},\"joins\":[{\"op\":\"Pointing\",\"name\":\"dep\",\"minDistance\":1,\"maxDistance\":1,\"left\":1,\"right\":2}]}]}");
  ASSERT_EQ(1, depEdges);

}

TEST_F(CorpusStorageManagerTest, DeleteEdgeLabel) {

  api::GraphUpdate updateInsert;
  updateInsert.addNode("node1");
  updateInsert.addNode("node2");
  updateInsert.addEdge("node1", "node2", "", "POINTING", "dep");
  updateInsert.addEdgeLabel("node1", "node2", "", "POINTING", "dep", "ns", "anno", "testVal");

  storageEmpty->applyUpdate("testCorpus", updateInsert);

  auto depEdgesWithAnno = storageEmpty->count({"testCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"},\"2\":{\"id\":2,\"root\":false,\"token\":false,\"variable\":\"2\"}},\"joins\":[{\"op\":\"Pointing\",\"name\":\"dep\",\"minDistance\":1,\"maxDistance\":1,\"edgeAnnotations\":[{\"namespace\":\"ns\",\"name\":\"anno\",\"value\":\"testVal\",\"textMatching\":\"EXACT_EQUAL\",\"qualifiedName\":\"ns:anno\"}],\"left\":1,\"right\":2}]}]}");
  ASSERT_EQ(1, depEdgesWithAnno);

  api::GraphUpdate updateDelete;
  updateDelete.deleteEdgeLabel("node1", "node2", "", "POINTING", "dep", "ns", "anno");

  storageEmpty->applyUpdate("testCorpus", updateDelete);

  depEdgesWithAnno = storageEmpty->count({"testCorpus"},
                                    "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"},\"2\":{\"id\":2,\"root\":false,\"token\":false,\"variable\":\"2\"}},\"joins\":[{\"op\":\"Pointing\",\"name\":\"dep\",\"minDistance\":1,\"maxDistance\":1,\"edgeAnnotations\":[{\"namespace\":\"ns\",\"name\":\"anno\",\"value\":\"testVal\",\"textMatching\":\"EXACT_EQUAL\",\"qualifiedName\":\"ns:anno\"}],\"left\":1,\"right\":2}]}]}");

  ASSERT_EQ(0, depEdgesWithAnno);
}

TEST_F(CorpusStorageManagerTest, ReloadWithLog) {

  api::GraphUpdate updateInsert;
  updateInsert.addNode("n1");
  updateInsert.addNode("n2");
  updateInsert.addEdge("n1", "n2", "dep", "POINTING", "dep");
  updateInsert.addNode("n3");
  updateInsert.addNode("n4");
  updateInsert.addEdge("n3", "n4", "dep", "POINTING", "dep");

  storageEmpty->applyUpdate("testCorpus", updateInsert);

  auto depEdges = storageEmpty->count({"testCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"},\"2\":{\"id\":2,\"root\":false,\"token\":false,\"variable\":\"2\"}},\"joins\":[{\"op\":\"Pointing\",\"name\":\"dep\",\"minDistance\":1,\"maxDistance\":1,\"left\":1,\"right\":2}]}]}");
  ASSERT_EQ(2, depEdges);

  // save the corpus to a temporary location
  boost::filesystem::path exportPath =
      boost::filesystem::unique_path(
        boost::filesystem::temp_directory_path().string() + "/annis-temporary-export-%%%%-%%%%-%%%%-%%%%");
  storageEmpty->exportCorpus("testCorpus", exportPath.string());

  // reload the same corpus under a different name
  storageEmpty->importCorpus(exportPath.string(), "copyOfTestCorpus");

  // test that the edges are still there
  depEdges = storageEmpty->count({"copyOfTestCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"},\"2\":{\"id\":2,\"root\":false,\"token\":false,\"variable\":\"2\"}},\"joins\":[{\"op\":\"Pointing\",\"name\":\"dep\",\"minDistance\":1,\"maxDistance\":1,\"left\":1,\"right\":2}]}]}");
  ASSERT_EQ(2, depEdges);

}

TEST_F(CorpusStorageManagerTest, SubgraphGUMSingle) {

  std::vector<std::string> ids;
  ids.push_back("GUM/GUM_whow_skittles#tok_936");
  std::vector<api::Node> nodes = storageTest->subgraph("GUM", ids, 5, 5);

  EXPECT_EQ(56, nodes.size());
}


