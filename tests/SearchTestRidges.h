#ifndef SEARCHTESTRIDGES_H
#define SEARCHTESTRIDGES_H


#include "gtest/gtest.h"
#include "db.h"
#include "annotationsearch.h"
#include "operators/precedence.h"
#include "operators/overlap.h"
#include "operators/inclusion.h"
#include "query.h"

#include <boost/format.hpp>
#include <vector>

#include <humblelogging/api.h>

using namespace annis;

class SearchTestRidges : public ::testing::Test {
public:
  const unsigned int MAX_COUNT = 2000000u;
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
//    // manually convert all components to fallback implementation
//    auto components = db.getAllComponents();
//    for(auto c : components)
//    {
//      db.convertComponent(c, "fallback");
//    }
  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(SearchTestRidges, DiplNameSearch) {
  ExactAnnoKeySearch search(db, "dipl");
  unsigned int counter=0;
  while(search.hasNext() && counter < MAX_COUNT)
  {
    Match m = search.next();
    ASSERT_STREQ("dipl", db.strings.str(m.anno.name).c_str());
    ASSERT_STREQ("default_ns", db.strings.str(m.anno.ns).c_str());
    counter++;
  }

  EXPECT_EQ(153732u, counter);
}

TEST_F(SearchTestRidges, PosValueSearch) {
  ExactAnnoValueSearch search(db, "default_ns", "pos", "NN");
  unsigned int counter=0;
  while(search.hasNext() && counter < MAX_COUNT)
  {
    Match m = search.next();
    ASSERT_STREQ("pos", db.strings.str(m.anno.name).c_str());
    ASSERT_STREQ("NN", db.strings.str(m.anno.val).c_str());
    ASSERT_STREQ("default_ns", db.strings.str(m.anno.ns).c_str());
    counter++;
  }

  EXPECT_EQ(27490u, counter);
}

// Should test query
// pos="VVIZU" .10000,10010 pos="ART"
TEST_F(SearchTestRidges, Benchmark1) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "ART"));

  q.addOperator(std::make_shared<Precedence>(db, 2,10), 0, 1);

  while(q.hasNext() && counter < MAX_COUNT)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(21911u, counter);
}

// Should test query
// tok .2,10 tok
TEST_F(SearchTestRidges, Benchmark2) {

  unsigned int counter=0;

  Query q(db);

  q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis::annis_ns, annis::annis_tok));
  q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis::annis_ns,annis::annis_tok));

  q.addOperator(std::make_shared<Precedence>(db, 2, 10), 0, 1);
  while(q.hasNext() && counter < MAX_COUNT)
  {
    q.next();
    counter++;
  }

  EXPECT_EQ(1386828u, counter);
}

// Should test query
// tok .2,10 tok
TEST_F(SearchTestRidges, ClassicBenchmark2) {

  unsigned int counter=0;

  ExactAnnoKeySearch n1(db, annis::annis_ns, "tok");

  Annotation anyTokAnno = Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());

  std::pair<bool, uint32_t> n2_namespaceID = db.strings.findID(annis::annis_ns);
  std::pair<bool, uint32_t> n2_nameID = db.strings.findID("tok");
  if(n2_nameID.first && n2_namespaceID.first)
  {
    Component cOrder = {ComponentType::ORDERING, annis_ns, ""};


    const ReadableGraphStorage* edbOrder = db.getEdgeDB(cOrder);
    if(edbOrder != NULL)
    {
      while(n1.hasNext())
      {
        Match m1 = n1.next();

        // find all token in the range 2-10
        std::unique_ptr<EdgeIterator> itConnected = edbOrder->findConnected(m1.node, 2, 10);
        for(std::pair<bool, std::uint32_t> tok2 = itConnected->next();
            tok2.first; tok2 = itConnected->next())
        {
          // check if the node has the correct annotations
          for(const Annotation& anno : db.getNodeAnnotationsByID(tok2.second))
          {
            if(checkAnnotationEqual(anyTokAnno, anno))
            {
              counter++;
              break; // we don't have to search for other annotations
            }
          }
        }
      }
    }
  } // end if

  EXPECT_EQ(1386828u, counter);
}


// Should test query
// pos="PTKANT" . node
TEST_F(SearchTestRidges, PrecedenceMixedSpanTok) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "PTKANT"));
  q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis::annis_ns,annis::annis_node_name));

  q.addOperator(std::make_shared<Precedence>(db, 1, 1), 0, 1);
  while(q.hasNext() && counter < 100u)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%") % counter % db.getNodeName(m[0].node)
                       % db.getNodeName(m[1].node)).str()) ;
    counter++;
  }

  EXPECT_EQ(29u, counter);
}

// Should test query
// pos="NN" & norm="Blumen" & #1 _o_ #2
TEST_F(SearchTestRidges, NestedOverlap) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "norm", "Blumen"));

  q.addOperator(std::make_shared<Overlap>(db), 0, 1, true);

  while(q.hasNext())
  {
    auto m = q.next();
    //HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%") % counter % db.getNodeName(m[0].node)
    //                 % db.getNodeName(m[1].node)).str()) ;
    counter++;
  }

  EXPECT_EQ(152u, counter);
}

// Should test query
// pos="NN" & norm="Blumen" & #1 _o_ #2
TEST_F(SearchTestRidges, SeedOverlap) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "norm", "Blumen"));

  q.addOperator(std::make_shared<Overlap>(db), 0, 1, false);

  while(q.hasNext())
  {
    auto m = q.next();
    //HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%") % counter % db.getNodeName(m[0].node)
    //                 % db.getNodeName(m[1].node)).str()) ;
    counter++;
  }

  EXPECT_EQ(152u, counter);
}

// Should test query
// pos="NN" & norm="Blumen" & #1 _i_ #2
TEST_F(SearchTestRidges, Inclusion) {

  unsigned int counter=0;

  std::shared_ptr<AnnotationSearch> n1(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "NN"));
  std::shared_ptr<AnnotationSearch> n2(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "norm", "Blumen"));

  annis::Query q(db);
  q.addNode(n1);
  q.addNode(n2);

  q.addOperator(std::make_shared<Inclusion>(db), 0, 1);

  while(q.hasNext() && counter < MAX_COUNT)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("Match %1%\t%2%\t%3%") % counter % m[0].node % m[1].node).str()) ;
    counter++;
  }

  EXPECT_EQ(152u, counter);
}


#endif // SEARCHTESTRIDGES_H
