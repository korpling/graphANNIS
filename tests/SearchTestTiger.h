#ifndef SEARCHTESTTIGER_H
#define SEARCHTESTTIGER_H

#include "gtest/gtest.h"
#include "db.h"
#include "annotationsearch.h"

#include <vector>

using namespace annis;

class SearchTestTiger : public ::testing::Test {
 protected:
  DB db;
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
  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(SearchTestTiger, CatSearch) {
  AnnotationNameSearch search(db, "cat");
  unsigned int counter=0;
  while(search.hasNext())
  {
    Match m = search.next();
    ASSERT_STREQ("cat", db.strings.str(m.second.name).c_str());
    ASSERT_STREQ("tiger", db.strings.str(m.second.ns).c_str());
    counter++;
  }

  EXPECT_EQ(373436, counter);
}



#endif // SEARCHTESTTIGER_H
