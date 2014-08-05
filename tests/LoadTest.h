#ifndef LOADTEST_H
#define LOADTEST_H

#include "gtest/gtest.h"
#include "db.h"

class LoadTest : public ::testing::Test {
 protected:

  LoadTest() {
    // You can do set-up work for each test here.
  }

  virtual ~LoadTest() {
    // You can do clean-up work that doesn't throw exceptions here.
  }

  // If the constructor and destructor are not enough for setting up
  // and cleaning up each test, you can define the following methods:

  virtual void SetUp() {
    // Code here will be called immediately after the constructor (right
    // before each test).
  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(LoadTest, DummyReturnsTrue) {
  annis::DB db;
  bool result = db.loadNodeStorage("/home/thomas/korpora/pcc/pcc-2/pcc2_v6_relANNIS");
  EXPECT_EQ(true, result);
  std::string name = db.getNodeByID(0).name;
  EXPECT_EQ("tok_13", name);
}


#endif // LOADTEST_H
