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

#include "nodebyedgeannosearch.h"



using namespace annis;

NodeByEdgeAnnoSearch::NodeByEdgeAnnoSearch(std::vector<std::shared_ptr<const ReadableGraphStorage> > gs, std::set<Annotation> validEdgeAnnos,
                                           std::function<std::list<Annotation> (nodeid_t)> nodeAnnoMatchGenerator,
                                           bool maximalOneNodeAnno,
                                           std::int64_t wrappedNodeCountEstimate, std::string debugDescription)
 : nodeAnnoMatchGenerator(nodeAnnoMatchGenerator),
   maximalOneNodeAnno(maximalOneNodeAnno),
   wrappedNodeCountEstimate(wrappedNodeCountEstimate),
   debugDescription(debugDescription + " _edgeanno_")
{
  for(size_t i=0; i < gs.size(); i++)
  {
    for(const Annotation& anno : validEdgeAnnos)
    {
      searchRanges.push_back(gs[i]->getAnnoStorage().inverseAnnotations.equal_range(anno));
    }
  }
  currentRange = searchRanges.begin();

  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
  }

}

bool NodeByEdgeAnnoSearch::next(Match &m)
{
  do
  {
    if(!currentMatchBuffer.empty())
    {
      m = currentMatchBuffer.front();
      currentMatchBuffer.pop_front();
      return true;
    }
  } while(nextMatchBuffer());

  return false;
}

void NodeByEdgeAnnoSearch::reset()
{
  visited.clear();
  currentMatchBuffer.clear();
  currentRange = searchRanges.begin();
  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
  }
}


NodeByEdgeAnnoSearch::~NodeByEdgeAnnoSearch()
{

}

bool NodeByEdgeAnnoSearch::nextMatchBuffer()
{
  currentMatchBuffer.clear();

  bool valid = false;
  while(!valid && currentRange != searchRanges.end())
  {
    if(it != currentRange->second)
    {
      const Edge& matchingEdge = it->second;

      if(visited.find(matchingEdge.source) == visited.end())
      {
        for(const Annotation& anno : nodeAnnoMatchGenerator(matchingEdge.source))
        {
          currentMatchBuffer.push_back({matchingEdge.source, anno});
        }
        visited.emplace(matchingEdge.source);
        valid = true;
      }

      it++;
    }

    if(it == currentRange->second)
    {
      currentRange++;
      if(currentRange != searchRanges.end())
      {
        it = currentRange->first;
      }
    }
  }


  return valid;
}
