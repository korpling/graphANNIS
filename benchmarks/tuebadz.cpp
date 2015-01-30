#include "benchmark.h"
#include "examplequeries.h"

char tuebaCorpus[] = "tuebadz6";

class TuebaFixture : public CorpusFixture<true, tuebaCorpus>
{
public:
  DBGETTER

  virtual ~TuebaFixture() {}
};
class TuebaFallbackFixture : public CorpusFixture<false, tuebaCorpus>
{
public:


  DB& getDB()
  {
    static DB dbHolder = initDB();
    return dbHolder;
  }

  virtual ~TuebaFallbackFixture() {}
};


BASELINE_F(Tueba_Mixed1, Fallback, TuebaFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY(Mixed1, getDB(), 0u);
}


BENCHMARK_F(Tueba_Mixed1, Optimized, TuebaFixture, 5, 1)
{
  ANNIS_EXEC_QUERY(Mixed1, getDB(), 0u);
}

BASELINE_F(Tueba_RegexDom, Fallback, TuebaFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY(RegexDom, getDB(), 48181u);
}


BENCHMARK_F(Tueba_RegexDom, Optimized, TuebaFixture, 5, 1)
{
  ANNIS_EXEC_QUERY(RegexDom, getDB(), 48181u);
}
