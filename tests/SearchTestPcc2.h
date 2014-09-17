#ifndef SEARCHTESTPCC2_H
#define SEARCHTESTPCC2_H

#include "gtest/gtest.h"
#include "db.h"
#include "annotationsearch.h"

#include <vector>

using namespace annis;

class SearchTestPcc2 : public ::testing::Test {
 protected:
  DB db;
  SearchTestPcc2() {
    bool result = db.loadRelANNIS("/home/thomas/korpora/pcc/pcc-2/pcc2_v6_relANNIS");
//    bool result = db.load("/home/thomas/korpora/a4/pcc2");
    EXPECT_EQ(true, result);
  }

  virtual ~SearchTestPcc2() {
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

TEST_F(SearchTestPcc2, CatSearch) {
  AnnotationNameSearch search(db, "cat");
  unsigned int counter=0;
  while(search.hasNext())
  {
    Match m = search.next();
    EXPECT_STREQ("cat", db.str(m.second.name).c_str());
    EXPECT_STREQ("tiger", db.str(m.second.ns).c_str());
    counter++;
  }

  EXPECT_EQ(155, counter);
}



#endif // SEARCHTESTPCC2_H
