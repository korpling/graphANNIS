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

#include "gtest/gtest.h"
#include <annis/db.h>
#include <annis/annosearch/exactannovaluesearch.h>
#include <annis/annosearch/exactannokeysearch.h>
#include <cstdlib>
#include <boost/format.hpp>
#include <annis/query.h>
#include <annis/operators/dominance.h>
#include <annis/operators/partofsubcorpus.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/util/relannisloader.h>

#include "testlogger.h"

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
    bool loadedDB = RelANNISLoader::loadRelANNIS(db, dataDir + "/../relannis/pcc2");
    ASSERT_EQ(true, loadedDB);
  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(LoadTest, NodeAnnotations) {

  std::vector<annis::Annotation> annos = db.nodeAnnos.getAnnotations(0);
  ASSERT_EQ(6u, annos.size());

  EXPECT_STREQ(annis::annis_ns.c_str(), db.strings.str(annos[0].ns).c_str());
  EXPECT_STREQ("tok", db.strings.str(annos[0].name).c_str());
  EXPECT_STREQ("so", db.strings.str(annos[0].val).c_str());

  EXPECT_STREQ(annis::annis_ns.c_str(), db.strings.str(annos[1].ns).c_str());
  EXPECT_STREQ("node_name", db.strings.str(annos[1].name).c_str());
  EXPECT_STREQ("pcc2/4282#tok_13", db.strings.str(annos[1].val).c_str());

  EXPECT_STREQ(annis::annis_ns.c_str(), db.strings.str(annos[2].ns).c_str());
  EXPECT_STREQ("document", db.strings.str(annos[2].name).c_str());
  EXPECT_STREQ("4282", db.strings.str(annos[2].val).c_str());

  EXPECT_STREQ("tiger", db.strings.str(annos[3].ns).c_str());
  EXPECT_STREQ("lemma", db.strings.str(annos[3].name).c_str());
  EXPECT_STREQ("so", db.strings.str(annos[3].val).c_str());

  EXPECT_STREQ("tiger", db.strings.str(annos[4].ns).c_str());
  EXPECT_STREQ("morph", db.strings.str(annos[4].name).c_str());
  EXPECT_STREQ("--", db.strings.str(annos[4].val).c_str());


  EXPECT_STREQ("tiger", db.strings.str(annos[5].ns).c_str());
  EXPECT_STREQ("pos", db.strings.str(annos[5].name).c_str());
  EXPECT_STREQ("ADV", db.strings.str(annos[5].val).c_str());

}


TEST_F(LoadTest, Edges) {

  // get some edges
  std::vector<annis::Component> components = db.getDirectConnected(annis::Init::initEdge(10, 0));
  ASSERT_EQ(3u, components.size());

  EXPECT_EQ(annis::ComponentType::COVERAGE, components[0].type);
  EXPECT_STREQ(annis_ns.c_str(), components[0].layer.c_str());
  EXPECT_STREQ("", components[0].name.c_str());

  EXPECT_EQ(annis::ComponentType::COVERAGE, components[1].type);
  EXPECT_STREQ("exmaralda", components[1].layer.c_str());
  EXPECT_STREQ("", components[1].name.c_str());

  EXPECT_EQ(annis::ComponentType::LEFT_TOKEN, components[2].type);
  EXPECT_STREQ(annis::annis_ns.c_str(), components[2].layer.c_str());
  EXPECT_STREQ("", components[2].name.c_str());

  components = db.getDirectConnected(annis::Init::initEdge(371, 126));
  ASSERT_EQ(4u, components.size());

  EXPECT_EQ(annis::ComponentType::COVERAGE, components[0].type);
  EXPECT_STREQ(annis_ns.c_str(), components[0].layer.c_str());
  EXPECT_STREQ("", components[0].name.c_str());

  EXPECT_EQ(annis::ComponentType::DOMINANCE, components[1].type);
  EXPECT_STREQ("tiger", components[1].layer.c_str());
  EXPECT_STREQ("", components[1].name.c_str());

  EXPECT_EQ(annis::ComponentType::DOMINANCE, components[2].type);
  EXPECT_STREQ("tiger", components[2].layer.c_str());
  EXPECT_STREQ("edge", components[2].name.c_str());

  EXPECT_EQ(annis::ComponentType::LEFT_TOKEN, components[3].type);
  EXPECT_STREQ(annis::annis_ns.c_str(), components[3].layer.c_str());
  EXPECT_STREQ("", components[3].name.c_str());
}

