#include "query.h"

using namespace  annis;

Query::Query(std::vector<std::shared_ptr<SingleAlternativeQuery>> alternatives)
  : alternatives(alternatives), currentAlternativeIdx(0), needsUniqueCheck(false)
{

}

Query::Query(std::shared_ptr<SingleAlternativeQuery> alternative)
  : currentAlternativeIdx(0),  needsUniqueCheck(false)
{
  alternatives.push_back(alternative);
}

Query::~Query()
{

}

bool Query::next()
{
  // TODO: implements
  return false;
}
