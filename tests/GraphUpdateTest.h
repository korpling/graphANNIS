#pragma once

#include "gtest/gtest.h"
#include <annis/db.h>
#include <annis/api/graphupdate.h>


using namespace annis;

class GraphUpdateTest : public ::testing::Test {
protected:
  annis::DB db;
  std::string dataDir;
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
    bool loadedDB = db.load(dataDir + "/../relannis/pcc2");
    ASSERT_EQ(true, loadedDB);
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


  ASSERT_EQ(u.getDiffs().size());


}


