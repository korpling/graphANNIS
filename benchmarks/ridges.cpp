#include "benchmark.h"
#include "examplequeries.h"
#include "graphstorageregistry.h"


class RidgesFixture : public CorpusFixture<false>
{
public:
  DBGETTER
  
  RidgesFixture() : CorpusFixture<false>("ridges")
  {
    
  }
  
  virtual ~RidgesFixture() {}
};

class RidgesPrePostFixture : public CorpusFixture<false>
{
public:
  
  RidgesPrePostFixture() : CorpusFixture<false>("ridges")
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

class RidgesFallbackFixture : public CorpusFixture<true>
{
public:
  DBGETTER
  
  RidgesFallbackFixture() : CorpusFixture<true>("ridges")
  {
    
  }

  virtual ~RidgesFallbackFixture() {}
};


// pos="NN" & norm="Blumen" & #1 _i_ #2
BASELINE_F(BIN_ridges, Fallback, RidgesFallbackFixture, 5, 5)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNIncludesNormBlumen, getDB(), 152u);
}

BENCHMARK_F(BIN_ridges, Optimized, RidgesFixture, 5, 5)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNIncludesNormBlumen, getDB(), 152u);
}

BENCHMARK_F(BIN_ridges, PrePost, RidgesPrePostFixture, 5, 5)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNIncludesNormBlumen, getDB(), 152u);
}

// pos="NN" & norm="Blumen" & #2 _o_ #1
BASELINE_F(BON_ridges, Fallback, RidgesFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNOverlapsNormBlumen, getDB(), 152u);
}

BENCHMARK_F(BON_ridges, Optimized, RidgesFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNOverlapsNormBlumen, getDB(), 152u);
}


BENCHMARK_F(BON_ridges, PrePost, RidgesPrePostFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(PosNNOverlapsNormBlumen, getDB(), 152u);
}

// pos="NN" .2,10 pos="ART"
BASELINE_F(NPA_ridges, Fallback, RidgesFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreceedingART, getDB(), 21911u);
}
BENCHMARK_F(NPA_ridges, Optimized, RidgesFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreceedingART, getDB(), 21911u);
}

BENCHMARK_F(NPA_ridges, PrePost, RidgesPrePostFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreceedingART, getDB(), 21911u);
}

// tok .2,10 tok
BASELINE_F(TOK_ridges, Fallback, RidgesFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(TokPreceedingTok, getDB(), 1386828u);
}
BENCHMARK_F(TOK_ridges, Optimized, RidgesFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(TokPreceedingTok, getDB(), 1386828u);
}

BENCHMARK_F(TOK_ridges, PrePost, RidgesPrePostFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(TokPreceedingTok, getDB(), 1386828u);
}

