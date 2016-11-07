#pragma once

#include <gtest/gtest.h>
#include <annis/db.h>
#include <annis/api/graphupdate.h>
#include <annis/api/corpusstorage.h>

#include <annis/query.h>
#include <annis/annosearch/exactannokeysearch.h>

#include <memory>
#include <boost/filesystem.hpp>

#include <humblelogging/api.h>

using namespace annis;

class CorpusStorageTest : public ::testing::Test {
protected:
  std::string dataDir;
  boost::filesystem::path tmpDBPath;
  std::unique_ptr<api::CorpusStorage> storage;

  CorpusStorageTest()
    : dataDir("data")
  {
  }

  virtual ~CorpusStorageTest() {
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


    storage = std::unique_ptr<api::CorpusStorage>(new api::CorpusStorage(tmpDBPath.string()));
    ASSERT_EQ(true, (bool) storage);

  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(CorpusStorageTest, AddNodeLabel) {



  api::GraphUpdate u;
  u.addNode("node1");
  u.addNodeLabel("node1", "test", "anno", "testVal");
  u.finish();

  ASSERT_EQ(2, u.getDiffs().size());


  storage->applyUpdate("testCorpus", u);


  auto numOfNodes = storage->count({"testCorpus"},
                                  "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"root\":false,\"token\":false,\"variable\":\"1\"}},\"joins\":[]}]}");
  ASSERT_EQ(1, numOfNodes);

  auto numOfTestAnnos = storage->count({"testCorpus"},
                                       "{\"alternatives\":[{\"nodes\":{\"1\":{\"id\":1,\"nodeAnnotations\":[{\"namespace\":\"test\",\"name\":\"anno\",\"value\":\"testVal\",\"textMatching\":\"EXACT_EQUAL\",\"qualifiedName\":\"test:anno\"}],\"root\":false,\"token\":false,\"variable\":\"1\"}},\"joins\":[]}]}");
  ASSERT_EQ(1, numOfTestAnnos);
}


