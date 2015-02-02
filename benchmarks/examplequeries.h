#ifndef EXAMPLEQUERIES
#define EXAMPLEQUERIES

#include <db.h>
#include <query.h>
#include <exactannovaluesearch.h>
#include <exactannokeysearch.h>
#include <regexannosearch.h>
#include <operators/precedence.h>
#include <operators/inclusion.h>
#include <operators/dominance.h>
#include <operators/overlap.h>
#include <operators/pointing.h>

namespace annis
{

#define ANNIS_EXEC_QUERY(name, db, count) {\
  counter = 0;\
  Query q=annis::ExampleQueries::name(db);\
  while(q.hasNext())\
  {\
    q.next();\
    counter++;\
  }\
  if(counter != count) {\
  std::cerr << "FATAL ERROR: query " << #name << " should have count " << count << " but was " << counter << std::endl;\
  std::cerr << "" << __FILE__ << ":" << __LINE__ << std::endl;\
  exit(-1);}\
}

class ExampleQueries
{
public:
  static Query PosNNIncludesNormBlumen(const DB& db)
  {
    Query q(db);
    q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "NN"));
    q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "norm", "Blumen"));

    q.addOperator(std::make_shared<annis::Inclusion>(db), 1, 0);
    return q;
  }

  static Query PosNNOverlapsNormBlumen(const DB& db)
  {
    Query q(db);
    auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "NN"));
    auto n2 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "norm", "Blumen"));
    q.addOperator(std::make_shared<Overlap>(db), n2, n1);
    return q;
  }

  static Query NNPreceedingART(const DB& db)
  {
    Query q(db);
    q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "NN"));
    q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "default_ns", "pos", "ART"));

    q.addOperator(std::make_shared<Precedence>(db, 2, 10), 0, 1);
    return q;
  }

  static Query TokPreceedingTok(const DB& db)
  {

    Query q(db);
    q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis::annis_ns,annis::annis_tok));
    q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis::annis_ns,annis::annis_tok));


    q.addOperator(std::make_shared<Precedence>(db, 2, 10), 0, 1);

    return q;
  }

  static Query Cat(const DB& db)
  {
    Query q(db);
    q.addNode(std::make_shared<ExactAnnoKeySearch>(db, "cat"));
    return q;
  }

  static Query BilharzioseSentence(const DB& db)
  {
    Query q(db);
    auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "tiger", "cat", "S"));
    auto n2 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_tok, "Bilharziose"));

    q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n1, n2);

    return q;
  }

  static Query NNPreARTPreNN(const DB& db)
  {

    Query q(db);
    q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "tiger", "pos", "NN"));
    q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "tiger", "pos", "ART"));
    q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "tiger", "pos", "NN"));

    q.addOperator(std::make_shared<Precedence>(db, 2,10), 0, 1);
    q.addOperator(std::make_shared<Precedence>(db), 1, 2);

    return q;
  }

  static Query RegexDom(const DB& db)
  {
    Query q(db);
    auto n1 = q.addNode(std::make_shared<RegexAnnoSearch>(db,
                                                          "cat",".P"));
    auto n2 = q.addNode(std::make_shared<RegexAnnoSearch>(db,
                                                          annis_ns, annis_tok,
                                                         "A.*"));

    q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n1, n2);

    return q;
  }


  /*
  node & merged:pos="PPER" & node & mmax:relation="anaphoric" & node & node & mmax:relation="anaphoric"
  & #1 >[func="ON"] #3
  & #3 >* #2
  & #2 _i_ #4
  & #5 >[func="ON"] #6
  & #6 >* #7
  & #4 ->anaphoric #7
  */
  static Query Mixed1(const DB& db)
  {
    Query q(db);
    auto n1 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_node_name));
    auto n2 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "merged", "pos", "PPER"));
    auto n3 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_node_name));
    auto n4 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "mmax", "relation", "anaphoric"));
    auto n5 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_node_name));
    auto n6 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_node_name));
    auto n7 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "mmax", "relation", "anaphoric"));

    Annotation funcOnAnno =
        Init::initAnnotation(db.strings.findID("func").first, db.strings.findID("ON").first);

    q.addOperator(std::make_shared<Inclusion>(db), n2, n4);
    q.addOperator(std::make_shared<Pointing>(db, "", "anaphoric"), n4, n7);
    q.addOperator(std::make_shared<Dominance>(db, "", "", funcOnAnno), n1, n3);
    q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n3, n2);
    q.addOperator(std::make_shared<Dominance>(db, "", "", funcOnAnno), n5, n6);
    q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n6, n7);

    return q;
  }

  static Query NodeDom(const DB& db, unsigned int maxDistance=uintmax)
  {
    Query q(db);
    auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db,
                                                          "tiger","cat", "TOP"));
    auto n2 = q.addNode(std::make_shared<ExactAnnoKeySearch>(db,
                                                          annis_ns, annis_node_name));

    q.addOperator(std::make_shared<Dominance>(db, "", "", 1, maxDistance), n1, n2);

    return q;
  }

  static Query PPERIncludesAnaphoric(const DB& db)
  {
    Query q(db);
    auto n1 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "merged", "pos", "PPER"));
    auto n2 = q.addNode(std::make_shared<ExactAnnoValueSearch>(db, "mmax", "relation", "anaphoric"));

    q.addOperator(std::make_shared<Inclusion>(db), n1, n2);
    return q;
  }

};
} // end namespace annis;
#endif // EXAMPLEQUERIES

