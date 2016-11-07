#pragma once

#include <gtest/gtest.h>
#include <annis/db.h>
#include <annis/api/graphupdate.h>

#include <annis/query.h>
#include <annis/annosearch/exactannokeysearch.h>

using namespace annis;

class GraphUpdateTest : public ::testing::Test {
protected:
  std::string dataDir;
  annis::DB db;

  GraphUpdateTest()
    : dataDir("data")
  {
  }

  virtual ~GraphUpdateTest() {
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
//    bool loadedDB = db.load(dataDir + "/../relannis/pcc2");
//    ASSERT_EQ(true, loadedDB);
  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(GraphUpdateTest, DiffSize) {

  api::GraphUpdate u;
  u.addNode("node1");
  u.addNodeLabel("node1", "test", "anno", "testVal");
  u.finish();

  ASSERT_EQ(2, u.getDiffs().size());

  db.update(u);

  annis::Query q(db);
  q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_node_name));

  size_t counter = 0;
  while(q.next())
  {
     counter++;
  }
  ASSERT_EQ(1, counter);

}


