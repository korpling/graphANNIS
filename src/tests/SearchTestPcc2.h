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
#include <annis/annosearch/regexannosearch.h>
#include <annis/operators/overlap.h>
#include <annis/operators/inclusion.h>
#include <annis/operators/precedence.h>
#include <annis/operators/pointing.h>
#include <annis/operators/dominance.h>
#include <annis/query/query.h>
#include <annis/json/jsonqueryparser.h>

#include "testlogger.h"

#include <vector>
#include <boost/format.hpp>
#include <fstream>

using namespace annis;

class SearchTestPcc2 : public ::testing::Test {
protected:
  DB db;
  std::shared_ptr<Query> q;

  SearchTestPcc2() {
  }

  virtual ~SearchTestPcc2() {
    // You can do clean-up work that doesn't throw exceptions here.
  }

  // If the constructor and destructor are not enough for setting up
  // and cleaning up each test, you can define the following methods:

  virtual void SetUp() {
    char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
    std::string dataDir("data");
    if (testDataEnv != NULL) {
      dataDir = testDataEnv;
    }
    bool loadedDB = db.load(dataDir + "/pcc2");
    EXPECT_EQ(true, loadedDB);

    char* testQueriesEnv = std::getenv("ANNIS4_TEST_QUERIES");
    std::string globalQueryDir("queries");
    if (testQueriesEnv != NULL) {
      globalQueryDir = testQueriesEnv;
    }
    std::string queryDir = globalQueryDir + "/SearchTestPcc2";

    // get test name and read the json file
    auto info = ::testing::UnitTest::GetInstance()->current_test_info();
    if(info != nullptr)
    {
      std::ifstream in;
      std::string jsonFileName = queryDir + "/" + info->name() + ".json";
      in.open(jsonFileName);
      if(in.is_open()) {
        q = JSONQueryParser::parse(db, db.edges, in);
        in.close();
      }
    }
  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(SearchTestPcc2, CatSearch) {
  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;
  while (q->next()) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(1, m.size());
    ASSERT_STREQ("cat", db.strings.str(m[0].anno.name).c_str());
    ASSERT_STREQ("tiger", db.strings.str(m[0].anno.ns).c_str());
    counter++;
  }

  EXPECT_EQ(155u, counter);
}

TEST_F(SearchTestPcc2, MMaxAnnos_ambiguity) {
  
  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;
  while (q->next()) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(1, m.size());
    ASSERT_STREQ("mmax", db.strings.str(m[0].anno.ns).c_str());
    ASSERT_STREQ("ambiguity", db.strings.str(m[0].anno.name).c_str());
    ASSERT_STREQ("not_ambig", db.strings.str(m[0].anno.val).c_str());
    counter++;
  }

  EXPECT_EQ(73u, counter);
}

TEST_F(SearchTestPcc2, MMaxAnnos_complex_np) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;
  while (q->next()) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(1, m.size());
    ASSERT_STREQ("mmax", db.strings.str(m[0].anno.ns).c_str());
    ASSERT_STREQ("complex_np", db.strings.str(m[0].anno.name).c_str());
    ASSERT_STREQ("yes", db.strings.str(m[0].anno.val).c_str());
    counter++;
  }

  EXPECT_EQ(17u, counter);
}

TEST_F(SearchTestPcc2, TokenIndex) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next()) {
    counter++;
  }

  EXPECT_EQ(2u, counter);
}

TEST_F(SearchTestPcc2, IsConnectedRange) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next()) {
    counter++;
  }

  EXPECT_EQ(1u, counter);
}

TEST_F(SearchTestPcc2, DepthFirst) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next()) {
    counter++;
  }

  EXPECT_EQ(9u, counter);
}

// exmaralda:Inf-Stat="new" _o_ exmaralda:PP
TEST_F(SearchTestPcc2, TestQueryOverlap1) {

  ASSERT_TRUE((bool) q);

  unsigned int counter = 0;
  while (q->next()) {
    auto m = q->getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(3u, counter);
}

// mmax:ambiguity="not_ambig" _o_ mmax:complex_np="yes"
TEST_F(SearchTestPcc2, TestQueryOverlap2) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;
  while (q->next()) {
    std::vector<Match> m = q->getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(47u, counter);
}

// mmax:ambiguity="not_ambig" _i_ mmax:complex_np="yes"
TEST_F(SearchTestPcc2, InclusionQuery) {
  
  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;
  while (q->next()) {
    std::vector<Match> m = q->getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(23u, counter);
}

TEST_F(SearchTestPcc2, StructureInclusionSeed) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;
  while (q->next()) {
    std::vector<Match> m = q->getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(2u, counter);
}

TEST_F(SearchTestPcc2, StructureInclusionFilter) {

  SingleAlternativeQuery q(db);
  auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "cat", "S"));
  auto n2 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "cat", "AP"));

  q.addOperator(std::make_shared<Inclusion>(db, db.edges), n1, n2, true);

  unsigned int counter = 0;
  while (q.next()) {
    std::vector<Match> m = q.getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(2u, counter);
}

