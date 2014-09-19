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
  ASSERT_EQ(5, annos.size());

  EXPECT_STREQ(annis::annis_ns.c_str(), db.strings.str(annos[4].ns).c_str());
  EXPECT_STREQ("node_name", db.strings.str(annos[4].name).c_str());
  EXPECT_STREQ("tok_13", db.strings.str(annos[4].val).c_str());

  EXPECT_STREQ(annis::annis_ns.c_str(), db.strings.str(annos[3].ns).c_str());
  EXPECT_STREQ("tok", db.strings.str(annos[3].name).c_str());
  EXPECT_STREQ("so", db.strings.str(annos[3].val).c_str());

  EXPECT_STREQ("tiger", db.strings.str(annos[2].ns).c_str());
  EXPECT_STREQ("lemma", db.strings.str(annos[2].name).c_str());
  EXPECT_STREQ("so", db.strings.str(annos[2].val).c_str());

  EXPECT_STREQ("tiger", db.strings.str(annos[1].ns).c_str());
  EXPECT_STREQ("morph", db.strings.str(annos[1].name).c_str());
  EXPECT_STREQ("--", db.strings.str(annos[1].val).c_str());


  EXPECT_STREQ("tiger", db.strings.str(annos[0].ns).c_str());
  EXPECT_STREQ("pos", db.strings.str(annos[0].name).c_str());
  EXPECT_STREQ("ADV", db.strings.str(annos[0].val).c_str());

}


TEST_F(LoadTest, Edges) {

  // get some edges
  std::vector<annis::Component> components = db.getDirectConnected(annis::constructEdge(0, 10));
  ASSERT_EQ(1, components.size());
  EXPECT_EQ(annis::ComponentType::COVERAGE, components[0].type);
  EXPECT_STREQ("exmaralda", components[0].layer);
  EXPECT_STREQ("", components[0].name);

  components = db.getDirectConnected(annis::constructEdge(126, 371));
  ASSERT_EQ(2, components.size());

  EXPECT_EQ(annis::ComponentType::DOMINANCE, components[0].type);
  EXPECT_STREQ("tiger", components[0].layer);
  EXPECT_STREQ("", components[0].name);

  EXPECT_EQ(annis::ComponentType::DOMINANCE, components[1].type);
  EXPECT_STREQ("tiger", components[1].layer);
  EXPECT_STREQ("edge", components[1].name);
}


TEST_F(LoadTest, EdgeAnnos) {

  annis::Edge edge = annis::constructEdge(126, 371);
  std::vector<annis::Component> components = db.getDirectConnected(edge);

  ASSERT_EQ(2, components.size());

  std::vector<annis::Annotation> edgeAnnos = db.getEdgeAnnotations(components[0], edge);
  EXPECT_EQ(1, edgeAnnos.size());
  EXPECT_STREQ("tiger", db.strings.str(edgeAnnos[0].ns).c_str());
  EXPECT_STREQ("func", db.strings.str(edgeAnnos[0].name).c_str());
  EXPECT_STREQ("OA", db.strings.str(edgeAnnos[0].val).c_str());

  edgeAnnos = db.getEdgeAnnotations(components[1], edge);
  EXPECT_EQ(1, edgeAnnos.size());
  EXPECT_STREQ("tiger", db.strings.str(edgeAnnos[0].ns).c_str());
  EXPECT_STREQ("func", db.strings.str(edgeAnnos[0].name).c_str());
  EXPECT_STREQ("OA", db.strings.str(edgeAnnos[0].val).c_str());
}

TEST_F(LoadTest, Ordering) {

  annis::Component componentOrdering = annis::constructComponent(annis::ComponentType::ORDERING,
                                                 annis::annis_ns, "");
  const annis::EdgeDB* edb = db.getEdgeDB(componentOrdering);
  ASSERT_TRUE(edb != NULL);
  // tok . tok
  EXPECT_TRUE(edb->isConnected(annis::constructEdge(0, 1)));

  // test the last two token
  EXPECT_TRUE(edb->isConnected(annis::constructEdge(517, 880)));

  // span . tok
  EXPECT_FALSE(edb->isConnected(annis::constructEdge(125, 126)));
  // tok . span
  EXPECT_FALSE(edb->isConnected(annis::constructEdge(151, 61)));
  // span . span
  EXPECT_FALSE(edb->isConnected(annis::constructEdge(152, 61)));

  annis::Component componentLeftToken = annis::constructComponent(annis::ComponentType::LEFT_TOKEN,
                                                 annis::annis_ns, "");
  edb = db.getEdgeDB(componentLeftToken);
  ASSERT_TRUE(edb != NULL);
  // span _l_ tok
  EXPECT_TRUE(edb->isConnected(annis::constructEdge(125, 124)));
  EXPECT_TRUE(edb->isConnected(annis::constructEdge(61, 49)));

  annis::Component componentRightToken = annis::constructComponent(annis::ComponentType::RIGHT_TOKEN,
                                                 annis::annis_ns, "");
  edb = db.getEdgeDB(componentRightToken);
  ASSERT_TRUE(edb != NULL);
  // span _r_ tok
  EXPECT_TRUE(edb->isConnected(annis::constructEdge(125, 124)));
  EXPECT_TRUE(edb->isConnected(annis::constructEdge(61, 60)));


}


#endif // LOADTEST_H
