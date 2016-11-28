#include "indexjoin.h"


#include <annis/operators/operator.h>


using namespace annis;

IndexJoin::IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     Match (*nextMatchFunc)(const Match &))
  : lhs(lhs), lhsIdx(lhsIdx), nextMatchFunc(nextMatchFunc), execState(State::INIT)
{

}

bool IndexJoin::next(std::vector<Match> &tuple)
{
  switch(execState)
  {
    case State::INIT:
      {

        std::thread lhsFetcher(lhsFetchLoop);

        std::vector<Match> lhsMatch;
        // get the first LHS
        bool found = lhs->next(lhsMatch);
        if(found)
        {
        }
        else
        {
          execState = State::FINISHED;
          return false;
        }

        execState = State::STARTED;
      }
      break;
    case State::STARTED:

      break;
    case State::FINISHED:
      // this join has finished and does not produce any more results
      return false;
      break;
  }

  return false;
}

void IndexJoin::reset()
{

}

void IndexJoin::lhsFetchLoop()
{
  std::vector<Match> currentLHS;
  while(lhs->next(currentLHS))
  {

  }
}
