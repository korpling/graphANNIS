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
#include <annis/annosearch/annotationsearch.h>
#include <annis/operators/precedence.h>
#include <annis/operators/overlap.h>
#include <annis/operators/inclusion.h>
#include <annis/query/query.h>
#include <annis/query/singlealternativequery.h>

#include <boost/format.hpp>
#include <vector>

#include "testlogger.h"

using namespace annis;

class SearchTestRidges : public ::testing::Test {
public:
  const unsigned int MAX_COUNT = 2000000u;

 protected:
  DB db;
  std::shared_ptr<Query> q;
  
  SearchTestRidges() {
  }

  virtual ~SearchTestRidges() {
    // You can do clean-up work that doesn't throw exceptions here.
  }

  // If the constructor and destructor are not enough for setting up
  // and cleaning up each test, you can define the following methods:

  virtual void SetUp() {
    char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
    std::string dataDir("data");
    if(testDataEnv != NULL)
    {
      dataDir = testDataEnv;
    }
    bool loadedDB = db.load(dataDir + "/ridges");
    EXPECT_EQ(true, loadedDB);
    
    char* testQueriesEnv = std::getenv("ANNIS4_TEST_QUERIES");
    std::string globalQueryDir("queries");
    if (testQueriesEnv != NULL) {
      globalQueryDir = testQueriesEnv;
    }
    std::string queryDir = globalQueryDir + "/SearchTestRidges";

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

TEST_F(SearchTestRidges, DiplNameSearch) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;
  while(q->next() && counter < MAX_COUNT)
  {
    auto m = q->getCurrent();
    ASSERT_EQ(1, m.size());
    ASSERT_STREQ("dipl", db.strings.str(m[0].anno.name).c_str());
    ASSERT_STREQ("default_ns", db.strings.str(m[0].anno.ns).c_str());
    counter++;
  }

  EXPECT_EQ(153732u, counter);
}

TEST_F(SearchTestRidges, PosValueSearch) {
  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;
  while( q->next() && counter < MAX_COUNT)
  {
    auto m = q->getCurrent();
    ASSERT_EQ(1, m.size());
    ASSERT_STREQ("pos", db.strings.str(m[0].anno.name).c_str());
    ASSERT_STREQ("NN", db.strings.str(m[0].anno.val).c_str());
    ASSERT_STREQ("default_ns", db.strings.str(m[0].anno.ns).c_str());
    counter++;
  }

  EXPECT_EQ(27490u, counter);
}

// Should test query
// default_ns:pos="NN" .2,10 default_ns:pos="ART"
TEST_F(SearchTestRidges, Benchmark1) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;

  while(q->next() && counter < MAX_COUNT)
  {
    std::vector<Match> m = q->getCurrent();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(21911u, counter);
}

// Should test query
// tok .2,10 tok
TEST_F(SearchTestRidges, Benchmark2) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;

  while(q->next() && counter < MAX_COUNT)
  {
    counter++;
  }

  EXPECT_EQ(1386828u, counter);
}

// Should test query
// default_ns:pos="PTKANT" . node
TEST_F(SearchTestRidges, PrecedenceMixedSpanTok) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;

  while(q->next() && counter < 100u)
  {
    std::vector<Match> m = q->getCurrent();
    HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%") % counter % db.getNodeName(m[0].node)
                       % db.getNodeName(m[1].node)).str()) ;
    counter++;
  }

  EXPECT_EQ(29u, counter);
}

// Should test query
// default_ns:pos="NN" & default_ns:norm="Blumen" & #1 _o_ #2
TEST_F(SearchTestRidges, NestedOverlap) {

  unsigned int counter=0;

  SingleAlternativeQuery q(db);
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "norm", "Blumen"));

  q.addOperator(std::make_shared<Overlap>(db, db.edges.getFunc), 0, 1, true);

  while(q.next())
  {
    auto m = q.getCurrent();
    //HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%") % counter % db.getNodeName(m[0].node)
    //                 % db.getNodeName(m[1].node)).str()) ;
    counter++;
  }

  EXPECT_EQ(152u, counter);
}

// Should test query
// default_ns:pos="NN" & default_ns:norm="Blumen" & #1 _o_ #2
TEST_F(SearchTestRidges, SeedOverlap) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;

  while(q->next() && counter < MAX_COUNT)
  {
    auto m = q->getCurrent();
    //HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%") % counter % db.getNodeName(m[0].node)
    //                 % db.getNodeName(m[1].node)).str()) ;
    counter++;
  }

  EXPECT_EQ(152u, counter);
}

// Should test query
// default_ns:pos="NN" & default_ns:norm="Blumen" & #1 _i_ #2
TEST_F(SearchTestRidges, Inclusion) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;

  while(q->next() && counter < MAX_COUNT)
  {
    auto m = q->getCurrent();
    HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%") % counter % m[0].node % m[1].node).str()) ;
    counter++;
  }
  EXPECT_EQ(152u, counter);

}
