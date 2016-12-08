#include "indexjoin.h"

#include <future>
#include <list>

#include <annis/operators/operator.h>
#include <annis/util/comparefunctions.h>


using namespace annis;

IndexJoin::IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     std::shared_ptr<Operator> op,
                     std::function<std::list<Match>(nodeid_t)> matchGeneratorFunc)
  : lhs(lhs), lhsIdx(lhsIdx)
{


  taskBufferGenerator = [matchGeneratorFunc, op, lhsIdx](std::vector<Match> currentLHS) -> std::list<MatchPair>
  {
    std::list<MatchPair> result;

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
  } while (fillTaskBuffer());

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


bool IndexJoin::fillTaskBuffer()
{
  std::vector<Match> currentLHS;
  while(taskBuffer.size() < 4 && lhs->next(currentLHS))
  {
    taskBuffer.push_back(std::async(taskBufferGenerator, currentLHS));
  }

  return !taskBuffer.empty();

}

bool IndexJoin::nextMatchBuffer()
{
  while(!taskBuffer.empty())
  {
    std::future<std::list<MatchPair>>& firstFuture = taskBuffer.front();
    matchBuffer = firstFuture.get();
    taskBuffer.pop_front();
    if(!matchBuffer.empty())
    {
      return true;
    }
  }

  return false;
}

IndexJoin::~IndexJoin()
{
}
