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

#include <annis/join/simdindexjoin.h>

#include <annis/operators/operator.h>     // for Operator
#include <annis/util/comparefunctions.h>  // for checkAnnotationEqual
#include <algorithm>                      // for move
#include <future>                         // for future, async, launch, laun...
#include <list>                           // for list
#include "annis/iterators.h"              // for AnnoIt, Iterator
#include "annis/types.h"                  // for Match, Annotation, nodeid_t
#include "annis/util/threadpool.h"        // for ThreadPool

using namespace annis;

SIMDIndexJoin::SIMDIndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                             std::shared_ptr<Operator> op,
                             const AnnoStorage<nodeid_t>& annos,
                             Annotation rhsAnnoToFind, boost::optional<Annotation> constAnno)
  : lhs(lhs), lhsIdx(lhsIdx), op(op), annos(annos), rhsAnnoToFind(rhsAnnoToFind), constAnno(constAnno)
{
}

bool SIMDIndexJoin::next(std::vector<Match> &tuple)
{
  tuple.clear();

  do
  {
    while(!matchBuffer.empty())
    {
      const nodeid_t& n = matchBuffer.front();

      tuple.reserve(currentLHS.size()+1);
      tuple.insert(tuple.begin(), currentLHS.begin(), currentLHS.end());
      if(constAnno)
      {
        tuple.push_back({n, *constAnno});
      }
      else
      {
        tuple.push_back({n, rhsAnnoToFind});
      }

      matchBuffer.pop_front();
      return true;

    }
  } while (fillMatchBuffer());

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

bool SIMDIndexJoin::fillMatchBuffer()
{
  Vc::uint32_v valueTemplate = rhsAnnoToFind.val;

  while(matchBuffer.empty() && lhs->next(currentLHS))
  {
    Vc::uint32_v v_lhsNode = currentLHS[lhsIdx].node;

    std::unique_ptr<AnnoIt> reachableNodesIt = op->retrieveMatches(currentLHS[lhsIdx]);
    if(reachableNodesIt)
    {
      const bool reflexiveCheckNeeded =
          !(op->isReflexive()
            || rhsAnnoToFind.ns != currentLHS[lhsIdx].anno.ns
          || rhsAnnoToFind.name != currentLHS[lhsIdx].anno.name);

      annoVals.clear();
      reachableNodes.clear();

      Match m;
      while(reachableNodesIt->next(m))
      {
        boost::optional<Annotation> foundAnnos = annos.getAnnotations(m.node, rhsAnnoToFind.ns, rhsAnnoToFind.name);
        if(foundAnnos)
        {
          annoVals.push_back(foundAnnos->val);
          reachableNodes.push_back(m.node);
        }
      }

      // add padding to make sure there is no invalid memory when copying to SIMD
      size_t padding = Vc::uint32_v::size() - (annoVals.size() % Vc::uint32_v::size());
      if(padding > 0)
      {
        annoVals.resize(annoVals.size()+padding, 0);
        reachableNodes.reserve(annoVals.size()+padding);
      }

      if(reflexiveCheckNeeded)
      {

        for(size_t i=0; i < annoVals.size() && i < reachableNodes.size(); i += Vc::uint32_v::size())
        {
          // transform the data to SIMD
          Vc::uint32_v v_annoVals(&annoVals[i]);
          Vc::uint32_v v_reachableNodes(&reachableNodes[i]);

          // search for values that are the same and don't have the same LHS and RHS node
          Vc::Mask<uint32_t> v_valid = (v_annoVals == valueTemplate) && (v_lhsNode != v_reachableNodes);

          // collect results
          collectResults(v_valid, i);
        }
      }
      else
      {
        for(size_t i=0; i < annoVals.size() && i < reachableNodes.size(); i += Vc::uint32_v::size())
        {
          // transform the data to SIMD
          Vc::uint32_v v_annoVals(&annoVals[i]);

          // search for values that are the same
          Vc::Mask<uint32_t> v_valid = (v_annoVals == valueTemplate);

          // collect results
          collectResults(v_valid, i);
        }
      }
    } // end if reachable nodes iterator valide
  } // end while LHS valid and nothing found yet

  return !matchBuffer.empty();
}


SIMDIndexJoin::~SIMDIndexJoin()
{
}
