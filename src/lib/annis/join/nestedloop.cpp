#include <annis/join/nestedloop.h>
#include <annis/annosearch/annotationsearch.h>
#include <annis/iterators.h>
#include <annis/operators/operator.h>

using namespace annis;


NestedLoopJoin::NestedLoopJoin(std::shared_ptr<Operator> op,
                               std::shared_ptr<Iterator> lhs,
                               std::shared_ptr<Iterator> rhs,
                               size_t lhsIdx, size_t rhsIdx,
                               bool leftIsOuter,
                               unsigned maxBufferedTasks,
                               std::shared_ptr<ThreadPool> threadPool)
  : op(op), leftIsOuter(leftIsOuter), initialized(false),
    outer(leftIsOuter ? lhs : rhs), inner(leftIsOuter ? rhs : lhs),
    outerIdx(leftIsOuter ? lhsIdx : rhsIdx), innerIdx(leftIsOuter ? rhsIdx : lhsIdx),
    firstOuterFinished(false),
    maxBufferedTasks(maxBufferedTasks > 0 ? maxBufferedTasks : 1),
    threadPool(threadPool)
{

}

bool NestedLoopJoin::next(std::vector<Match>& result)
{
  result.clear();
  
  if(!op || !outer || !inner || (firstOuterFinished && innerCache.empty()))
  {
    return false;
  }

  fillTaskBuffer();

  while(!taskBuffer.empty())
  {
    taskBuffer.front().wait();
    const MatchPair current = taskBuffer.front().get();
    taskBuffer.pop_front();

    if(current.found)
    {
      result.reserve(matchInner.size() + matchOuter.size());
      // return a tuple where the first values are from the outer relation and the iner relations tuples are added behind

      result.insert(result.end(), matchOuter.begin(), matchOuter.end());
      result.insert(result.end(), matchInner.begin(), matchInner.end());
    }

    // re-fill the task buffer with a new task
    fillTaskBuffer();

    if(current.found)
    {
      return true;
    }
  }

  return false;
}

bool NestedLoopJoin::fetchNextInner()
{ 
  if(firstOuterFinished)
  {
    if(itInnerCache != innerCache.end())
    {
      matchInner = *itInnerCache;
      itInnerCache++;
      return true;
    }
    else
    {
      return false;
    }
  }
  else
  {
    bool hasNext = inner->next(matchInner);
    if(hasNext)
    {
      innerCache.push_back(matchInner);
    }
    return hasNext;
  }
}

void NestedLoopJoin::fillTaskBuffer()
{
  bool proceed = true;

  if(!initialized)
  {
    proceed = false;
    if(outer->next(matchOuter))
    {
      proceed = true;
      initialized = true;
    }
  }

  std::shared_ptr<Operator>& opRef = op;
  const bool _leftIsOuter = leftIsOuter;
  auto filterFunc = [opRef, _leftIsOuter](const Match& outer, const Match& inner) -> MatchPair {
    MatchPair result;
    result.found = false;

    bool include = true;
    // do not include the same match if not reflexive
    if(!opRef->isReflexive()
       && outer.node == inner.node
       && checkAnnotationKeyEqual(outer.anno, inner.anno)) {
      include = false;
    }

    if(include)
    {
      if(_leftIsOuter)
      {
        if(opRef->filter(outer, inner))
        {
          result.found = true;
          result.lhs = outer;
          result.rhs = inner;
        }
      }
      else
      {
        if(opRef->filter(inner, outer))
        {
          result.found = true;
          result.lhs = inner;
          result.rhs = outer;
        }
      }
    } // end if include

    return std::move(result);
  };

  while(proceed && taskBuffer.size() < maxBufferedTasks)
  {
    while(fetchNextInner())
    {
      if(threadPool)
      {
        taskBuffer.push_back(threadPool->enqueue(filterFunc, matchOuter[outerIdx], matchInner[innerIdx]));
      }
      else
      {
        taskBuffer.push_back(std::async(std::launch::deferred, filterFunc, matchOuter[outerIdx], matchInner[innerIdx]));
      }

    } // end for each right

    if(outer->next(matchOuter))
    {
      firstOuterFinished = true;
      itInnerCache = innerCache.begin();
      inner->reset();

      if(innerCache.empty())
      {
        // inner is always empty, no point in trying to get more from the outer side
        proceed = false;
      }

    }
    else
    {
      proceed = false;
    }
  } // end while proceed
}


void NestedLoopJoin::reset()
{
  outer->reset();
  inner->reset();
  taskBuffer.clear();
  initialized = false;
  if(firstOuterFinished)
  {
    itInnerCache = innerCache.begin();
  }
}

NestedLoopJoin::~NestedLoopJoin()
{

}
