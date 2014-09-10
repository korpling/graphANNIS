#ifndef LOADTEST_H
#define LOADTEST_H

#include "gtest/gtest.h"
#include "db.h"

class LoadTest : public ::testing::Test {
 protected:
  annis::DB db;
  LoadTest() {
    bool result = db.loadRelANNIS("/home/thomas/korpora/pcc/pcc-2/pcc2_v6_relANNIS");
    EXPECT_EQ(true, result);
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

TEST_F(LoadTest, NodeAnnotations) {

  std::vector<annis::Annotation> annos = db.getNodeAnnotationsByID(0);
  ASSERT_EQ(3, annos.size());
  EXPECT_STREQ("tiger", db.string(annos[2].ns).c_str());
  EXPECT_STREQ("lemma", db.string(annos[2].name).c_str());
  EXPECT_STREQ("so", db.string(annos[2].val).c_str());
  EXPECT_STREQ("tiger", db.string(annos[1].ns).c_str());
  EXPECT_STREQ("morph", db.string(annos[1].name).c_str());
  EXPECT_STREQ("--", db.string(annos[1].val).c_str());
  EXPECT_STREQ("tiger", db.string(annos[0].ns).c_str());
  EXPECT_STREQ("pos", db.string(annos[0].name).c_str());
  EXPECT_STREQ("ADV", db.string(annos[0].val).c_str());
}


TEST_F(LoadTest, Edges) {

  // get some edges
  std::vector<annis::Edge> edges = db.getEdgesBetweenNodes(0, 10);
  EXPECT_EQ(1, edges.size());
  EXPECT_EQ(0, edges[0].component);
  EXPECT_EQ(0, edges[0].source);
  EXPECT_EQ(10, edges[0].target);

  edges = db.getEdgesBetweenNodes(126, 371);
  EXPECT_EQ(2, edges.size());
  EXPECT_EQ(156, edges[0].component);
  EXPECT_EQ(185, edges[1].component);
}

TEST_F(LoadTest, EdgeAnnos) {

  std::vector<annis::Edge> edges = db.getEdgesBetweenNodes(126, 371);
  std::vector<annis::Annotation> edgeAnnos = db.getEdgeAnnotations(edges[0]);
  EXPECT_EQ(1, edgeAnnos.size());
  EXPECT_STREQ("tiger", db.string(edgeAnnos[0].ns).c_str());
  EXPECT_STREQ("func", db.string(edgeAnnos[0].name).c_str());
  EXPECT_STREQ("OA", db.string(edgeAnnos[0].val).c_str());

  edgeAnnos = db.getEdgeAnnotations(edges[1]);
  EXPECT_EQ(1, edgeAnnos.size());
  EXPECT_STREQ("tiger", db.string(edgeAnnos[0].ns).c_str());
  EXPECT_STREQ("func", db.string(edgeAnnos[0].name).c_str());
  EXPECT_STREQ("OA", db.string(edgeAnnos[0].val).c_str());
}



#endif // LOADTEST_H
