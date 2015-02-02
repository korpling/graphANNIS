#ifndef SEARCHTUEBASMALL_H
#define SEARCHTUEBASMALL_H

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

class SearchTuebaSmall : public ::testing::Test {
 protected:
  DB db;
  SearchTuebaSmall() {

  }

  virtual ~SearchTuebaSmall() {
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
    bool loadedDB = db.load(dataDir + "/tuebadz6_small");
    EXPECT_EQ(true, loadedDB);
  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};


TEST_F(SearchTuebaSmall, EdgeAnno) {

  Query q = ExampleQueries::DomFuncON(db);
  unsigned int counter=0;
  while(q.hasNext() && counter < 200u)
  {
    q.next();
    counter++;
  }

  EXPECT_EQ(153u, counter);
}

TEST_F(SearchTuebaSmall, Dom) {

  Query q(db);
  auto n1 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db,
                                                           annis_ns, annis_node_name));
  auto n2 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db,
                                                        annis_ns, annis_node_name));

  q.addOperator(std::make_shared<Dominance>(db, "", "", 1, 1), n1, n2);

  unsigned int counter=0;
  while(q.hasNext() && counter < 20000u)
  {
    q.next();
    counter++;
  }

  EXPECT_EQ(13021u, counter);
}


#endif // SEARCHTUEBASMALL_H
