#include "indexjoin.h"

#include <future>
#include <list>

#include <annis/operators/operator.h>
#include <annis/util/comparefunctions.h>


using namespace annis;

IndexJoin::IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     std::shared_ptr<Operator> op,
                     std::function<std::list<Match>(nodeid_t)> matchGeneratorFunc)
  : lhs(lhs), lhsIdx(lhsIdx), op(op)
{

  bool isReflexive = op->isReflexive();
  rhsBufferGenerator = [matchGeneratorFunc, isReflexive](const Match& currentLHS, nodeid_t rhsNode) -> MatchCandidate
  {
    MatchCandidate candidate;
    candidate.valid = false;
    std::list<Match> rhsAnnos = matchGeneratorFunc(rhsNode);
    for(Match currentRHS : rhsAnnos)
    {
      // additionally check for reflexivity
      if((isReflexive || currentLHS.node != currentRHS.node
      || !checkAnnotationEqual(currentLHS.anno, currentRHS.anno)))
      {
        candidate.valid = true;
        candidate.rhs.push_back(currentRHS);
      }
    }
    return candidate;
  };
}

bool IndexJoin::next(std::vector<Match> &tuple)
{
  tuple.clear();

  do
  {
    do
    {
      while(!rhsAnnoBuffer.empty())
      {
        const Match& rhs = rhsAnnoBuffer.front();

        tuple.reserve(currentLHS.size()+1);
        tuple.insert(tuple.begin(), currentLHS.begin(), currentLHS.end());
        tuple.push_back(rhs);

        rhsAnnoBuffer.pop();
        return true;

      }
    } while (nextRHSBuffer());
  } while (nextCurrentLHS());

  return false;
}

void IndexJoin::reset()
{
  if(lhs)
  {
    lhs->reset();
  }
}

IndexJoin::~IndexJoin()
{
}

bool IndexJoin::nextCurrentLHS()
{
  bool currentLHSValid = lhs->next(currentLHS);
  if(currentLHSValid)
  {
    // fill up the RHS buffer with all reachable nodes from the next LHS
    std::unique_ptr<AnnoIt> reachableNodesIt = op->retrieveMatches(currentLHS[lhsIdx]);
    if(reachableNodesIt)
    {
      Match currentRHSNode;
      while(reachableNodesIt->next(currentRHSNode))
      {
        rhsBuffer.push(std::async(rhsBufferGenerator, currentLHS[lhsIdx], currentRHSNode.node));
      }
    }
  }
  return currentLHSValid;

}

bool IndexJoin::nextRHSBuffer()
{
  while(!rhsBuffer.empty())
  {
    // add all matching annotations of the first valid entry to the buffer
    std::future<MatchCandidate>& firstFuture = rhsBuffer.front();
    firstFuture.wait();
    MatchCandidate firstCandidate = firstFuture.get();
    rhsBuffer.pop();

    if(firstCandidate.valid)
    {
      for(const Match& rhsAnno : firstCandidate.rhs)
      {
        rhsAnnoBuffer.push(rhsAnno);
      }
    }

    return true;
  }
  return false;
}

