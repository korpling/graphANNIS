#ifndef LOADTEST_H
#define LOADTEST_H

#include "gtest/gtest.h"
#include "db.h"
#include "annotationsearch.h"
#include "operators/defaultjoins.h"
#include <cstdlib>
#include <boost/format.hpp>

using namespace annis;

class LoadTest : public ::testing::Test {
protected:
  annis::DB db;
  std::string dataDir;
  LoadTest()
    : dataDir("data")
  {
  }

  virtual ~LoadTest() {
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
    bool loadedDB = db.loadRelANNIS(dataDir + "/pcc2_v6_relANNIS");
    ASSERT_EQ(true, loadedDB);
  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(LoadTest, NodeAnnotations) {

  std::list<annis::Annotation> annosAsList = db.getNodeAnnotationsByID(0);
  std::vector<annis::Annotation> annos(annosAsList.size());
  std::copy(annosAsList.begin(), annosAsList.end(), annos.begin());
  ASSERT_EQ(5u, annos.size());


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
  std::vector<annis::Component> components = db.getDirectConnected(annis::Init::initEdge(10, 0));
  ASSERT_EQ(3u, components.size());

  EXPECT_EQ(annis::ComponentType::COVERAGE, components[0].type);
  EXPECT_STREQ(annis_ns.c_str(), components[0].layer);
  EXPECT_STREQ("", components[0].name);

  EXPECT_EQ(annis::ComponentType::COVERAGE, components[1].type);
  EXPECT_STREQ("exmaralda", components[1].layer);
  EXPECT_STREQ("", components[1].name);

  EXPECT_EQ(annis::ComponentType::LEFT_TOKEN, components[2].type);
  EXPECT_STREQ(annis::annis_ns.c_str(), components[2].layer);
  EXPECT_STREQ("", components[2].name);

  components = db.getDirectConnected(annis::Init::initEdge(371, 126));
  ASSERT_EQ(4u, components.size());

  EXPECT_EQ(annis::ComponentType::COVERAGE, components[0].type);
  EXPECT_STREQ(annis_ns.c_str(), components[0].layer);
  EXPECT_STREQ("", components[0].name);

  EXPECT_EQ(annis::ComponentType::DOMINANCE, components[1].type);
  EXPECT_STREQ("tiger", components[1].layer);
  EXPECT_STREQ("", components[1].name);

  EXPECT_EQ(annis::ComponentType::DOMINANCE, components[2].type);
  EXPECT_STREQ("tiger", components[2].layer);
  EXPECT_STREQ("edge", components[2].name);

  EXPECT_EQ(annis::ComponentType::LEFT_TOKEN, components[3].type);
  EXPECT_STREQ(annis::annis_ns.c_str(), components[3].layer);
  EXPECT_STREQ("", components[3].name);
}

TEST_F(LoadTest, OutgoingEdges) {


  AnnotationNameSearch catSearch(db, "tiger", "cat", "CPP");
  EXPECT_TRUE(catSearch.hasNext());

  Match cppNode = catSearch.next();

  const EdgeDB* edbDom = db.getEdgeDB(annis::ComponentType::DOMINANCE, "tiger", "edge");
  std::vector<nodeid_t> outEdges = edbDom->getOutgoingEdges(cppNode.node);
  EXPECT_EQ(3, outEdges.size());

}


TEST_F(LoadTest, EdgeAnnos) {

  annis::Edge edge = annis::Init::initEdge(371, 126);
  std::vector<annis::Component> components = db.getDirectConnected(edge);

  ASSERT_EQ(4u, components.size());

  std::vector<annis::Annotation> edgeAnnos = db.getEdgeAnnotations(components[1], edge);
  EXPECT_EQ(1u, edgeAnnos.size());
  EXPECT_STREQ("tiger", db.strings.str(edgeAnnos[0].ns).c_str());
  EXPECT_STREQ("func", db.strings.str(edgeAnnos[0].name).c_str());
  EXPECT_STREQ("OA", db.strings.str(edgeAnnos[0].val).c_str());

  edgeAnnos = db.getEdgeAnnotations(components[2], edge);
  EXPECT_EQ(1u, edgeAnnos.size());
  EXPECT_STREQ("tiger", db.strings.str(edgeAnnos[0].ns).c_str());
  EXPECT_STREQ("func", db.strings.str(edgeAnnos[0].name).c_str());
  EXPECT_STREQ("OA", db.strings.str(edgeAnnos[0].val).c_str());

  edgeAnnos = db.getEdgeAnnotations(components[0], edge);
  EXPECT_EQ(0u, edgeAnnos.size());
  edgeAnnos = db.getEdgeAnnotations(components[3], edge);
  EXPECT_EQ(0u, edgeAnnos.size());

}

TEST_F(LoadTest, Ordering) {

  annis::Component componentOrdering = annis::Init::initComponent(annis::ComponentType::ORDERING,
                                                                 annis::annis_ns, "");
  const annis::EdgeDB* edb = db.getEdgeDB(componentOrdering);
  ASSERT_TRUE(edb != NULL);
  // tok . tok
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(0, 1)));

  // test the last two token
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(517, 880)));

  // span . tok
  EXPECT_FALSE(edb->isConnected(annis::Init::initEdge(125, 126)));
  // tok . span
  EXPECT_FALSE(edb->isConnected(annis::Init::initEdge(151, 61)));
  // span . span
  EXPECT_FALSE(edb->isConnected(annis::Init::initEdge(152, 61)));

  annis::Component componentLeftToken = annis::Init::initComponent(annis::ComponentType::LEFT_TOKEN,
                                                                  annis::annis_ns, "");
  edb = db.getEdgeDB(componentLeftToken);
  ASSERT_TRUE(edb != NULL);
  // span _l_ tok (both direcctions)
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(125, 124)));
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(124, 125)));
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(61, 49)));
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(49, 61)));

  annis::Component componentRightToken = annis::Init::initComponent(annis::ComponentType::RIGHT_TOKEN,
                                                                   annis::annis_ns, "");
  edb = db.getEdgeDB(componentRightToken);
  ASSERT_TRUE(edb != NULL);
  // span _r_ tok (both direcctions)
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(125, 124)));
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(124, 125)));
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(61, 60)));
  EXPECT_TRUE(edb->isConnected(annis::Init::initEdge(60, 61)));


}

// cat="S" >* "Tiefe"
TEST_F(LoadTest, Dom)
{
  AnnotationNameSearch n1(db, "tiger", "cat", "S");
  AnnotationNameSearch n2(db, annis_ns, annis_tok, "Tiefe");

  unsigned int counter=0;

  const EdgeDB* edbDom = db.getEdgeDB(ComponentType::DOMINANCE, "tiger", "");
  NestedLoopJoin n1Dom2(edbDom, n1, n2, 1, uintmax);

  for(BinaryMatch m=n1Dom2.next(); m.found; m=n1Dom2.next())
    {
     HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%\t%4%\t%5%")
                      % counter % m.lhs.node % m.rhs.node % db.getNodeName(m.lhs.node)
                         % db.getNodeName(m.rhs.node)).str()) ;
    counter++;
  }

  EXPECT_EQ(1u, counter);
}



#endif // LOADTEST_H
