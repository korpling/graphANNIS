#include "benchmark.h"
#include "examplequeries.h"

char parlamentCorpus[] = "parlament";

class ParlamentFixture : public CorpusFixture<false, parlamentCorpus>
{
public:
  DBGETTER

  virtual ~ParlamentFixture() {}
};
class ParlamentFallbackFixture : public CorpusFixture<true, parlamentCorpus>
{
public:
  DBGETTER

  virtual ~ParlamentFallbackFixture() {}
};


BASELINE_F(Parlament_JederObwohl, Fallback, ParlamentFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(JederObwohl, getDB(), 7);
}

BENCHMARK_F(Parlament_JederObwohl, Fallback, ParlamentFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(JederObwohl, getDB(), 7);
}


