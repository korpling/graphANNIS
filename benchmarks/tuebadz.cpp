#include "benchmark.h"
#include "examplequeries.h"

char tuebaCorpus[] = "tuebadz6";
char tuebaCorpusSmall[] = "tuebadz6_small";

class TuebaFixture : public CorpusFixture<false, tuebaCorpus>
{
public:
  DBGETTER

  virtual ~TuebaFixture() {}

};
class TuebaFallbackFixture : public CorpusFixture<true, tuebaCorpus>
{
public:
  DBGETTER


  virtual ~TuebaFallbackFixture() {}
};

class TuebaFixtureVar : public CorpusFixture<false, tuebaCorpus>
{
public:
  DBGETTER

  virtual std::vector<int64_t> getExperimentValues() const
  {
    return {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13};
  }

  virtual void setUp(int64_t experimentValue)
  {
    CorpusFixture::setUp(experimentValue);
    maxDistance = experimentValue;
  }

  virtual ~TuebaFixtureVar() {}

  unsigned int maxDistance;
};
class TuebaFallbackFixtureVar : public CorpusFixture<true, tuebaCorpus>
{
public:


  DBGETTER

  virtual std::vector<int64_t> getExperimentValues() const
  {
    return {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13};
  }

  virtual void setUp(int64_t experimentValue)
  {
    CorpusFixture::setUp(experimentValue);
    maxDistance = experimentValue;
  }

  virtual ~TuebaFallbackFixtureVar() {}
  unsigned int maxDistance;
};


BASELINE_F(MIX_tuebadz6, Fallback, TuebaFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(Mixed1, getDB(), 0u);
}


BENCHMARK_F(MIX_tuebadz6, Optimized, TuebaFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(Mixed1, getDB(), 0u);
}

BASELINE_F(REG2_tuebadz6, Fallback, TuebaFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(RegexDom, getDB(), 1u);
}


BENCHMARK_F(REG2_tuebadz6, Optimized, TuebaFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(RegexDom, getDB(), 1u);
}

BASELINE_F(PIA_tuebadz6, Fallback, TuebaFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(PPERIncludesAnaphoric, getDB(), 13031u);
}


BENCHMARK_F(PIA_tuebadz6, Optimized, TuebaFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(PPERIncludesAnaphoric, getDB(), 13031u);
}

BASELINE_F(FUN_tuebadz6, Fallback, TuebaFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(DomFuncON, getDB(), 76748u);
}


BENCHMARK_F(FUN_tuebadz6, Optimized, TuebaFixture, 5, 1)
{
  ANNIS_EXEC_QUERY_COUNT(DomFuncON, getDB(), 76748u);
}

BASELINE_F(DOM_tuebadz6, Fallback, TuebaFallbackFixtureVar, 5, 1)
{
  counter = 0;
  Query q=annis::ExampleQueries::NodeDom(getDB(), maxDistance);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
}


BENCHMARK_F(DOM_tuebadz6, Optimized, TuebaFixtureVar, 5, 1)
{
  counter = 0;
  Query q=annis::ExampleQueries::NodeDom(getDB(), maxDistance);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
}
