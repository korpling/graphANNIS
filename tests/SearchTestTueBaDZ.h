#ifndef SEARCHTESTTUEBADAZ_H
#define SEARCHTESTTUEBADAZ_H

#include "gtest/gtest.h"
#include "db.h"
#include "operators/defaultjoins.h"
#include "operators/precedence.h"
#include "operators/overlap.h"
#include "annotationsearch.h"

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


TEST_F(SearchTestTueBaDZ, DISABLED_Benchmark1) {

  AnnotationNameSearch n1(db, annis_ns, annis_node_name);
  AnnotationNameSearch n2(db, "merged", "pos", "PPER");
  AnnotationNameSearch n3(db, annis_ns, annis_node_name);
  AnnotationNameSearch n4(db, "mmax", "relation", "anaphoric");
  AnnotationNameSearch n5(db, annis_ns, annis_node_name);
  AnnotationNameSearch n6(db, annis_ns, annis_node_name);
  AnnotationNameSearch n7(db, "mmax", "relation", "anaphoric");

  Overlap n2_incl_n4(db, n2, n4);

  unsigned int counter=0;

  EXPECT_EQ(373436, counter);
}


#endif // SEARCHTESTTUEBADAZ_H
