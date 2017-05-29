#pragma once

#include <annis/query/singlealternativequery.h>

#include <vector>
#include <unordered_set>
#include <boost/functional/hash.hpp>
#include <google/btree_set.h>

namespace annis {


class Query
{
public:
  Query(std::vector<std::shared_ptr<SingleAlternativeQuery>> alternatives);
  Query(std::shared_ptr<SingleAlternativeQuery> alternative);
  virtual ~Query();

  bool next();
  const std::vector<Match>& getCurrent()
  {
    if(proxyMode)
    {
      // just act as an proxy
      return alternatives[0]->getCurrent();
    }
    else
    {
      return currentResult;
    }
  }

  std::string debugString();
private:

  struct MatchVectorHash
  {
    std::size_t operator()(std::vector<Match> const& v) const
    {
      std::size_t seed = 0;
      for(size_t i=0; i < v.size(); i++)
      {
        const Match& m = v[i];
        boost::hash_combine(seed, m.node);
        boost::hash_combine(seed, m.anno.ns);
        boost::hash_combine(seed, m.anno.name);
        boost::hash_combine(seed, m.anno.val);
      }

      return seed;
    }
  };


private:
  std::vector<std::shared_ptr<SingleAlternativeQuery>> alternatives;
  const bool proxyMode;
  size_t currentAlternativeIdx;

  std::vector<Match> currentResult;

  std::unordered_set<std::vector<Match>, MatchVectorHash> uniqueResultSet;
};


}
