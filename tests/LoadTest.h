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
  ASSERT_EQ(4, annos.size());

  EXPECT_STREQ(annis::annis_ns.c_str(), db.str(annos[3].ns).c_str());
  EXPECT_STREQ("tok", db.str(annos[3].name).c_str());
  EXPECT_STREQ("so", db.str(annos[3].val).c_str());

  EXPECT_STREQ("tiger", db.str(annos[2].ns).c_str());
  EXPECT_STREQ("lemma", db.str(annos[2].name).c_str());
  EXPECT_STREQ("so", db.str(annos[2].val).c_str());

  EXPECT_STREQ("tiger", db.str(annos[1].ns).c_str());
  EXPECT_STREQ("morph", db.str(annos[1].name).c_str());
  EXPECT_STREQ("--", db.str(annos[1].val).c_str());


  EXPECT_STREQ("tiger", db.str(annos[0].ns).c_str());
  EXPECT_STREQ("pos", db.str(annos[0].name).c_str());
  EXPECT_STREQ("ADV", db.str(annos[0].val).c_str());



}


TEST_F(LoadTest, Edges) {

  // get some edges
  std::vector<annis::Component> components = db.getDirectConnected(annis::Edge(0, 10));
  ASSERT_EQ(1, components.size());
  EXPECT_EQ(annis::ComponentType::COVERAGE, components[0].type);
  EXPECT_STREQ("exmaralda", components[0].ns);
  EXPECT_STREQ("", components[0].name);

  components = db.getDirectConnected(annis::Edge(126, 371));
  ASSERT_EQ(2, components.size());

  EXPECT_EQ(annis::ComponentType::DOMINANCE, components[0].type);
  EXPECT_STREQ("tiger", components[0].ns);
  EXPECT_STREQ("", components[0].name);

  EXPECT_EQ(annis::ComponentType::DOMINANCE, components[1].type);
  EXPECT_STREQ("tiger", components[1].ns);
  EXPECT_STREQ("edge", components[1].name);
}


TEST_F(LoadTest, EdgeAnnos) {

  annis::Edge edge(126, 371);
  std::vector<annis::Component> components = db.getDirectConnected(edge);

  ASSERT_EQ(2, components.size());

  std::vector<annis::Annotation> edgeAnnos = db.getEdgeAnnotations(components[0], edge);
  EXPECT_EQ(1, edgeAnnos.size());
  EXPECT_STREQ("tiger", db.str(edgeAnnos[0].ns).c_str());
  EXPECT_STREQ("func", db.str(edgeAnnos[0].name).c_str());
  EXPECT_STREQ("OA", db.str(edgeAnnos[0].val).c_str());

  edgeAnnos = db.getEdgeAnnotations(components[1], edge);
  EXPECT_EQ(1, edgeAnnos.size());
  EXPECT_STREQ("tiger", db.str(edgeAnnos[0].ns).c_str());
  EXPECT_STREQ("func", db.str(edgeAnnos[0].name).c_str());
  EXPECT_STREQ("OA", db.str(edgeAnnos[0].val).c_str());
}



#endif // LOADTEST_H
