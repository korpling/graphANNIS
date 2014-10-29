#ifndef SEARCHTESTRIDGES_H
#define SEARCHTESTRIDGES_H

#include "gtest/gtest.h"
#include "db.h"
#include "annotationsearch.h"
#include "operators/defaultjoins.h"
#include "operators/precedence.h"

#include <vector>

using namespace annis;

class SearchTestRidges : public ::testing::Test {
 protected:
  DB db;
  SearchTestRidges() {
  }

  virtual ~SearchTestRidges() {
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
    bool loadedDB = db.load(dataDir + "/ridges");
    EXPECT_EQ(true, loadedDB);
  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(SearchTestRidges, DiplNameSearch) {
  AnnotationNameSearch search(db, "dipl");
  unsigned int counter=0;
  while(search.hasNext())
  {
    Match m = search.next();
    ASSERT_STREQ("dipl", db.strings.str(m.second.name).c_str());
    ASSERT_STREQ("default_ns", db.strings.str(m.second.ns).c_str());
    counter++;
  }

  EXPECT_EQ(153732, counter);
}

TEST_F(SearchTestRidges, PosValueSearch) {
  AnnotationNameSearch search(db, "default_ns", "pos", "NN");
  unsigned int counter=0;
  while(search.hasNext())
  {
    Match m = search.next();
    ASSERT_STREQ("pos", db.strings.str(m.second.name).c_str());
    ASSERT_STREQ("NN", db.strings.str(m.second.val).c_str());
    ASSERT_STREQ("default_ns", db.strings.str(m.second.ns).c_str());
    counter++;
  }

  EXPECT_EQ(27490, counter);
}

// Should test query
// pos="NN" .2,10 pos="ART"
TEST_F(SearchTestRidges, Benchmark1) {

  unsigned int counter=0;

  AnnotationNameSearch n1(db, "default_ns", "pos", "NN");
  AnnotationNameSearch n2(db, "default_ns", "pos", "ART");

  Precedence join(db, n1, n2, 2, 10);
  for(BinaryMatch m=join.next(); m.found; m = join.next())
  {
    counter++;
  }

  EXPECT_EQ(21911, counter);
}

// Should test query
// tok .2,10 tok
TEST_F(SearchTestRidges, Benchmark2) {

  unsigned int counter=0;

  AnnotationNameSearch n1(db, annis::annis_ns, annis::annis_tok);
  Annotation annos2 = initAnnotation(db.strings.add(annis::annis_tok), 0, db.strings.add(annis::annis_ns));

  annis::SeedJoin join(db, db.getEdgeDB(ComponentType::ORDERING, annis_ns, ""), n1, annos2, 2, 10);
  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }

  EXPECT_EQ(1386828, counter);
}


#endif // SEARCHTESTRIDGES_H
