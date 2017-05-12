#include "query.h"

#include <sstream>

#include <annis/util/plan.h>

using namespace  annis;

Query::Query(std::vector<std::shared_ptr<SingleAlternativeQuery>> alternatives)
  : alternatives(alternatives), proxyMode(alternatives.size() == 1), currentAlternativeIdx(0)
{

}

Query::Query(std::shared_ptr<SingleAlternativeQuery> alternative)
  : proxyMode(true), currentAlternativeIdx(0)
{
  alternatives.push_back(alternative);
}

Query::~Query()
{

}

bool Query::next()
{
  if(proxyMode)
  {
    // just act as an proxy
    return alternatives[0]->next();
  }
  else
  {
    for(;currentAlternativeIdx < alternatives.size(); currentAlternativeIdx++)
    {
      while(alternatives[currentAlternativeIdx] && alternatives[currentAlternativeIdx]->next())
      {
        currentResult = alternatives[currentAlternativeIdx]->getCurrent();

        if(uniqueResultSet.find(currentResult) == uniqueResultSet.end())
        {
          uniqueResultSet.insert(currentResult);
          return true;
        }

      }
    }
  }
  return false;
}

std::string Query::debugString()
{
  if(proxyMode)
  {
    return alternatives[0]->getBestPlan()->debugString();
  }
  else
  {
    std::stringstream ss;
    for(size_t i=0; i < alternatives.size(); i++)
    {
      if(alternatives[i])
      {
        ss << alternatives[i]->getBestPlan()->debugString();
        if(i+1 < alternatives.size())
        {
          ss << "---[OR]---" << std::endl;
        }
      }
    }
    return ss.str();
  }
}
