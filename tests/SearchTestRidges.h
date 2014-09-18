#ifndef SEARCHTESTRIDGES_H
#define SEARCHTESTRIDGES_H

#include "gtest/gtest.h"
#include "db.h"
#include "annotationsearch.h"

#include <vector>

using namespace annis;

class SearchTestRidges : public ::testing::Test {
 protected:
  DB db;
  SearchTestRidges() {
    bool result = db.load("/home/thomas/korpora/a4/ridges");
    EXPECT_EQ(true, result);
  }

  virtual ~SearchTestRidges() {
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

TEST_F(SearchTestRidges, DiplSearch) {
  AnnotationNameSearch search(db, "dipl");
  unsigned int counter=0;
  while(search.hasNext())
  {
    Match m = search.next();
    ASSERT_STREQ("dipl", db.str(m.second.name).c_str());
    ASSERT_STREQ("default_ns", db.str(m.second.ns).c_str());
    counter++;
  }

  EXPECT_EQ(153732, counter);
}



#endif // SEARCHTESTRIDGES_H
