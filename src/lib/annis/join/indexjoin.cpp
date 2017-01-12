/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#include "indexjoin.h"
#include <annis/annosearch/annotationsearch.h>

using namespace annis;

IndexJoin::IndexJoin(const DB &db, std::shared_ptr<Operator> op,
                     std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     std::function<std::list<Annotation>(nodeid_t)> matchGeneratorFunc,
                     bool maximalOneRHSAnno)
  : db(db), op(op),
    left(lhs), lhsIdx(lhsIdx), matchGeneratorFunc(matchGeneratorFunc),
    currentLHSMatchValid(false),
    operatorIsReflexive(op->isReflexive()),
    maximalOneRHSAnno(maximalOneRHSAnno)
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
  
  if(!maximalOneRHSAnno && nextRightAnnotation())
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
      if(maximalOneRHSAnno)
      {
        std::list<Annotation> annos = matchGeneratorFunc(currentRHSMatch.node);

        if(!annos.empty())
        {
          currentRHSMatch.anno = annos.front();
          if(operatorIsReflexive || currentLHSMatch[lhsIdx].node != currentRHSMatch.node
             || !checkAnnotationKeyEqual(currentLHSMatch[lhsIdx].anno, currentRHSMatch.anno))
          {
            tuple.reserve(currentLHSMatch.size()+1);
            tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
            tuple.push_back(currentRHSMatch);

            return true;
          }
        }
      }
      else
      {
        rhsCandidates = matchGeneratorFunc(currentRHSMatch.node);

        if(nextRightAnnotation())
        {
          tuple.reserve(currentLHSMatch.size()+1);
          tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
          tuple.push_back(currentRHSMatch);
          return true;
        }
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