TEST_F(SearchTestPcc2, AnyNodeIncludeSeed) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;
  while (q->next()) {
    std::vector<Match> m = q->getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeDebugName(m[0].node) % db.getNodeDebugName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(14349u, counter);
}

TEST_F(SearchTestPcc2, AnyNodeIncludeFilter) {

  SingleAlternativeQuery q(db);
  auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_node_type, "node"));
  auto n2 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_node_type, "node"));

  q.addOperator(std::make_shared<Inclusion>(db, db.edges), n1, n2, true);

  unsigned int counter = 0;
  while (q.next()) {
    std::vector<Match> m = q.getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeDebugName(m[0].node) % db.getNodeDebugName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(14349u, counter);
}

TEST_F(SearchTestPcc2, NodeCount) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;
  while (q->next()) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(1, m.size());
    HL_INFO(logger, (boost::format("match\t%1%") % db.getNodeName(m[0].node)).str());
    counter++;
  }

  EXPECT_EQ(998u, counter);
}

TEST_F(SearchTestPcc2, Precedence) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 2000) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(27u, counter);
}

TEST_F(SearchTestPcc2, TokIdentCovNN) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 2000) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(5u, counter);
}

TEST_F(SearchTestPcc2, TokIdentCovNode) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 2000) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(2u, counter);
}

TEST_F(SearchTestPcc2, NodeIdentCovNode) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 2000) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(1078u, counter);
}

// Should test query
// mmax:np_form="defnp" & mmax:np_form="pper"  & #2 ->anaphor_antecedent * #1
TEST_F(SearchTestPcc2, IndirectPointing) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 2000) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(13u, counter);
}

TEST_F(SearchTestPcc2, IndirectPointingNested) {

  unsigned int counter = 0;

  SingleAlternativeQuery q(db);
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "mmax", "np_form", "defnp"));
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "mmax", "np_form", "pper"));

  q.addOperator(std::make_shared<Pointing>(db.edges, db.strings, "", "anaphor_antecedent", 1, uintmax), 1, 0, true);

  while (q.next() && counter < 2000) {
    std::vector<Match> m = q.getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(13u, counter);
}

// Should test query
// mmax:np_form="defnp" & mmax:np_form="pper"  & #2 ->anaphor_antecedent #1
TEST_F(SearchTestPcc2, DirectPointing) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 2000) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(5u, counter);
}

TEST_F(SearchTestPcc2, DirectPointingNested) {

  unsigned int counter = 0;

  SingleAlternativeQuery q(db);
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "mmax", "np_form", "defnp"));
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "mmax", "np_form", "pper"));

  q.addOperator(std::make_shared<Pointing>(db.edges, db.strings, "", "anaphor_antecedent", 1, 1), 1, 0, true);

  while (q.next() && counter < 2000) {
    std::vector<Match> m = q.getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(5u, counter);
}

// Should test query
// pos="ADJD" & "." & #1 ->dep[func="punct"] #2

TEST_F(SearchTestPcc2, DirectPointingWithAnno) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 2000) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(4u, counter);
}

TEST_F(SearchTestPcc2, DirectPointingWithAnnoNested) {

  unsigned int counter = 0;

  SingleAlternativeQuery q(db);
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "tiger", "pos", "ADJD"));
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_tok, "."));

  std::shared_ptr<Operator> op =
          std::make_shared<Pointing>(
          db.edges, db.strings, "", "dep",
          Init::initAnnotation(db.strings.add("func"), db.strings.add("punct")));
  q.addOperator(op, 0, 1, true);

  while (q.next() && counter < 2000) {
    std::vector<Match> m = q.getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(4u, counter);
}

// Should test query
// tiger:cat="S" >2,4 cat
TEST_F(SearchTestPcc2, RangedDominance) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 2000) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(93u, counter);
}


// Should test query
// node >2,4 node
TEST_F(SearchTestPcc2, MultiDominance) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 4000) {
    std::vector<Match> m = q->getCurrent();
    ASSERT_EQ(2, m.size());
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(2072u, counter);
}

TEST_F(SearchTestPcc2, Regex) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 100) {
    std::vector<Match> m = q->getCurrent();
    counter++;
  }

  EXPECT_EQ(12, counter);
}

TEST_F(SearchTestPcc2, Profile) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;

  while (q->next() && counter < 5000) {
    std::vector<Match> m = q->getCurrent();
    counter++;
  }

  EXPECT_EQ(38, counter);
}

TEST_F(SearchTestPcc2, InvalidReflexivity) {
  ASSERT_TRUE((bool) q);

  EXPECT_FALSE(q->next());
}
