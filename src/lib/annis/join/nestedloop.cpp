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

  filterFunc = [op, leftIsOuter] (
      const std::vector<Match> outerVec,
      const std::vector<Match> innerVec,
      const size_t outerIdx, const size_t innerIdx) -> MatchPair {
    MatchPair result;
    result.found = false;

    const Match& outer = outerVec[outerIdx];
    const Match& inner = innerVec[innerIdx];


    bool include = true;
    // do not include the same match if not reflexive
    if(!op->isReflexive()
       && outer.node == inner.node
       && checkAnnotationKeyEqual(outer.anno, inner.anno)) {
      include = false;
    }

    if(include)
    {
      if(leftIsOuter)
      {
        if(op->filter(outer, inner))
        {
          result.found = true;
          result.lhs = outerVec;
          result.rhs = innerVec;
        }
      }
      else
      {
        if(op->filter(inner, outer))
        {
          result.found = true;
          result.lhs = innerVec;
          result.rhs = outerVec;
        }
      }
    } // end if include

    return std::move(result);
  };
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
      result.reserve(current.lhs.size() + current.rhs.size());
      // return a tuple where the first values are from the outer relation and the iner relations tuples are added behind

      result.insert(result.end(), current.lhs.begin(), current.lhs.end());
      result.insert(result.end(), current.rhs.begin(), current.rhs.end());
    }

    // re-fill the task buffer with a new task
    fillTaskBuffer();

    if(current.found)
    {
      assert(result.size() > 0);
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

  while(proceed && taskBuffer.size() < maxBufferedTasks)
  {
    while(fetchNextInner())
    {
      if(threadPool)
      {
        taskBuffer.push_back(threadPool->enqueue(filterFunc, matchOuter, matchInner, outerIdx, innerIdx));
      }
      else
      {
        taskBuffer.push_back(std::async(std::launch::deferred, filterFunc,
                                         matchOuter, matchInner, outerIdx, innerIdx));
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
