#ifndef SEARCHTESTTUEBADAZ_H
#define SEARCHTESTTUEBADAZ_H

#include "gtest/gtest.h"
#include "db.h"
#include "operators/precedence.h"
#include "operators/overlap.h"
#include "operators/inclusion.h"
#include "operators/pointing.h"
#include "operators/dominance.h"
#include "exactannosearch.h"
#include "query.h"

#include <vector>

using namespace annis;

class SearchTestTueBaDZ : public ::testing::Test {
 protected:
  DB db;
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
TEST_F(SearchTestTueBaDZ, Benchmark1) {

  Query q(db);
  auto n1 = q.addNode(std::make_shared<ExactAnnoSearch>(db, annis_ns, annis_node_name));
  auto n2 = q.addNode(std::make_shared<ExactAnnoSearch>(db, "merged", "pos", "PPER"));
  auto n3 = q.addNode(std::make_shared<ExactAnnoSearch>(db, annis_ns, annis_node_name));
  auto n4 = q.addNode(std::make_shared<ExactAnnoSearch>(db, "mmax", "relation", "anaphoric"));
  auto n5 = q.addNode(std::make_shared<ExactAnnoSearch>(db, annis_ns, annis_node_name));
  auto n6 = q.addNode(std::make_shared<ExactAnnoSearch>(db, annis_ns, annis_node_name));
  auto n7 = q.addNode(std::make_shared<ExactAnnoSearch>(db, "mmax", "relation", "anaphoric"));

  Annotation funcOnAnno =
      Init::initAnnotation(db.strings.add("func"), db.strings.add("ON"));

  q.addOperator(std::make_shared<Inclusion>(db), n2, n4);
  q.addOperator(std::make_shared<Pointing>(db, "", "anaphoric"), n4, n7);
  q.addOperator(std::make_shared<Dominance>(db, "", "", funcOnAnno), n1, n3);
  q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n3, n2);
  q.addOperator(std::make_shared<Dominance>(db, "", "", funcOnAnno), n5, n6);
  q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n6, n7);

  unsigned int counter=0;
  while(q.hasNext() && counter < 10u)
  {
    q.next();
    counter++;
  }

  EXPECT_EQ(0u, counter);
}


#endif // SEARCHTESTTUEBADAZ_H
