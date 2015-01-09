#include "query.h"
#include "operators/defaultjoins.h"

using namespace annis;

Query::Query(const DB &db)
  : db(db), initialized(false)
{
}


size_t annis::Query::addNode(std::shared_ptr<AnnoIt> n)
{
  initialized = false;

  size_t idx = nodes.size();
  nodes.push_back(n);
  return idx;
}

void Query::addOperator(std::shared_ptr<Operator> op, size_t idxLeft, size_t idxRight, bool useNestedLoop)
{
  initialized = false;

  OperatorEntry entry;
  if(useNestedLoop)
  {
    entry.op = std::make_shared<NestedLoopJoin>(op);
  }
  else
  {
    entry.op = std::make_shared<SeedJoin>(db, op);
  }
  entry.idxLeft = idxLeft;
  entry.idxRight = idxRight;

  operators.push_back(entry);
}

void Query::internalInit()
{
  // clear old internal variables
  source.clear();

  // 1. add all nodes
  for(auto& n : nodes)
  {
    source.push_back(n);
  }

  // 2. add the operators which produce the results
  for(auto& e : operators)
  {
    if(e.idxLeft < source.size() && e.idxRight < source.size())
    {
      e.op->init(source[e.idxLeft], source[e.idxRight]);

      std::shared_ptr<JoinWrapIterator> itLeft =
          std::make_shared<JoinWrapIterator>(e.op, source[e.idxLeft]->getAnnotation(), true);
      std::shared_ptr<JoinWrapIterator> itRight =
          std::make_shared<JoinWrapIterator>(e.op, source[e.idxRight]->getAnnotation(), false);

      itLeft->setOther(itRight);
      itRight->setOther(itLeft);

      source[e.idxLeft] = itLeft;
      source[e.idxRight] = itRight;
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

    // call "next()" on all sources
    for(size_t i=0; i < source.size(); i++)
    {
      result[i] = source[i]->next();
    }

    return result;
  }
  return std::vector<Match>(0);
}

