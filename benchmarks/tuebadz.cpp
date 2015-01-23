#include "benchmark.h"

char tuebaCorpus[] = "tuebadz6";

class TuebaFixture : public CorpusFixture<true, tuebaCorpus>
{
public:
  virtual ~TuebaFixture() {}
};
class TuebaFallbackFixture : public CorpusFixture<false, tuebaCorpus>
{
public:
  virtual ~TuebaFallbackFixture() {}
};

/*
node & merged:pos="PPER" & node & mmax:relation="anaphoric" & node & node & mmax:relation="anaphoric"
& #1 >[func="ON"] #3
& #3 >* #2
& #2 _i_ #4
& #5 >[func="ON"] #6
& #6 >* #7
& #4 ->anaphoric #7
*/
BASELINE_F(Tueba_Complex1, Fallback, TuebaFallbackFixture, 5, 1) {

  Query q(db);
  auto n1 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_node_name));
  auto n2 = q.addNode(std::make_shared<AnnotationNameSearch>(db, "merged", "pos", "PPER"));
  auto n3 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_node_name));
  auto n4 = q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "relation", "anaphoric"));
  auto n5 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_node_name));
  auto n6 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_node_name));
  auto n7 = q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "relation", "anaphoric"));

  Annotation funcOnAnno =
      Init::initAnnotation(db.strings.add("func"), db.strings.add("ON"));

  q.addOperator(std::make_shared<Inclusion>(db), n2, n4);
  q.addOperator(std::make_shared<Pointing>(db, "", "anaphoric"), n4, n7);
  q.addOperator(std::make_shared<Dominance>(db, "", "", funcOnAnno), n1, n3);
  q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n3, n2);
  q.addOperator(std::make_shared<Dominance>(db, "", "", funcOnAnno), n5, n6);
  q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n6, n7);

  unsigned int counter=0;
  while(q.hasNext() && counter < 10u)
  {
    q.next();
    counter++;
  }
  assert(counter == 0u);
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
BENCHMARK_F(Tueba_Complex1, Optimized, TuebaFixture, 5, 1) {

  Query q(db);
  auto n1 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_node_name));
  auto n2 = q.addNode(std::make_shared<AnnotationNameSearch>(db, "merged", "pos", "PPER"));
  auto n3 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_node_name));
  auto n4 = q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "relation", "anaphoric"));
  auto n5 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_node_name));
  auto n6 = q.addNode(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_node_name));
  auto n7 = q.addNode(std::make_shared<AnnotationNameSearch>(db, "mmax", "relation", "anaphoric"));

  Annotation funcOnAnno =
      Init::initAnnotation(db.strings.add("func"), db.strings.add("ON"));

  q.addOperator(std::make_shared<Inclusion>(db), n2, n4);
  q.addOperator(std::make_shared<Pointing>(db, "", "anaphoric"), n4, n7);
  q.addOperator(std::make_shared<Dominance>(db, "", "", funcOnAnno), n1, n3);
  q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n3, n2);
  q.addOperator(std::make_shared<Dominance>(db, "", "", funcOnAnno), n5, n6);
  q.addOperator(std::make_shared<Dominance>(db, "", "", 1, uintmax), n6, n7);

  unsigned int counter=0;
  while(q.hasNext() && counter < 10u)
  {
    q.next();
    counter++;
  }
  assert(counter == 0u);
}
