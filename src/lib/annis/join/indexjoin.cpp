#include "indexjoin.h"


#include <future>
#include <list>


#include <annis/operators/operator.h>
#include <annis/util/comparefunctions.h>


using namespace annis;

IndexJoin::IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     std::shared_ptr<Operator> op,
                     std::function<std::list<Match>(nodeid_t)> matchGeneratorFunc, unsigned maxBufferedTasks,
                     std::shared_ptr<ThreadPool> threadPool)
  : lhs(lhs), lhsIdx(lhsIdx), maxNumfOfTasks(maxBufferedTasks > 0 ? maxBufferedTasks : 1), threadPool(threadPool)
{


  taskBufferGenerator = [matchGeneratorFunc, op, lhsIdx](std::vector<Match> currentLHS) -> std::deque<MatchPair>
  {
    std::deque<MatchPair> result;

    std::unique_ptr<AnnoIt> reachableNodesIt = op->retrieveMatches(currentLHS[lhsIdx]);
    if(reachableNodesIt)
    {
      Match reachableNode;
      while(reachableNodesIt->next(reachableNode))
      {
        for(Match currentRHS : matchGeneratorFunc(reachableNode.node))
        {
          if((op->isReflexive() || currentLHS[lhsIdx].node != currentRHS.node
          || !checkAnnotationEqual(currentLHS[lhsIdx].anno, currentRHS.anno)))
          {
            result.push_back({currentLHS, currentRHS});
          }
        }
      }
    }

    return result;
  };
}

bool IndexJoin::next(std::vector<Match> &tuple)
{
  tuple.clear();


  do
  {
    while(!matchBuffer.empty())
    {
      const MatchPair& m = matchBuffer.front();

      tuple.reserve(m.lhs.size()+1);
      tuple.insert(tuple.begin(), m.lhs.begin(), m.lhs.end());
      tuple.push_back(m.rhs);

      matchBuffer.pop_front();
      return true;

    }
  } while (nextMatchBuffer());

  return false;
}

void IndexJoin::reset()
{
  if(lhs)
  {
    lhs->reset();
  }

  matchBuffer.clear();
  taskBuffer.clear();
}


void IndexJoin::fillTaskBuffer()
{
  std::vector<Match> currentLHS;
  while(taskBuffer.size() < maxNumfOfTasks && lhs->next(currentLHS))
  {
    if(threadPool)
    {
      taskBuffer.push_back(threadPool->enqueue(taskBufferGenerator, currentLHS));
    }
    else
    {
      // do not use threads
      taskBuffer.push_back(std::async(std::launch::deferred, taskBufferGenerator, currentLHS));
    }
  }
}

bool IndexJoin::nextMatchBuffer()
{
  // make sure the task buffer is filled
  fillTaskBuffer();

  while(!taskBuffer.empty())
  {
    taskBuffer.front().wait();
    matchBuffer = taskBuffer.front().get();
    taskBuffer.pop_front();

    // re-fill the task buffer with a new task
    fillTaskBuffer();

    // if there is a non empty result return true, otherwise try more entries of the task buffer
    if(!matchBuffer.empty())
    {
      return true;
    }
  }

  // task buffer is empty and we can't fill it any more
  return false;
}

IndexJoin::~IndexJoin()
{
}
