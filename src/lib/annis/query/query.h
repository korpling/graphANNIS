#pragma once

#include <annis/query/singlealternativequery.h>

#include <vector>
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
  std::vector<std::shared_ptr<SingleAlternativeQuery>> alternatives;
  const bool proxyMode;
  size_t currentAlternativeIdx;

  std::vector<Match> currentResult;

  btree::btree_set<std::vector<Match>> uniqueResultSet;
};


}
