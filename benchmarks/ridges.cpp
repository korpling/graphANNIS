#include "benchmark.h"
#include "examplequeries.h"
#include "graphstorageregistry.h"

char ridgesCorpus[] = "ridges";

class RidgesFixture : public CorpusFixture<false, ridgesCorpus>
{
public:
  DBGETTER
  virtual ~RidgesFixture() {}
};

class RidgesPrePostFixture : public CorpusFixture<false, ridgesCorpus>
{
public:

  RidgesPrePostFixture()
  {
    addOverride(ComponentType::COVERAGE, annis_ns, "", GraphStorageRegistry::prepostorderO32L32);
    addOverride(ComponentType::COVERAGE, "default_ns", "", GraphStorageRegistry::prepostorderO32L32);
    addOverride(ComponentType::ORDERING, annis_ns, "", GraphStorageRegistry::prepostorderO32L32);
//    addOverride(ComponentType::LEFT_TOKEN, annis_ns, "", "prepostorder");
//    addOverride(ComponentType::RIGHT_TOKEN, annis_ns, "", "prepostorder");
  }

  DBGETTER

  virtual ~RidgesPrePostFixture() {}
};

class RidgesFallbackFixture : public CorpusFixture<true, ridgesCorpus>
{
public:
  DBGETTER

  virtual ~RidgesFallbackFixture() {}
};


// pos="NN" & norm="Blumen" & #1 _i_ #2
BASELINE_F(Ridges_PosNNIncludesNormBlumen, Fallback, RidgesFallbackFixture, 5, 5)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNIncludesNormBlumen, getDB(), 152u);
}

BENCHMARK_F(Ridges_PosNNIncludesNormBlumen, Optimized, RidgesFixture, 5, 5)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNIncludesNormBlumen, getDB(), 152u);
}

BENCHMARK_F(Ridges_PosNNIncludesNormBlumen, PrePost, RidgesPrePostFixture, 5, 5)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNIncludesNormBlumen, getDB(), 152u);
}

// pos="NN" & norm="Blumen" & #2 _o_ #1
BASELINE_F(Ridges_PosNNOverlapsNormBlumen, Fallback, RidgesFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNOverlapsNormBlumen, getDB(), 152u);
}

BENCHMARK_F(Ridges_PosNNOverlapsNormBlumen, Optimized, RidgesFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNOverlapsNormBlumen, getDB(), 152u);
}


BENCHMARK_F(Ridges_PosNNOverlapsNormBlumen, PrePost, RidgesPrePostFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNOverlapsNormBlumen, getDB(), 152u);
}

// pos="NN" .2,10 pos="ART"
BASELINE_F(Ridges_NNPreceedingART, Fallback, RidgesFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreceedingART, getDB(), 21911u);
}
BENCHMARK_F(Ridges_NNPreceedingART, Optimized, RidgesFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreceedingART, getDB(), 21911u);
}

BENCHMARK_F(Ridges_NNPreceedingART, PrePost, RidgesPrePostFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreceedingART, getDB(), 21911u);
}

// tok .2,10 tok
BASELINE_F(Ridges_TokPreceedingTok, Fallback, RidgesFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(TokPreceedingTok, getDB(), 1386828u);
}
BENCHMARK_F(Ridges_TokPreceedingTok, Optimized, RidgesFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(TokPreceedingTok, getDB(), 1386828u);
}

BENCHMARK_F(Ridges_TokPreceedingTok, PrePost, RidgesPrePostFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(TokPreceedingTok, getDB(), 1386828u);
}

