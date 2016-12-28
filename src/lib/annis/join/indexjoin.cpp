#include "indexjoin.h"
#include <annis/annosearch/annotationsearch.h>

using namespace annis;

IndexJoin::IndexJoin(const DB &db, std::shared_ptr<Operator> op,
                    std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                    std::function<std::list<Annotation>(nodeid_t)> matchGeneratorFunc)
  : db(db), op(op),
    left(lhs), lhsIdx(lhsIdx), matchGeneratorFunc(matchGeneratorFunc),
    currentLHSMatchValid(false),
    operatorIsReflexive(op->isReflexive())
{

}

IndexJoin::~IndexJoin()
{

}

bool IndexJoin::next(std::vector<Match>& tuple)
{
  tuple.clear();

  if(!currentLHSMatchValid)
  {
    nextLeftMatch();
  }
  
  if(!op || !left || !currentLHSMatchValid)
  {
    return false;
  }
  
  if(nextRightAnnotation())
  {
    tuple.reserve(currentLHSMatch.size()+1);
    tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
    tuple.push_back(currentRHSMatch);
    return true;
  }

  do
  {
    while(matchesByOperator && matchesByOperator->next(currentRHSMatch))
    {
      rhsCandidates = matchGeneratorFunc(currentRHSMatch.node);

      if(nextRightAnnotation())
      {
        tuple.reserve(currentLHSMatch.size()+1);
        tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
        tuple.push_back(currentRHSMatch);
        return true;
      }
    } // end while there are right candidates
  } while(nextLeftMatch()); // end while left has match


  return false;
}

void IndexJoin::reset()
{
  if(left)
  {
    left->reset();
  }

  matchesByOperator.reset(nullptr);
  rhsCandidates.clear();
  currentLHSMatchValid = false;
}

bool IndexJoin::nextLeftMatch()
{
  rhsCandidates.clear();
  if(op && op->valid() && left && left->next(currentLHSMatch))
  {
    currentLHSMatchValid = true;


    matchesByOperator = op->retrieveMatches(currentLHSMatch[lhsIdx]);
    if(matchesByOperator)
    {
      return true;
    }
  }

  return false;
}

bool IndexJoin::nextRightAnnotation()
{
  while(!rhsCandidates.empty())
  {
    if(operatorIsReflexive || currentLHSMatch[lhsIdx].node != currentRHSMatch.node
       || !checkAnnotationKeyEqual(currentLHSMatch[lhsIdx].anno, rhsCandidates.front()))
    {
      currentRHSMatch.anno = std::move(rhsCandidates.front());
      rhsCandidates.pop_front();
      return true;
    }
    else
    {
      rhsCandidates.pop_front();
    }
  }
  return false;
}

