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
  const std::vector<Match>& getCurrent() { return currentResult;}

  std::string debugString()
  {
    // TODO: implmenent
    return "TODO";
  };

private:
  std::vector<std::shared_ptr<SingleAlternativeQuery>> alternatives;
  size_t currentAlternativeIdx;

  const bool needsUniqueCheck;

  std::vector<Match> currentResult;

  btree::btree_set<nodeid_t> uniqueResultSet;
};


}
