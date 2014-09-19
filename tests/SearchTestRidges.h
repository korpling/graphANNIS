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
// pos="NN" .2,10 pos="ART"
TEST_F(SearchTestRidges, BenchmarkTest) {

  unsigned int counter=0;

  AnnotationNameSearch n1(db, "default_ns", "pos", "VAIMP");
  Component cOrder = constructComponent(ComponentType::ORDERING, annis_ns, "");
  Component cLeft = constructComponent(ComponentType::LEFT_TOKEN, annis_ns, "");
  Component cRight = constructComponent(ComponentType::RIGHT_TOKEN, annis_ns, "");


  const EdgeDB* edbOrder = db.getEdgeDB(cOrder);
  const EdgeDB* edbLeft = db.getEdgeDB(cLeft);
  const EdgeDB* edbRight = db.getEdgeDB(cRight);
  if(edbOrder != NULL && edbLeft != NULL && edbRight != NULL)
  {
    // get all nodes with pos="NN"
    unsigned int n1Counter =0;
    while(n1.hasNext())
    {
      Match m1 = n1.next();
      n1Counter++;

      std::cout << "pos=\"NN\" check nr. " << n1Counter << std::endl;

      // get the right-most covered token of m1
      std::uint32_t tok1 = edbRight->getOutgoingEdges(m1.first)[0];

      // check with all matching nodes for #2
      AnnotationNameSearch n2(db, "default_ns", "pos", "PIDAT");
      while(n2.hasNext())
      {
        Match m2 = n2.next();
        // get the left-most covered token of m2
        std::uint32_t tok2 = edbRight->getOutgoingEdges(m2.first)[0];
        // check if both are connected
        if(edbOrder->isConnected(constructEdge(tok1, tok2), 2,10))
        {
          counter++;
        }
      }
    }
  }

  EXPECT_EQ(21911, counter);
}



#endif // SEARCHTESTRIDGES_H
