#include "query.h"
#include "operators/defaultjoins.h"

using namespace annis;

Query::Query()
{
}


size_t annis::Query::addNode(std::shared_ptr<annis::CacheableAnnoIt> n)
{
  size_t idx = source.size();
  source.push_back(n);
  isOrig.push_back(true);
  return idx;
}

void Query::addOperator(std::shared_ptr<BinaryOperatorIterator> op, size_t idxLeft, size_t idxRight)
{
  if(idxLeft < source.size() && idxRight < source.size())
  {
    op->init(source[idxLeft], source[idxRight]);
    source[idxLeft] = std::make_shared<JoinWrapIterator>(op, true);
    isOrig[idxLeft] = true;

    source[idxRight] = std::make_shared<JoinWrapIterator>(op, false);
    isOrig[idxRight] = false;
  }
}

bool Query::hasNext()
{
  for(const auto& s : source)
  {
    if(!s->hasNext())
    {
      return false;
    }
  }
  return true;
}

std::vector<Match> Query::next()
{
  if(hasNext())
  {
    std::vector<Match> result(source.size());

    // call "next()" on all original sources
    for(size_t i=0; i < source.size(); i++)
    {
      if(isOrig[i])
      {
        source[i]->next();
      }
    }

    for(size_t i=0; i < source.size(); i++)
    {
      result[i] = source[i]->current();
    }
    return result;
  }
  return std::vector<Match>(0);
}

