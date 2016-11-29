#include "indexjoin.h"


#include <annis/operators/operator.h>


using namespace annis;

IndexJoin::IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     bool (*nextMatchFunc)(const Match &, Match&))
  : fetchLoopStarted(false)
{
  auto& resultsReference = results;

  lhsFetchLoop = [lhs, lhsIdx, nextMatchFunc, &resultsReference]() -> void {
    std::vector<Match> currentLHS;
    while(lhs->next(currentLHS))
    {
      // TODO: create multiple threads in background
      Match currentRHS;
      if(nextMatchFunc(currentLHS[lhsIdx], currentRHS))
      {
        std::vector<Match> tuple;
        tuple.reserve(currentLHS.size()+1);
        tuple.insert(tuple.end(), currentLHS.begin(), currentLHS.end());
        tuple.push_back(currentRHS);

        resultsReference.push(tuple);
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
    std::thread lhsFetcher(lhsFetchLoop);
  }

  //  wait for next item in queue or return immediatly if queue was shutdown
  return results.pop(tuple);
}

void IndexJoin::reset()
{

}

