#include "indexjoin.h"


#include <annis/operators/operator.h>


using namespace annis;

IndexJoin::IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     std::shared_ptr<Operator> op,
                     std::function<bool(const Match &)> filterFunc)
  : fetchLoopStarted(false)
{
  auto& resultsReference = results;

  lhsFetchLoop = [lhs, lhsIdx, filterFunc, op, &resultsReference]() -> void {
    std::vector<Match> currentLHSVector;
    while(lhs->next(currentLHSVector))
    {
      const Match& currentLHS = currentLHSVector[lhsIdx];

      std::unique_ptr<AnnoIt> itRHS = op->retrieveMatches(currentLHS);

      if(itRHS)
      {
        // TODO: create multiple threads in background
        Match currentRHS;
        while(itRHS->next(currentRHS))
        {
          if(filterFunc(currentRHS))
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
    // when finished shutdown the queue
    resultsReference.shutdown();
  };
}

bool IndexJoin::next(std::vector<Match> &tuple)
{
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