TEST_F(LoadTest, OutgoingEdges) {


  ExactAnnoValueSearch catSearch(db, "tiger", "cat", "CPP");
  Match cppNode;
  EXPECT_TRUE(catSearch.next(cppNode));

  std::shared_ptr<const ReadableGraphStorage> gsDom = db.edges.getGraphStorage(annis::ComponentType::DOMINANCE, "tiger", "edge");
  std::vector<nodeid_t> outEdges = gsDom->getOutgoingEdges(cppNode.node);
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

  annis::Component componentOrdering = {annis::ComponentType::ORDERING,
                                                                 annis::annis_ns, ""};
  std::shared_ptr<const ReadableGraphStorage> gs = db.edges.getGraphStorage(componentOrdering);
  ASSERT_TRUE(gs != NULL);
  // tok . tok
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(0, 1)));

  // test the last two token
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(517, 880)));

  // span . tok
  EXPECT_FALSE(gs->isConnected(annis::Init::initEdge(125, 126)));
  // tok . span
  EXPECT_FALSE(gs->isConnected(annis::Init::initEdge(151, 61)));
  // span . span
  EXPECT_FALSE(gs->isConnected(annis::Init::initEdge(152, 61)));

  annis::Component componentLeftToken = {annis::ComponentType::LEFT_TOKEN,
                                                                  annis::annis_ns, ""};
  gs = db.edges.getGraphStorage(componentLeftToken);
  ASSERT_TRUE(gs != NULL);
  // span _l_ tok (both direcctions)
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(125, 124)));
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(124, 125)));
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(61, 49)));
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(49, 61)));

  annis::Component componentRightToken = {annis::ComponentType::RIGHT_TOKEN,
                                                                   annis::annis_ns, ""};
  gs = db.edges.getGraphStorage(componentRightToken);
  ASSERT_TRUE(gs != NULL);
  // span _r_ tok (both direcctions)
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(125, 124)));
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(124, 125)));
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(61, 60)));
  EXPECT_TRUE(gs->isConnected(annis::Init::initEdge(60, 61)));


}

// cat="S" >* "Tiefe"
TEST_F(LoadTest, Dom)
{

  unsigned int counter=0;

  Query q(db);
  auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "tiger", "cat", "S"));
  auto n2 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_tok, "Tiefe"));

  q.addOperator(std::make_shared<Dominance>(db.edges, db.strings, "tiger", "", 1, uintmax), n1, n2);

  while(q.next())
  {
    std::vector<Match> m = q.getCurrent();
    HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%\t%4%\t%5%")
                     % counter % m[0].node % m[1].node % db.getNodeName(m[0].node)
                     % db.getNodeName(m[1].node)).str()) ;
    counter++;
  }

  EXPECT_EQ(1u, counter);
}

TEST_F(LoadTest, IsConnected)
{

  annis::Component component = {annis::ComponentType::DOMINANCE,
                                                                 "tiger", ""};
  std::shared_ptr<const ReadableGraphStorage> gs = db.edges.getGraphStorage(component);

  EXPECT_TRUE(gs->isConnected(Init::initEdge(387, 16), 1, uintmax));
  EXPECT_TRUE(gs->isConnected(Init::initEdge(387, 16), 1, 2));
  EXPECT_TRUE(gs->isConnected(Init::initEdge(387, 16), 2, 2));
  EXPECT_FALSE(gs->isConnected(Init::initEdge(387, 16), 3, uintmax));
  EXPECT_FALSE(gs->isConnected(Init::initEdge(387, 16), 5, 10));

}

TEST_F(LoadTest, Distance)
{

  annis::Component component = {annis::ComponentType::DOMINANCE, "tiger", ""};
  std::shared_ptr<const ReadableGraphStorage> gs = db.edges.getGraphStorage(component);

  EXPECT_EQ(2, gs->distance(Init::initEdge(387, 16)));

}

// Should test query
// cat="AP" >2 node
TEST_F(LoadTest, RangedDom) {

  unsigned int counter=0;

  Query q(db);
  auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "tiger", "cat", "AP"));
  auto n2 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_node_name));

  q.addOperator(std::make_shared<Dominance>(db.edges, db.strings, "", "", 3, 5), n1, n2);

  while(q.next() && counter < 2000)
  {
    std::vector<Match> m = q.getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%")
                     % db.getNodeDebugName(m[0].node)
                     % db.getNodeDebugName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(7u, counter);
}

// Should test query
// cat="S" > "was"
TEST_F(LoadTest, SecEdge) {

  unsigned int counter=0;

  Query q(db);
  auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "tiger", "cat", "S"));
  auto n2 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_tok, "was"));

  q.addOperator(std::make_shared<Dominance>(db.edges, db.strings, "", ""), n1, n2);

  while(q.next() && counter < 2000)
  {
    std::vector<Match> m = q.getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%")
                     % db.getNodeDebugName(m[0].node)
                     % db.getNodeDebugName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(2u, counter);
}

TEST_F(LoadTest, NodesOfDocument) {
  Query q(db);

  auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_node_name, "pcc2/11299"));
  auto n2 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_node_name));

  q.addOperator(std::make_shared<PartOfSubCorpus>(db.edges, db.strings), n1, n2);

  int counter=0;
  while(q.next())
  {
    const std::vector<Match> m = q.getCurrent();
    ASSERT_EQ(2, m.size());

    EXPECT_STREQ("pcc2/11299", db.getNodeName(m[0].node).c_str());

    counter++;
  }
  EXPECT_EQ(558u, counter);
}

TEST_F(LoadTest, NodesOfToplevelCorpus) {
  Query q(db);

  auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_node_name, "pcc2"));
  auto n2 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_tok));

  q.addOperator(std::make_shared<PartOfSubCorpus>(db.edges, db.strings), n1, n2);

  int counter=0;
  while(q.next())
  {
    const std::vector<Match> m = q.getCurrent();
    ASSERT_EQ(2, m.size());

    EXPECT_STREQ("pcc2", db.getNodeName(m[0].node).c_str());

    counter++;
  }
  EXPECT_EQ(399u, counter);
}
