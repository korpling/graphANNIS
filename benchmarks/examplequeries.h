#ifndef EXAMPLEQUERIES
#define EXAMPLEQUERIES

#include <db.h>
#include <query.h>
#include <exactannosearch.h>
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
    q.addNode(std::make_shared<ExactAnnoSearch>(db, "default_ns", "pos", "NN"));
    q.addNode(std::make_shared<ExactAnnoSearch>(db, "default_ns", "norm", "Blumen"));

    q.addOperator(std::make_shared<annis::Inclusion>(db), 1, 0);
    return q;
  }

  static Query PosNNOverlapsNormBlumen(const DB& db)
  {
    Query q(db);
    auto n1 = q.addNode(std::make_shared<ExactAnnoSearch>(db, "default_ns", "pos", "NN"));
    auto n2 = q.addNode(std::make_shared<ExactAnnoSearch>(db, "default_ns", "norm", "Blumen"));
    q.addOperator(std::make_shared<Overlap>(db), n2, n1);
    return q;
  }

  static Query NNPreceedingART(const DB& db)
  {
    Query q(db);
    q.addNode(std::make_shared<ExactAnnoSearch>(db, "default_ns", "pos", "NN"));
    q.addNode(std::make_shared<ExactAnnoSearch>(db, "default_ns", "pos", "ART"));

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
    auto n1 = q.addNode(std::make_shared<ExactAnnoSearch>(db, "tiger", "cat", "S"));
    auto n2 = q.addNode(std::make_shared<ExactAnnoSearch>(db, annis_ns, annis_tok, "Bilharziose"));

    q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n1, n2);

    return q;
  }

  static Query NNPreARTPreNN(const DB& db)
  {

    Query q(db);
    q.addNode(std::make_shared<ExactAnnoSearch>(db, "tiger", "pos", "NN"));
    q.addNode(std::make_shared<ExactAnnoSearch>(db, "tiger", "pos", "ART"));
    q.addNode(std::make_shared<ExactAnnoSearch>(db, "tiger", "pos", "NN"));

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

};
} // end namespace annis;
#endif // EXAMPLEQUERIES

