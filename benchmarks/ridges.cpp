#include "benchmark.h"

char ridgesCorpus[] = "ridges";

class RidgesFixture : public CorpusFixture<true, ridgesCorpus>
{
public:

  virtual ~RidgesFixture() {}
};

class RidgesPrePostFixture : public CorpusFixture<true, ridgesCorpus>
{
public:

  RidgesPrePostFixture()
  {
    Component test = {ComponentType::COVERAGE, annis_ns, ""};
    addOverride(ComponentType::COVERAGE, annis_ns, "", "prepostorder");
    addOverride(ComponentType::COVERAGE, "default_ns", "", "prepostorder");
  }

  virtual ~RidgesPrePostFixture() {}
};

class RidgesFallbackFixture : public CorpusFixture<false, ridgesCorpus>
{
public:
  virtual ~RidgesFallbackFixture() {}
};


// pos="NN" & norm="Blumen" & #1 _i_ #2
BASELINE_F(Ridges_PosNNIncludesNormBlumen, Fallback, RidgesFallbackFixture, 5, 1)
{

  Query q(getDB());
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "norm", "Blumen"));

  q.addOperator(std::make_shared<annis::Inclusion>(getDB()), 1, 0);

  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 152u);
}
BENCHMARK_F(Ridges_PosNNIncludesNormBlumen, Optimized, RidgesFixture, 5, 1)
{

  Query q(getDB());
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "norm", "Blumen"));

  q.addOperator(std::make_shared<annis::Inclusion>(getDB()), 1, 0);

  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 152u);
}


BENCHMARK_F(Ridges_PosNNIncludesNormBlumen, PrePost, RidgesPrePostFixture, 5, 1)
{

  Query q(getDB());
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "norm", "Blumen"));

  q.addOperator(std::make_shared<annis::Inclusion>(getDB()), 1, 0);

  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 152u);
}

// pos="NN" & norm="Blumen" & #2 _o_ #1
BASELINE_F(Ridges_PosNNOverlapsNormBlumen, Fallback, RidgesFallbackFixture, 5, 1) {

  Query q(getDB());
  auto n1 = q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "NN"));
  auto n2 = q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "norm", "Blumen"));
  q.addOperator(std::make_shared<Overlap>(getDB()), n2, n1);

  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 152u);
}

BENCHMARK_F(Ridges_PosNNOverlapsNormBlumen, Optimized, RidgesFixture, 5, 1) {

  Query q(getDB());
  auto n1 = q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "NN"));
  auto n2 = q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "norm", "Blumen"));
  q.addOperator(std::make_shared<Overlap>(getDB()), n2, n1);

  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 152u);
}


BENCHMARK_F(Ridges_PosNNOverlapsNormBlumen, PrePost, RidgesPrePostFixture, 5, 1) {

  Query q(getDB());
  auto n1 = q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "NN"));
  auto n2 = q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "norm", "Blumen"));
  q.addOperator(std::make_shared<Overlap>(getDB()), n2, n1);

  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 152u);
}

// pos="NN" .2,10 pos="ART"
BASELINE_F(Ridges_NNPreceedingART, Fallback, RidgesFallbackFixture, 5, 1) {

  Query q(getDB());
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "ART"));

  q.addOperator(std::make_shared<Precedence>(getDB(), 2, 10), 0, 1);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 21911u);
}
BENCHMARK_F(Ridges_NNPreceedingART, Optimized, RidgesFixture, 5, 1) {

  Query q(getDB());
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "ART"));

  q.addOperator(std::make_shared<Precedence>(getDB(), 2, 10), 0, 1);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 21911u);
}

BENCHMARK_F(Ridges_NNPreceedingART, PrePost, RidgesPrePostFixture, 5, 1) {

  Query q(getDB());
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "NN"));
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), "default_ns", "pos", "ART"));

  q.addOperator(std::make_shared<Precedence>(getDB(), 2, 10), 0, 1);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 21911u);
}

// tok .2,10 tok
BASELINE_F(Ridges_TokPreceedingTok, Fallback, RidgesFallbackFixture, 5, 1) {

  Query q(getDB());
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), annis::annis_ns,annis::annis_tok));
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), annis::annis_ns,annis::annis_tok));


  q.addOperator(std::make_shared<Precedence>(getDB(), 2, 10), 0, 1);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 1386828u);
}
BENCHMARK_F(Ridges_TokPreceedingTok, Optimized, RidgesFixture, 5, 1) {

  Query q(getDB());
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), annis::annis_ns,annis::annis_tok));
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), annis::annis_ns,annis::annis_tok));


  q.addOperator(std::make_shared<Precedence>(getDB(), 2, 10), 0, 1);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 1386828u);
}

BENCHMARK_F(Ridges_TokPreceedingTok, PrePost, RidgesPrePostFixture, 5, 1) {

  Query q(getDB());
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), annis::annis_ns,annis::annis_tok));
  q.addNode(std::make_shared<AnnotationNameSearch>(getDB(), annis::annis_ns,annis::annis_tok));


  q.addOperator(std::make_shared<Precedence>(getDB(), 2, 10), 0, 1);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
  assert(counter == 1386828u);
}

