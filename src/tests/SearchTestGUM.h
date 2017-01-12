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
#include <annis/query.h>
#include <annis/json/jsonqueryparser.h>

#include <vector>
#include <boost/format.hpp>
#include <fstream>

#include "testlogger.h"

#ifdef ENABLE_VALGRIND
  #include <valgrind/callgrind.h>
#else
  #define CALLGRIND_STOP_INSTRUMENTATION

  #define CALLGRIND_START_INSTRUMENTATION
#endif // ENABLE_VALGRIND

using namespace annis;

class SearchTestGUM : public ::testing::Test {
protected:
  DB db;
  std::shared_ptr<Query> q;

  SearchTestGUM() {
  }

  virtual ~SearchTestGUM() {
    // You can do clean-up work that doesn't throw exceptions here.
  }

  // If the constructor and destructor are not enough for setting up
  // and cleaning up each test, you can define the following methods:

  virtual void SetUp() {

    CALLGRIND_STOP_INSTRUMENTATION;
    char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
    std::string dataDir("data");
    if (testDataEnv != NULL) {
      dataDir = testDataEnv;
    }
    bool loadedDB = db.load(dataDir + "/GUM", true);
    EXPECT_EQ(true, loadedDB);

    char* testQueriesEnv = std::getenv("ANNIS4_TEST_QUERIES");
    std::string globalQueryDir("queries");
    if (testQueriesEnv != NULL) {
      globalQueryDir = testQueriesEnv;
    }
    std::string queryDir = globalQueryDir + "/SearchTestGUM";

    // get test name and read the json file
    auto info = ::testing::UnitTest::GetInstance()->current_test_info();
    if(info != nullptr)
    {
      std::ifstream in;
      std::string jsonFileName = queryDir + "/" + info->name() + ".json";
      in.open(jsonFileName);
      if(in.is_open()) {
        QueryConfig config;
        q = JSONQueryParser::parse(db, db.edges, in, config);
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

TEST_F(SearchTestGUM, dep_xcomp) {
  ASSERT_TRUE((bool) q);
  
  unsigned int counter = 0;
  while (q->next()) {
    counter++;
  }

  EXPECT_EQ(1u, counter);
}

TEST_F(SearchTestGUM, entity) {
  ASSERT_TRUE((bool) q);

  unsigned int counter = 0;
  while (q->next() && counter < 100) {
    counter++;
  }

  EXPECT_EQ(2u, counter);
}

TEST_F(SearchTestGUM, corefAnno) {
  ASSERT_TRUE((bool) q);

  unsigned int counter = 0;
  while (q->next() && counter < 700) {
    counter++;
  }

  EXPECT_EQ(636u, counter);
}

TEST_F(SearchTestGUM, IndirectPointingNested) {

  unsigned int counter = 0;

  Query q(db);
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "ref", "entity", "object"));
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "ref", "entity", "abstract"));

  q.addOperator(std::make_shared<Pointing>(db.edges, db.strings, "", "coref", 1, uintmax), 0, 1, true);

  auto startTime = annis::Helper::getSystemTimeInMilliSeconds();
  while (q.next() && counter < 1000) {
    counter++;
  }
  auto endTime = annis::Helper::getSystemTimeInMilliSeconds();
  HL_INFO(logger, "IndirectPointingNested query took " + std::to_string(endTime-startTime) + " ms");


  EXPECT_EQ(273u, counter);
}

TEST_F(SearchTestGUM, tok_dep_tok) {
  ASSERT_TRUE((bool) q);

  unsigned int counter = 0;
  while(q->next() && counter < 1000) {
    counter++;
  }

  EXPECT_EQ(246u, counter);
}

TEST_F(SearchTestGUM, VV_dep) {
  ASSERT_TRUE((bool) q);

  unsigned int counter = 0;
  while(q->next() && counter < 5000) {
    counter++;
  }

  EXPECT_EQ(955u, counter);
}

TEST_F(SearchTestGUM, nonexisting_dep) {
  ASSERT_TRUE((bool) q);

  unsigned int counter = 0;
  while(q->next() && counter < 1000) {
    counter++;
  }

  EXPECT_EQ(0u, counter);
}

TEST_F(SearchTestGUM, kind_dom_kind) {
  ASSERT_TRUE((bool) q);

  unsigned int counter = 0;
  while(q->next() && counter < 1000) {
    counter++;
  }

  EXPECT_EQ(56u, counter);
}

TEST_F(SearchTestGUM, city) {
  ASSERT_TRUE((bool) q);

  unsigned int counter = 0;
  while(q->next() && counter < 1000) {
    counter++;
  }

  EXPECT_EQ(64u, counter);
}

TEST_F(SearchTestGUM, pos_dep_pos_Thread4) {
  QueryConfig config;
  config.numOfBackgroundTasks = 4;
  config.threadPool = std::make_shared<ThreadPool>(4);

  std::shared_ptr<Query> result = std::make_shared<Query>(db, config);

  result->addNode(std::make_shared<ExactAnnoKeySearch>(db, "pos"));
  result->addNode(std::make_shared<ExactAnnoKeySearch>(db, "pos"));

  Annotation edgeAnno = {db.strings.add("func"), 0, db.strings.add("dep")};
  result->addOperator(std::make_shared<Pointing>(db.edges, db.strings, "", "dep", edgeAnno), 0, 1);

  CALLGRIND_START_INSTRUMENTATION;
  unsigned int counter = 0;
  while(result->next() && counter < 1000) {
    counter++;
  }
  CALLGRIND_STOP_INSTRUMENTATION;

  EXPECT_EQ(246u, counter);

}
