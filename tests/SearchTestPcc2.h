#ifndef SEARCHTESTPCC2_H
#define SEARCHTESTPCC2_H

#include "gtest/gtest.h"
#include "db.h"
#include "annotationsearch.h"
#include "operators/overlap.h"
#include "operators/inclusion.h"
#include "operators/precedence.h"
#include "operators/pointing.h"
#include "operators/dominance.h"
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

  unsigned int counter=0;

  Query q(db);

  auto n1 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Die"));
  auto n2 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Jugendlichen"));

  q.addOperator(std::make_shared<Precedence>(db), n1, n2);

  while(q.hasNext())
  {
    q.next();
    counter++;
  }

  EXPECT_EQ(2u, counter);
}

TEST_F(SearchTestPcc2, IsConnectedRange) {

  Query q(db);

  auto n1 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Jugendlichen"));
  auto n2 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Musikcafé"));

  unsigned int counter=0;

  q.addOperator(std::make_shared<Precedence>(db, 3, 10), n1 , n2);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }

  EXPECT_EQ(1u, counter);
}

TEST_F(SearchTestPcc2, DepthFirst) {

  unsigned int counter=0;

  Query q(db);
  auto n1 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Tiefe"));
  auto n2 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok));

  q.addOperator(std::make_shared<Precedence>(db, 2, 10), n1, n2);

  while(q.hasNext())
  {
    q.next();
    counter++;
  }

  EXPECT_EQ(9u, counter);
}

// exmaralda:Inf-Stat="new" _o_ exmaralda:PP
TEST_F(SearchTestPcc2, TestQueryOverlap1) {

  Query q(db);
  auto n1 = q.addNode(std::make_shared<AnnotationNameSearch>(db, "exmaralda", "Inf-Stat", "new"));
  auto n2 = q.addNode(std::make_shared<AnnotationNameSearch>(db, "exmaralda", "PP"));
  q.addOperator(std::make_shared<Overlap>(db), n1, n2);

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

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "ambiguity", "not_ambig"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "complex_np", "yes"));
  q.addOperator(std::make_shared<Overlap>(db), 0, 1);

  unsigned int counter=0;
  while(q.hasNext())
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(47u, counter);
}

// mmax:ambiguity="not_ambig" _i_ mmax:complex_np="yes"
TEST_F(SearchTestPcc2, TestQueryInclude) {

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "ambiguity", "not_ambig"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "complex_np", "yes"));
  q.addOperator(std::make_shared<Inclusion>(db), 0, 1);

  unsigned int counter=0;
  while(q.hasNext())
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(23u, counter);
}

// Should test query
// pos="NN" .2,20 pos="ART"
TEST_F(SearchTestPcc2, Precedence) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "exmaralda", "Inf-Stat", "acc-sit"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "exmaralda", "NP", "NP"));

  q.addOperator(std::make_shared<Precedence>(db, 1, 500), 0, 1);

  while(q.hasNext() && counter < 2000)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(27u, counter);
}

// Should test query
// mmax:np_form="defnp" & mmax:np_form="pper"  & #2 ->anaphor_antecedent * #1
TEST_F(SearchTestPcc2, IndirectPointing) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "np_form", "defnp"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "np_form", "pper"));

  q.addOperator(std::make_shared<Pointing>(db, "", "anaphor_antecedent", 1, uintmax), 1, 0);

  while(q.hasNext() && counter < 2000)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(13u, counter);
}

TEST_F(SearchTestPcc2, IndirectPointingNested) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "np_form", "defnp"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "np_form", "pper"));

  q.addOperator(std::make_shared<Pointing>(db, "", "anaphor_antecedent", 1, uintmax), 1, 0, true);

  while(q.hasNext() && counter < 2000)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(13u, counter);
}

// Should test query
// mmax:np_form="defnp" & mmax:np_form="pper"  & #2 ->anaphor_antecedent * #1
TEST_F(SearchTestPcc2, DirectPointing) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "np_form", "defnp"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "np_form", "pper"));

  q.addOperator(std::make_shared<Pointing>(db, "", "anaphor_antecedent", 1, 1), 1, 0);

  while(q.hasNext() && counter < 2000)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(5u, counter);
}

TEST_F(SearchTestPcc2, DirectPointingNested) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "np_form", "defnp"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "np_form", "pper"));

  q.addOperator(std::make_shared<Pointing>(db, "", "anaphor_antecedent", 1, 1), 1, 0, true);

  while(q.hasNext() && counter < 2000)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(5u, counter);
}

// Should test query
// pos="ADJD" & "." & #1 ->dep[func="punct"] #2
TEST_F(SearchTestPcc2, DirectPointingWithAnno) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "tiger", "pos", "ADJD"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "."));

  std::shared_ptr<Operator> op =
      std::make_shared<Pointing>(
        db, "", "dep",
        Init::initAnnotation(db.strings.add("func"), db.strings.add("punct")));
  q.addOperator(op, 0, 1);

  while(q.hasNext() && counter < 2000)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(4u, counter);
}

TEST_F(SearchTestPcc2, DirectPointingWithAnnoNested) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "tiger", "pos", "ADJD"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "."));

  std::shared_ptr<Operator> op =
      std::make_shared<Pointing>(
        db, "", "dep",
        Init::initAnnotation(db.strings.add("func"), db.strings.add("punct")));
  q.addOperator(op, 0, 1, true);

  while(q.hasNext() && counter < 2000)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(4u, counter);
}

// Should test query
// cat="S" >2,4 cat
TEST_F(SearchTestPcc2, RangedDominance) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "tiger", "cat", "S"));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, "tiger", "cat"));

  q.addOperator(std::make_shared<Dominance>(db, "", "", 2, 4), 0, 1);

  while(q.hasNext() && counter < 2000)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(93u, counter);
}

// Should test query
// node >2,4 node
TEST_F(SearchTestPcc2, MultiDominance) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns,
                                                   annis_node_name));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns,
                                                   annis_node_name));

  q.addOperator(std::make_shared<Dominance>(db, "", "", 2, 4), 0, 1);

  while(q.hasNext() && counter < 4000)
  {
    std::vector<Match> m = q.next();
    HL_INFO(logger, (boost::format("match\t%1%\t%2%") % db.getNodeName(m[0].node) % db.getNodeName(m[1].node)).str());
    counter++;
  }

  EXPECT_EQ(2072u, counter);
}

TEST_F(SearchTestPcc2, Profile) {

  unsigned int counter=0;

  Query q(db);
  q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns,
                                                   annis_node_name));
  q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns,
                                                   annis_node_name));

  q.addOperator(std::make_shared<Pointing>(db, "", "dep", Init::initAnnotation(db.strings.add("func"), db.strings.add("sbj"))), 0, 1);

  while(q.hasNext() && counter < 5000)
  {
    std::vector<Match> m = q.next();
    counter++;
  }

  EXPECT_EQ(38, counter);
}

#endif // SEARCHTESTPCC2_H
