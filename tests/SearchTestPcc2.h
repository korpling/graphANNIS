#ifndef SEARCHTESTPCC2_H
#define SEARCHTESTPCC2_H

#include "gtest/gtest.h"
#include "db.h"
#include "annotationsearch.h"
#include "operators/defaultjoins.h"
#include "operators/overlap.h"
#include "operators/inclusion.h"
#include "query.h"

#include <vector>
#include <boost/format.hpp>

using namespace annis;

class SearchTestPcc2 : public ::testing::Test {
 protected:
  DB db;
  SearchTestPcc2()
  {
  }

  virtual ~SearchTestPcc2() {
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
//    bool loadedDB = db.loadRelANNIS(dataDir + "/pcc2_v6_relANNIS");
    bool loadedDB = db.load(dataDir + "/pcc2");
    EXPECT_EQ(true, loadedDB);

  }

  virtual void TearDown() {
    // Code here will be called immediately after each test (right
    // before the destructor).
  }

  // Objects declared here can be used by all tests in the test case for Foo.
};

TEST_F(SearchTestPcc2, CatSearch) {
  AnnotationNameSearch search(db, "cat");
  unsigned int counter=0;
  while(search.hasNext())
  {
    Match m = search.next();
    ASSERT_STREQ("cat", db.strings.str(m.anno.name).c_str());
    ASSERT_STREQ("tiger", db.strings.str(m.anno.ns).c_str());
    counter++;
  }

  EXPECT_EQ(155u, counter);
}

TEST_F(SearchTestPcc2, MMaxAnnos) {

  AnnotationNameSearch n1(db, "mmax", "ambiguity", "not_ambig");
  AnnotationNameSearch n2(db, "mmax", "complex_np", "yes");

  unsigned int counter=0;
  while(n1.hasNext())
  {
    Match m = n1.next();
    ASSERT_STREQ("mmax", db.strings.str(m.anno.ns).c_str());
    ASSERT_STREQ("ambiguity", db.strings.str(m.anno.name).c_str());
    ASSERT_STREQ("not_ambig", db.strings.str(m.anno.val).c_str());
    counter++;
  }

  EXPECT_EQ(73u, counter);

  counter=0;
  while(n2.hasNext())
  {
    Match m = n2.next();
    ASSERT_STREQ("mmax", db.strings.str(m.anno.ns).c_str());
    ASSERT_STREQ("complex_np", db.strings.str(m.anno.name).c_str());
    ASSERT_STREQ("yes", db.strings.str(m.anno.val).c_str());
    counter++;
  }

  EXPECT_EQ(17u, counter);
}

TEST_F(SearchTestPcc2, TokenIndex) {
  std::shared_ptr<AnnoIt> n1(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Die"));
  std::shared_ptr<AnnoIt> n2(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Jugendlichen"));

  unsigned int counter=0;

  Component c = Init::initComponent(ComponentType::ORDERING, annis_ns, "");
  const EdgeDB* edb = db.getEdgeDB(c);
  if(edb != NULL)
  {
    NestedLoopJoin join(edb, n1, n2);
    for(BinaryMatch match = join.next(); match.found; match = join.next())
    {
      counter++;
    }
  }

  EXPECT_EQ(2u, counter);
}

TEST_F(SearchTestPcc2, IsConnectedRange) {
  std::shared_ptr<AnnoIt> n1(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Jugendlichen"));
  std::shared_ptr<AnnoIt> n2(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Musikcafé"));

  unsigned int counter=0;

  NestedLoopJoin join(db.getEdgeDB(ComponentType::ORDERING, annis_ns, ""), n1, n2, 3, 10);
  for(BinaryMatch match = join.next(); match.found; match = join.next())
  {
    counter++;
  }

  EXPECT_EQ(1u, counter);
}

TEST_F(SearchTestPcc2, DepthFirst) {
    std::shared_ptr<AnnoIt> n1(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Tiefe"));
    Annotation anno2 = Init::initAnnotation(db.strings.add("node_name"), 0, db.strings.add(annis_ns));

    unsigned int counter=0;

    Component c = Init::initComponent(ComponentType::ORDERING, annis_ns, "");
    const EdgeDB* edb = db.getEdgeDB(c);
    if(edb != NULL)
    {
      SeedJoin join(db, edb, n1, anno2, 2, 10);
      for(BinaryMatch match=join.next(); match.found; match = join.next())
      {
        counter++;
      }
    }

  EXPECT_EQ(9u, counter);
}

// exmaralda:Inf-Stat="new" _o_ exmaralda:PP
TEST_F(SearchTestPcc2, TestQueryOverlap1) {
  std::shared_ptr<CacheableAnnoIt> n1(std::make_shared<AnnotationNameSearch>(db, "exmaralda", "Inf-Stat", "new"));
  std::shared_ptr<CacheableAnnoIt> n2(std::make_shared<AnnotationNameSearch>(db, "exmaralda", "PP"));

  std::shared_ptr<BinaryOperatorIterator> join(std::make_shared<SeedOverlap>(db));
  join->init(n1, n2);

  Query q;
  q.addNode(n1);
  q.addNode(n2);
  q.addOperator(join, 0, 1);

  unsigned int counter=0;
  while(q.hasNext())
  {
    auto m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(3u, counter);
}

// mmax:ambiguity="not_ambig" _o_ mmax:complex_np="yes"
TEST_F(SearchTestPcc2, TestQueryOverlap2) {
  std::shared_ptr<AnnoIt> n1(std::make_shared<AnnotationNameSearch>(db, "mmax", "ambiguity", "not_ambig"));
  std::shared_ptr<AnnoIt> n2(std::make_shared<AnnotationNameSearch>(db, "mmax", "complex_np", "yes"));

  SeedOverlap join(db);
  join.init(n1, n2);

  unsigned int counter=0;
  for(BinaryMatch m=join.next(); m.found; m=join.next())
  {
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m.lhs.node) % db.getNodeName(m.rhs.node)).str());
    counter++;
  }

  EXPECT_EQ(47u, counter);
}

// mmax:ambiguity="not_ambig" _i_ mmax:complex_np="yes"
TEST_F(SearchTestPcc2, TestQueryInclude) {
  std::shared_ptr<AnnoIt> n1(std::make_shared<AnnotationNameSearch>(db, "mmax", "ambiguity", "not_ambig"));
  std::shared_ptr<AnnoIt> n2(std::make_shared<AnnotationNameSearch>(db, "mmax", "complex_np", "yes"));

  Inclusion join(db, n1, n2);

  unsigned int counter=0;
  for(BinaryMatch m=join.next(); m.found; m=join.next())
  {
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m.lhs.node) % db.getNodeName(m.rhs.node)).str());
    counter++;
  }

  EXPECT_EQ(23u, counter);
}



#endif // SEARCHTESTPCC2_H
