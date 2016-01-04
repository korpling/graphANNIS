#ifndef SEARCHTESTTUEBADAZ_H
#define SEARCHTESTTUEBADAZ_H

#include "gtest/gtest.h"
#include "db.h"
#include "operators/precedence.h"
#include "operators/overlap.h"
#include "operators/inclusion.h"
#include "operators/pointing.h"
#include "operators/dominance.h"
#include "exactannovaluesearch.h"
#include "query.h"
#include "../benchmarks/examplequeries.h"

#include <vector>

using namespace annis;

class SearchTestTueBaDZ : public ::testing::Test {
 protected:
  DB db;
  std::shared_ptr<Query> q;
  SearchTestTueBaDZ() {

  }

  virtual ~SearchTestTueBaDZ() {
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
    bool loadedDB = db.load(dataDir + "/tuebadz6");
    EXPECT_EQ(true, loadedDB);
    
    char* testQueriesEnv = std::getenv("ANNIS4_TEST_QUERIES");
    std::string globalQueryDir("queries");
    if (testQueriesEnv != NULL) {
      globalQueryDir = testQueriesEnv;
    }
    std::string queryDir = globalQueryDir + "/SearchTestTueBaDZ";

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

/*
 * Query:

node & merged:pos="PPER" & node & mmax:relation="anaphoric" & node & node & mmax:relation="anaphoric"
& #1 >[func="ON"] #3
& #3 >* #2
& #2 _i_ #4
& #5 >[func="ON"] #6
& #6 >* #7
& #4 ->anaphoric #7
*/
TEST_F(SearchTestTueBaDZ, Mix) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;
  while(q->hasNext() && counter < 10u)
  {
    q->next();
    counter++;
  }

  EXPECT_EQ(0u, counter);
}

TEST_F(SearchTestTueBaDZ, RegexDom) {

  ASSERT_TRUE((bool) q);
  
  unsigned int counter=0;
  while(q->hasNext() && counter < 100)
  {
    q->next();
    counter++;
  }

  EXPECT_EQ(1u, counter);
}

TEST_F(SearchTestTueBaDZ, NodeDom) {

 ASSERT_TRUE((bool) q);
  unsigned int counter=0;
  while(q->hasNext() && counter < 2200000u)
  {
    q->next();
    counter++;
  }

  EXPECT_EQ(2140993u, counter);
}


#endif // SEARCHTESTTUEBADAZ_H
