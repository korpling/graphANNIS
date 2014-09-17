#ifndef SEARCHTESTPCC2_H
#define SEARCHTESTPCC2_H

#include "gtest/gtest.h"
#include "db.h"
#include "annotationsearch.h"

using namespace annis;

class SearchTestPcc2 : public ::testing::Test {
 protected:
  DB db;
  SearchTestPcc2() {
    bool result = db.load("/home/thomas/korpora/a4/pcc2");
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
  while(search.hasNext())
  {
    Match m = search.next();
/*    std::cout << "ns: " << db.str(anno.ns) << "name: " << db.str(anno.name)
                 << "val: " << db.str(anno.val) << std::endl;
                 */
  }
}



#endif // SEARCHTESTPCC2_H
