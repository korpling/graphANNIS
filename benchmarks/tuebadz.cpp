#include "benchmark.h"
#include "examplequeries.h"


class TuebaFixtureVar : public CorpusFixture<false>
{
public:
  DBGETTER
  
  TuebaFixtureVar() : CorpusFixture<false>("tuebadz6")
  {
    
  }

  virtual std::vector<std::pair<int64_t, uint64_t>> getExperimentValues() const
  {
    std::vector<std::pair<int64_t, uint64_t>> result;
    for(int i=1; i <= 13; i++)
    {
      result.push_back({i,0});
    }
    return result;
  }

  virtual void setUp(int64_t experimentValue)
  {
    CorpusFixture::setUp(experimentValue);
    maxDistance = experimentValue;
  }

  virtual ~TuebaFixtureVar() {}

  unsigned int maxDistance;
};
class TuebaFallbackFixtureVar : public CorpusFixture<true>
{
public:


  DBGETTER

  TuebaFallbackFixtureVar() : CorpusFixture<true>("tuebadz6")
  {
    
  }
  
  virtual std::vector<std::pair<int64_t, uint64_t>> getExperimentValues() const
  {
    std::vector<std::pair<int64_t, uint64_t>> result;
    for(int i=1; i <= 13; i++)
    {
      result.push_back({i,0});
    }
    return result;;
  }

  virtual void setUp(int64_t experimentValue)
  {
    CorpusFixture::setUp(experimentValue);
    maxDistance = experimentValue;
  }

  virtual ~TuebaFallbackFixtureVar() {}
  unsigned int maxDistance;
};

//BASELINE_F(DOM_tuebadz6, Fallback, TuebaFallbackFixtureVar, 5, 1)
//{
//  counter = 0;
//  Query q=annis::ExampleQueries::NodeDom(getDB(), maxDistance);
//  while(q.hasNext())
//  {
//    q.next();
//    counter++;
//  }
//}
//
//
//BENCHMARK_F(DOM_tuebadz6, Optimized, TuebaFixtureVar, 5, 1)
//{
//  counter = 0;
//  Query q=annis::ExampleQueries::NodeDom(getDB(), maxDistance);
//  while(q.hasNext())
//  {
//    q.next();
//    counter++;
//  }
//}
