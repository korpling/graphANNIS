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

#include "simdindexjoin.h"

#include <annis/operators/operator.h>     // for Operator
#include <annis/util/comparefunctions.h>  // for checkAnnotationEqual
#include <algorithm>                      // for move
#include <future>                         // for future, async, launch, laun...
#include <list>                           // for list
#include "annis/iterators.h"              // for AnnoIt, Iterator
#include "annis/types.h"                  // for Match, Annotation, nodeid_t
#include "annis/util/threadpool.h"        // for ThreadPool

#include <Vc/Vc>


using namespace annis;

SIMDIndexJoin::SIMDIndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                             std::shared_ptr<Operator> op,
                             const AnnoStorage<nodeid_t>& annos,
                             Annotation rhsAnnoToFind)
  : lhs(lhs), lhsIdx(lhsIdx), op(op), annos(annos), rhsAnnoToFind(rhsAnnoToFind)
{
}

bool SIMDIndexJoin::next(std::vector<Match> &tuple)
{
  tuple.clear();

  do
  {
    while(!matchBuffer.empty())
    {
      const MatchPair& m = matchBuffer.front();

      tuple.reserve(m.lhs.size()+1);
      tuple.insert(tuple.begin(), m.lhs.begin(), m.lhs.end());
      tuple.push_back({m.rhs, rhsAnnoToFind});

      matchBuffer.pop_front();
      return true;

    }
  } while (nextMatchBuffer());

  return false;
}

void SIMDIndexJoin::reset()
{
  if(lhs)
  {
    lhs->reset();
  }

  matchBuffer.clear();
}

bool SIMDIndexJoin::nextMatchBuffer()
{
  std::vector<Match> currentLHS;


  Vc::uint32_v valueTemplate(rhsAnnoToFind.val);

  while(matchBuffer.empty() && lhs->next(currentLHS))
  {
    std::unique_ptr<AnnoIt> reachableNodesIt = op->retrieveMatches(currentLHS[lhsIdx]);
    if(reachableNodesIt)
    {
      const bool annoDefDifferent = rhsAnnoToFind.ns != currentLHS[lhsIdx].anno.ns
          || rhsAnnoToFind.name != currentLHS[lhsIdx].anno.name;


      constexpr size_t SIMD_VECTOR_SIZE = Vc::uint32_v::size();

      Vc::uint32_v vAnnoVals;
      Vc::Mask<uint32_t> maskFoundAnnos;

      // use an aligned memory allocator to make SIMD faster

      uint32_t annoVals[SIMD_VECTOR_SIZE];
      uint32_t reachableNodes[SIMD_VECTOR_SIZE];


      Match m;
      bool foundRHS = false;
      do
      {
        foundRHS = false;

        // fill each element of the vector
        for(size_t i=0; i < SIMD_VECTOR_SIZE; i++)
        {
          if(reachableNodesIt->next(m))
          {
            std::vector<Annotation> foundAnnos = annos.getAnnotations(m.node, rhsAnnoToFind.ns, rhsAnnoToFind.name);
            annoVals[i] = (foundAnnos.empty() ? 0 : foundAnnos[0].val);
            reachableNodes[i] = (m.node);

            foundRHS = true;
          }
          else
          {
            annoVals[i] = 0;
            reachableNodes[i] = 0;
          }
        }

        // transform the data to SIMD
        vAnnoVals.load(annoVals, Vc::Aligned);

        // search for values that are the same as a SIMD instruction
        maskFoundAnnos = (vAnnoVals == valueTemplate);
        if(Vc::any_of(maskFoundAnnos))
        {
          for(size_t foundIdx=static_cast<size_t>(maskFoundAnnos.firstOne()); foundIdx < maskFoundAnnos.size(); foundIdx++)
          {
            if(maskFoundAnnos[foundIdx])
            {
              if(annoDefDifferent || op->isReflexive() || currentLHS[lhsIdx].node != reachableNodes[foundIdx])
              {
                matchBuffer.push_back({currentLHS, reachableNodes[foundIdx]});
              }
            }
          }
        }

      } while(foundRHS);
    } // end if reachable nodes iterator valide
  } // end while LHS valid and nothing found yet

  return !matchBuffer.empty();
}


SIMDIndexJoin::~SIMDIndexJoin()
{
}
