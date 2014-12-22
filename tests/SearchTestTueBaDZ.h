#ifndef SEARCHTESTTUEBADAZ_H
#define SEARCHTESTTUEBADAZ_H

#include "gtest/gtest.h"
#include "db.h"
#include "operators/defaultjoins.h"
#include "operators/precedence.h"
#include "operators/overlap.h"
#include "operators/inclusion.h"
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

/*
 * Query:
 * node & merged:pos="PPER" & node & mmax:relation="anaphoric" & node & node & mmax:relation="anaphoric"
& #1 >[func="ON"] #3
& #3 >* #2
& #2 _i_ #4
& #5 >[func="ON"] #6
& #6 >* #7
& #4 ->anaphoric #7
*/
TEST_F(SearchTestTueBaDZ, DISABLED_Benchmark1) {

  AnnotationNameSearch n1(db, annis_ns, annis_node_name);
  std::shared_ptr<AnnoIt> n2(std::make_shared<AnnotationNameSearch>(db, "merged", "pos", "PPER"));
  AnnotationNameSearch n3(db, annis_ns, annis_node_name);
  std::shared_ptr<AnnoIt> n4(std::make_shared<AnnotationNameSearch>(db, "mmax", "relation", "anaphoric"));
  AnnotationNameSearch n5(db, annis_ns, annis_node_name);
  AnnotationNameSearch n6(db, annis_ns, annis_node_name);
  std::shared_ptr<AnnoIt> n7(std::make_shared<AnnotationNameSearch>(db, "mmax", "relation", "anaphoric"));

  const EdgeDB* edbAnaphoric = db.getEdgeDB(ComponentType::POINTING, "mmax", "anaphoric");
  std::shared_ptr<BinaryOperatorIterator> n2_incl_n4(std::make_shared<Inclusion>(db, n2, n4));

  std::shared_ptr<AnnoIt> wrap_n2_n4(std::make_shared<JoinWrapIterator>(n2_incl_n4));
  NestedLoopJoin n4_anaphoric_n7(edbAnaphoric, wrap_n2_n4, n7);

  unsigned int counter=0;

  EXPECT_EQ(373436u, counter);
}


#endif // SEARCHTESTTUEBADAZ_H
