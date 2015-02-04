#include "benchmark.h"
#include "examplequeries.h"

char tigerCorpus[] = "tiger2";

class TigerFixture : public CorpusFixture<false, tigerCorpus>
{
public:
  DBGETTER

  virtual ~TigerFixture() {}
};
class TigerFallbackFixture : public CorpusFixture<true, tigerCorpus>
{
public:
  DBGETTER

  virtual ~TigerFallbackFixture() {}
};


BASELINE_F(Tiger_Cat, Fallback, TigerFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(Cat, getDB(), 373436u);
}

BENCHMARK_F(Tiger_Cat, Optimized, TigerFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(Cat, getDB(), 373436u);
}

// cat="S" & tok="Bilharziose" & #1 >* #2
BASELINE_F(Tiger_BilharzioseSentence,  Fallback, TigerFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(BilharzioseSentence, getDB(), 21u);
}
BENCHMARK_F(Tiger_BilharzioseSentence, Optimized, TigerFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(BilharzioseSentence, getDB(), 21u);
}

// pos="NN" .2,10 pos="ART" . pos="NN"
BASELINE_F(Tiger_NNPreARTPreNN, Fallback, TigerFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreARTPreNN, getDB(), 114042u);
}

BENCHMARK_F(Tiger_NNPreARTPreNN, Optimized, TigerFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreARTPreNN, getDB(), 114042u);
}

// cat=/(.P)/ >* /A.*/
BASELINE_F(Tiger_RegexDom, Fallback, TigerFallbackFixture , 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(RegexDom, getDB(), 36294u);
}

BENCHMARK_F(Tiger_RegexDom, Optimized, TigerFixture , 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(RegexDom, getDB(), 36294u);
}

