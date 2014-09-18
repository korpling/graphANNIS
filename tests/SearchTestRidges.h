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
// pos="NN" . pos . tok . dipl
TEST_F(SearchTestRidges, Benchmark5) {
  unsigned int counter=0;

  AnnotationNameSearch n1(db, "default_ns", "pos", "NN");
  Component c = constructComponent(ComponentType::ORDERING, annis_ns, "tok");
  const EdgeDB* edb = db.getEdgeDB(c);
  if(edb != NULL)
  {
    while(n1.hasNext())
    {
      Match m1 = n1.next();
      AnnotationIterator* itP1_2 = edb->findConnected(m1.first);
      while(itP1_2->hasNext())
      {
        Match m2 = itP1_2->next();

      }
      delete itP1_2;
    }
  }
  // TODO: implement this complex query

//  EXPECT_EQ(27379, counter);
}



#endif // SEARCHTESTRIDGES_H
