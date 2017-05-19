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

#include <gtest/gtest.h>
#include <annis/db.h>
#include <annis/util/helper.h>
#include <annis/query/query.h>
#include <annis/operators/precedence.h>
#include <annis/operators/dominance.h>
#include <annis/annosearch/exactannovaluesearch.h>
#include <annis/annosearch/exactannokeysearch.h>
#include <annis/wrapper.h>

#include <vector>

#include "testlogger.h"

using namespace annis;

class SearchTestTiger : public ::testing::Test {
public:
  const unsigned int MAX_COUNT = 5000000u;

 protected:
  DB db;
  std::shared_ptr<Query> q;
  SearchTestTiger() {

  }

  virtual ~SearchTestTiger() {
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
    bool loadedDB = db.load(dataDir + "/tiger2");
    EXPECT_EQ(true, loadedDB);
    
    char* testQueriesEnv = std::getenv("ANNIS4_TEST_QUERIES");
    std::string globalQueryDir("queries");
    if (testQueriesEnv != NULL) {
      globalQueryDir = testQueriesEnv;
    }
    std::string queryDir = globalQueryDir + "/SearchTestTiger";

    // get test name and read the json file
    auto info = ::testing::UnitTest::GetInstance()->current_test_info();
    if(info != nullptr)
    {
      std::ifstream in;
      std::string jsonFileName = queryDir + "/" + info->name() + ".json";
      in.open(jsonFileName);
      if(in.is_open()) {
        q = JSONQueryParser::parse(db, in);
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

TEST_F(SearchTestTiger, CatSearch) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;
  while(q->next() && counter < MAX_COUNT)
  {
    auto m = q->getCurrent();
    ASSERT_EQ(1, m.size());
    ASSERT_STREQ("cat", db.strings.str(m[0].anno.name).c_str());
    ASSERT_STREQ("tiger", db.strings.str(m[0].anno.ns).c_str());
    counter++;
  }

  EXPECT_EQ(373436u, counter);
}

// Should test query
// tiger:pos="NN" .2,10 tiger:pos="ART"
TEST_F(SearchTestTiger, TokenPrecedence) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;

  while(q->next() && counter < MAX_COUNT)
  {
    counter++;
  }

  EXPECT_EQ(179024u, counter);
}

// Should test query
// tiger:pos="NN" .2,10 tiger:pos="ART" . tiger:pos="NN"
TEST_F(SearchTestTiger, TokenPrecedenceThreeNodes) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;

  while(q->next() && counter < MAX_COUNT)
  {
    counter++;
  }

  EXPECT_EQ(114042u, counter);
}

// tiger:cat="S" & tok="Bilharziose" & #1 >* #2
TEST_F(SearchTestTiger, BilharzioseSentence)
{
  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;

  while(q->next() && counter < MAX_COUNT)
  {
    auto m = q->getCurrent();
     HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%")
                      % counter
                      % db.getNodeDebugName(m[0].node)
                      % db.getNodeDebugName(m[1].node)).str()) ;
    counter++;
  }

  EXPECT_EQ(21u, counter);
}


