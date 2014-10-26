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

  std::pair<bool, uint32_t> n2_nameID = db.strings.findID("pos");
  std::pair<bool, uint32_t> n2_valueID = db.strings.findID("ART");
  if(n2_nameID.first && n2_valueID.first)
  {
    Component cOrder = initComponent(ComponentType::ORDERING, annis_ns, "");
    Component cLeft = initComponent(ComponentType::LEFT_TOKEN, annis_ns, "");
    Component cRight = initComponent(ComponentType::RIGHT_TOKEN, annis_ns, "");


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

        // get the right-most covered token of m1
        std::uint32_t tok1 = edbRight->getOutgoingEdges(m1.first)[0];

        // find all token in the range 2-10
        EdgeIterator* itConnected = edbOrder->findConnected(tok1, 2, 10);
        for(std::pair<bool, std::uint32_t> tok2 = itConnected->next();
            tok2.first; tok2 = itConnected->next())
        {
          // get all node that are left-aligned with tok2
          std::vector<std::uint32_t> n2_candidates = edbLeft->getOutgoingEdges(tok2.second);
          for(size_t i=0; i < n2_candidates.size(); i++)
          {
            // check if the node has the correct annotations
            std::vector<Annotation> n2_annos = db.getNodeAnnotationsByID(n2_candidates[i]);
            for(size_t j=0; j < n2_annos.size(); j++)
            {
              if(n2_annos[j].val == n2_valueID.second && n2_annos[j].name == n2_nameID.second)
              {
                counter++;
                break; // we don't have to search for other annotations
              }
            }
          }
        }
        delete itConnected;
      }
    }
  } // end if pos="ART" strings found

  EXPECT_EQ(21911, counter);
}

// Should test query
// tok .2,10 tok
TEST_F(SearchTestRidges, Benchmark2) {

  unsigned int counter=0;

  AnnotationNameSearch n1(db, annis::annis_ns, annis::annis_tok);

  std::pair<bool, uint32_t> n2_namespaceID = db.strings.findID(annis::annis_ns);
  std::pair<bool, uint32_t> n2_nameID = db.strings.findID(annis::annis_tok);
  if(n2_nameID.first && n2_namespaceID.first)
  {
    Component cOrder = initComponent(ComponentType::ORDERING, annis_ns, "");


    const EdgeDB* edbOrder = db.getEdgeDB(cOrder);
    if(edbOrder != NULL)
    {
      while(n1.hasNext())
      {
        Match m1 = n1.next();

        // find all token in the range 2-10
        EdgeIterator* itConnected = edbOrder->findConnected(m1.first, 2, 10);
        for(std::pair<bool, std::uint32_t> tok2 = itConnected->next();
            tok2.first; tok2 = itConnected->next())
        {
          // check if the node has the correct annotations
          std::vector<Annotation> n2_annos = db.getNodeAnnotationsByID(tok2.second);
          for(size_t j=0; j < n2_annos.size(); j++)
          {
            if(n2_annos[j].ns == n2_namespaceID.second && n2_annos[j].name == n2_nameID.second)
            {
              counter++;
              break; // we don't have to search for other annotations
            }
          }
        }
        delete itConnected;
      }
    }
  } // end if pos="ART" strings found

  EXPECT_EQ(1386828, counter);
}


#endif // SEARCHTESTRIDGES_H
