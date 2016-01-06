#include "query.h"
#include "join/nestedloop.h"
#include "join/seed.h"
#include "filter.h"

#include <vector>

using namespace annis;

Query::Query(const DB &db)
  : db(db), initialized(false)
{
}

Query::~Query() {
  
}

size_t annis::Query::addNode(std::shared_ptr<AnnotationSearch> n)
{
  initialized = false;

  size_t idx = nodes.size();
  nodes.push_back(n);
  return idx;
}

size_t annis::Query::addNode(std::shared_ptr<AnnotationKeySearch> n)
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
  entry.op = op;
  entry.useNestedLoop = useNestedLoop;
  entry.idxLeft = idxLeft;
  entry.idxRight = idxRight;

  operators.push_back(entry);
}

void Query::internalInit()
{
  if(initialized) {
    return;
  }

  // 1. add all nodes
  int i=0;
  for(auto& n : nodes)
  {
    source.push_back(n);
    querynode2component[i]=i;
    i++;
  }

  // 2. add the operators which produce the results
  for(auto& e : operators)
  {
    if(e.idxLeft < source.size() && e.idxRight < source.size())
    {
      int leftComponent = querynode2component[e.idxLeft];
      int rightComponent = querynode2component[e.idxRight];

      if(leftComponent == rightComponent)
      {
        addJoin(e, true);
      }
      else
      {
        addJoin(e, false);
        mergeComponents(leftComponent, rightComponent);
      }
    }
  }

  // 3. check if every node is connected
  int firstComponent;
  bool firstComponentSet = false;
  for(const auto& e : querynode2component)
  {
    if(firstComponentSet)
    {
      if(e.second != firstComponent)
      {
        std::cerr << "Node " << e.first << " is not connected" << std::endl;
        return;
      }
    }
    else
    {
      firstComponent = e.second;
      firstComponentSet = true;
    }
  }

  initialized = true;
}

void Query::addJoin(OperatorEntry& e, bool filterOnly)
{
  std::shared_ptr<BinaryIt> j;
  if(filterOnly)
  {
    j = std::make_shared<Filter>(e.op, source[e.idxLeft], source[e.idxRight]);
  }
  else
  {
    if(e.useNestedLoop)
    {
      j = std::make_shared<NestedLoopJoin>(e.op, source[e.idxLeft], source[e.idxRight]);
    }
    else
    {
      std::shared_ptr<AnnoIt> rightIt = nodes[e.idxRight];
      std::shared_ptr<AnnotationKeySearch> keySearch =
          std::dynamic_pointer_cast<AnnotationKeySearch>(rightIt);
      std::shared_ptr<AnnotationSearch> annoSearch =
          std::dynamic_pointer_cast<AnnotationSearch>(rightIt);

      if(keySearch)
      {
        j = std::make_shared<AnnoKeySeedJoin>(db, e.op, source[e.idxLeft],
            keySearch->getValidAnnotationKeys());
      }
      else if(annoSearch)
      {
        j = std::make_shared<MaterializedSeedJoin>(db, e.op, source[e.idxLeft],
            annoSearch->getValidAnnotations());
      }
      else
      {
        // fallback to nested loop
        j = std::make_shared<NestedLoopJoin>(e.op, source[e.idxLeft], source[e.idxRight]);
      }
    }
  }

  std::shared_ptr<JoinWrapIterator> itLeft =
      std::make_shared<JoinWrapIterator>(j, true);
  std::shared_ptr<JoinWrapIterator> itRight =
      std::make_shared<JoinWrapIterator>(j, false);

  itLeft->setOther(itRight);
  itRight->setOther(itLeft);

  source[e.idxLeft] = itLeft;
  source[e.idxRight] = itRight;
}

void Query::mergeComponents(int c1, int c2)
{
  if(c1 == c2)
  {
    // nothing todo
    return;
  }

  std::vector<int> nodeIDsForC2;
  for(const auto e : querynode2component)
  {
    if(e.second == c2)
    {
      nodeIDsForC2.push_back(e.first);
    }
  }
  // set the component id for each node of the other component
  for(auto nodeID : nodeIDsForC2)
  {
    querynode2component[nodeID] = c1;
  }
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

  std::vector<Match> result(source.size());

  // call "next()" on all sources
  for(size_t i=0; i < source.size(); i++)
  {
    result[i] = source[i]->next();
  }

  return result;

  return std::vector<Match>(0);
}

