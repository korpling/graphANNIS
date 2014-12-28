#include "query.h"
#include "operators/wrapper.h"

using namespace annis;

Query::Query()
  : initialized(false)
{
}


size_t annis::Query::addNode(std::shared_ptr<annis::CacheableAnnoIt> n)
{
  initialized = false;

  size_t idx = nodes.size();
  nodes.push_back(n);
  return idx;
}

void Query::addOperator(std::shared_ptr<BinaryIt> op, size_t idxLeft, size_t idxRight)
{
  initialized = false;

  OperatorEntry entry;
  entry.op = op;
  entry.idxLeft = idxLeft;
  entry.idxRight = idxRight;

  operators.push_back(entry);
}

void Query::internalInit()
{
  // clear old internal variables
  source.clear();
  isOrig.clear();

  // 1. add all nodes
  for(auto& n : nodes)
  {
    source.push_back(n);
    isOrig.push_back(true);
  }

  // 2. add the operators which produce the results
  for(auto& e : operators)
  {
    if(e.idxLeft < source.size() && e.idxRight < source.size())
    {
      e.op->init(source[e.idxLeft], source[e.idxRight]);
      source[e.idxLeft] = std::make_shared<JoinWrapIterator>(e.op, true);
      isOrig[e.idxLeft] = true;

      source[e.idxRight] = std::make_shared<JoinWrapIterator>(e.op, false);
      isOrig[e.idxRight] = false;
    }
  }

  // TODO: add filters

  initialized = true;
}

bool Query::hasNext()
{
  if(!initialized)
  {
    internalInit();
  }

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
  if(!initialized)
  {
    internalInit();
  }

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

