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

TEST_F(LoadTest, LoadRelANNIS) {
  annis::DB db;
  bool result = db.loadRelANNIS("/home/thomas/korpora/pcc/pcc-2/pcc2_v6_relANNIS");
  EXPECT_EQ(true, result);
  EXPECT_STREQ("tok_13", db.getNodeByID(0).name.c_str());

  std::vector<annis::Annotation> annos = db.getNodeAnnotationsByID(0);
  ASSERT_EQ(3, annos.size());
  EXPECT_STREQ("tiger", annos[2].ns.c_str());
  EXPECT_STREQ("lemma", annos[2].name.c_str());
  EXPECT_STREQ("so", annos[2].val.c_str());
  EXPECT_STREQ("tiger", annos[1].ns.c_str());
  EXPECT_STREQ("morph", annos[1].name.c_str());
  EXPECT_STREQ("--", annos[1].val.c_str());
  EXPECT_STREQ("tiger", annos[0].ns.c_str());
  EXPECT_STREQ("pos", annos[0].name.c_str());
  EXPECT_STREQ("ADV", annos[0].val.c_str());

  // get some edges
  std::vector<annis::Edge> edges = db.getEdgesBetweenNodes(0, 10);
  EXPECT_EQ(1, edges.size());
  EXPECT_EQ(0, edges[0].component);

}


#endif // LOADTEST_H
