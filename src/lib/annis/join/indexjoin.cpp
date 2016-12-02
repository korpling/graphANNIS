#include "indexjoin.h"

#include <future>
#include <list>

#include <annis/operators/operator.h>
#include <annis/util/comparefunctions.h>


using namespace annis;

IndexJoin::IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     std::shared_ptr<Operator> op,
                     std::function<std::list<Match>(nodeid_t)> matchGeneratorFunc)
  : fetchLoopStarted(false), results(8), lhs(lhs), lhsIdx(lhsIdx), op(op), matchGeneratorFunc(matchGeneratorFunc)
{

  auto& resultsReference = results;


  lhsFetchLoop = [lhs, lhsIdx, matchGeneratorFunc, op, &resultsReference]() -> void {
    std::vector<Match> currentLHSVector;
    while(lhs->next(currentLHSVector))
    {
      const Match& currentLHS = currentLHSVector[lhsIdx];

      std::unique_ptr<AnnoIt> itRHS = op->retrieveMatches(currentLHS);

      if(itRHS)
      {
        // TODO: create multiple threads in background
        Match rhsCandidateNode;
        while(itRHS->next(rhsCandidateNode))
        {
          std::list<Match> rhsAnnos = matchGeneratorFunc(rhsCandidateNode.node);
          for(Match currentRHS : rhsAnnos)
          {
            // additionally check for reflexivity
            if((op->isReflexive() || currentLHS.node != currentRHS.node
                   || !checkAnnotationEqual(currentLHS.anno, currentRHS.anno)))
            {
              std::vector<Match> tuple;
              tuple.reserve(currentLHSVector.size()+1);
              tuple.insert(tuple.end(), currentLHSVector.begin(), currentLHSVector.end());
              tuple.push_back(currentRHS);

              resultsReference.push(tuple);
            }
          }
        }
      }
    }
    // when finished shutdown the queue
    resultsReference.shutdown();
  };
}

bool IndexJoin::next(std::vector<Match> &tuple)
{

  // check if the chain of RHS matches for a single LHS is not empty
  std::shared_ptr<ResultListEntry> result = resultFuture.get();
  if(result)
  {
    const MatchPair& m = result->val;
    tuple.reserve(m.lhs.size()+1);
    tuple.insert(tuple.end(), m.lhs.begin(), m.lhs.end());
    tuple.push_back(m.rhs);

    // set the head of the list to the next item
    resultFuture = result->next;

    return true;
  }
  else
  {
    std::vector<Match> currentLHSVector;
    if(lhs->next(currentLHSVector))
    {
      // TODO: fill the linked list with new RHS matches
      const Match& currentLHS = currentLHSVector[lhsIdx];

      std::function<std::shared_ptr<ResultListEntry>(const std::shared_ptr<ResultListEntry>& )> recursiveFunc;

      recursiveFunc = [&recursiveFunc] (const std::shared_ptr<ResultListEntry>& old) -> std::shared_ptr<ResultListEntry>
      {
        std::shared_ptr<ResultListEntry> result = std::make_shared<ResultListEntry>();

        result->next = std::async(recursiveFunc, result).share();

        return result;
      };


      std::unique_ptr<AnnoIt> itRHS = op->retrieveMatches(currentLHS);

      if(itRHS)
      {
        Match rhsCandidateNode;
        while(itRHS->next(rhsCandidateNode))
        {

        }
      }
    }
    else
    {
      // nothing more to do
      return false;
    }
  }
  // OLD STUFF
  if(!fetchLoopStarted)
  {
    fetchLoopStarted = true;
    lhsFetcher = std::thread(lhsFetchLoop);
  }

  //  wait for next item in queue or return immediatly if queue was shutdown
  return results.pop(tuple);
}

void IndexJoin::reset()
{
  if(lhsFetcher.joinable())
  {
    lhsFetcher.join();
  }
  fetchLoopStarted = false;
}

IndexJoin::~IndexJoin()
{
  if(lhsFetcher.joinable())
  {
    lhsFetcher.join();
  }
}

