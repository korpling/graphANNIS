#include "benchmark.h"
#include "examplequeries.h"


class TigerFixture : public CorpusFixture<false>
{
public:
  DBGETTER

  TigerFixture() :  CorpusFixture<false>("tiger2")
  {
    
  }
  
  virtual ~TigerFixture() {}
};
class TigerFallbackFixture : public CorpusFixture<true>
{
public:
  DBGETTER
  
  TigerFallbackFixture() :  CorpusFixture<true>("tiger2")
  {
    
  }
  

  virtual ~TigerFallbackFixture() {}
};


BASELINE_F(CAT_tiger2, Fallback, TigerFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(Cat, getDB(), 373436u);
}

BENCHMARK_F(CAT_tiger2, Optimized, TigerFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(Cat, getDB(), 373436u);
}

// cat="S" & tok="Bilharziose" & #1 >* #2
BASELINE_F(BIL_tiger2,  Fallback, TigerFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(BilharzioseSentence, getDB(), 21u);
}
BENCHMARK_F(BIL_tiger2, Optimized, TigerFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(BilharzioseSentence, getDB(), 21u);
}

// pos="NN" .2,10 pos="ART" . pos="NN"
BASELINE_F(NAN_tiger2, Fallback, TigerFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreARTPreNN, getDB(), 114042u);
}

BENCHMARK_F(NAN_tiger2, Optimized, TigerFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(NNPreARTPreNN, getDB(), 114042u);
}

// cat=/(.P)/ >* /A.*/
BASELINE_F(REG1_tiger2, Fallback, TigerFallbackFixture , 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(RegexDom, getDB(), 36294u);
}

BENCHMARK_F(REG1_tiger2, Optimized, TigerFixture , 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(RegexDom, getDB(), 36294u);
}

