#include "benchmark.h"
#include "examplequeries.h"


class ParlamentFixture : public CorpusFixture<false>
{
public:
  DBGETTER
  
  ParlamentFixture() : CorpusFixture<false>("parlament")
  {
    
  }

  virtual ~ParlamentFixture() {}
};
class ParlamentFallbackFixture : public CorpusFixture<true>
{
public:
  DBGETTER

  ParlamentFallbackFixture() : CorpusFixture<true>("parlament")
  {
    
  }
  
  virtual ~ParlamentFallbackFixture() {}
};


BASELINE_F(JPO_parlament, Fallback, ParlamentFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(JederObwohl, getDB(), 4);
}

BENCHMARK_F(JPO_parlament, Optimized, ParlamentFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(JederObwohl, getDB(), 4);
}


