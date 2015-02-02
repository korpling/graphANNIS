#include "benchmark.h"
#include "examplequeries.h"

char tuebaCorpus[] = "tuebadz6";
char tuebaCorpusSmall[] = "tuebadz6_small";

class TuebaFixture : public CorpusFixture<true, tuebaCorpus>
{
public:
  DBGETTER

  virtual ~TuebaFixture() {}

};
class TuebaFallbackFixture : public CorpusFixture<false, tuebaCorpus>
{
public:
  DBGETTER


  virtual ~TuebaFallbackFixture() {}
};

class TuebaFixtureVar : public CorpusFixture<true, tuebaCorpus>
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
class TuebaFallbackFixtureVar : public CorpusFixture<false, tuebaCorpus>
{
public:


  DB& getDB()
  {
    static DB dbHolder = initDB();
    return dbHolder;
  }

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
  ANNIS_EXEC_QUERY(RegexDom, getDB(), 12u);
}


BENCHMARK_F(Tueba_RegexDom, Optimized, TuebaFixture, 5, 1)
{
  ANNIS_EXEC_QUERY(RegexDom, getDB(), 12u);
}

BASELINE_F(Tueba_Inclusion, Fallback, TuebaFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY(PPERIncludesAnaphoric, getDB(), 13031u);
}


BENCHMARK_F(Tueba_Inclusion, Optimized, TuebaFixture, 5, 1)
{
  ANNIS_EXEC_QUERY(PPERIncludesAnaphoric, getDB(), 13031u);
}

BASELINE_F(Tueba_DomEdgeAnno, Fallback, TuebaFallbackFixture, 5, 1)
{
  ANNIS_EXEC_QUERY(DomFuncON, getDB(), 153u);
}


BENCHMARK_F(Tueba_DomEdgeAnno, Optimized, TuebaFixture, 5, 1)
{
  ANNIS_EXEC_QUERY(DomFuncON, getDB(), 153u);
}

BASELINE_F(Tueba_NodeDom, Fallback, TuebaFallbackFixtureVar, 5, 1)
{
  counter = 0;
  Query q=annis::ExampleQueries::NodeDom(getDB(), maxDistance);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
}


BENCHMARK_F(Tueba_NodeDom, Optimized, TuebaFixtureVar, 5, 1)
{
  counter = 0;
  Query q=annis::ExampleQueries::NodeDom(getDB(), maxDistance);
  while(q.hasNext())
  {
    q.next();
    counter++;
  }
}
