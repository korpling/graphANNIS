#include "nodebyedgeannosearch.h"



using namespace annis;

NodeByEdgeAnnoSearch::NodeByEdgeAnnoSearch(std::vector<std::shared_ptr<const ReadableGraphStorage> > gs, std::set<Annotation> validEdgeAnnos,
                                           std::function<std::list<Match> (nodeid_t)> nodeAnnoMatchGenerator,
                                           std::int64_t wrappedNodeCountEstimate, std::string debugDescription)
 : nodeAnnoMatchGenerator(nodeAnnoMatchGenerator),
   wrappedNodeCountEstimate(wrappedNodeCountEstimate),
   debugDescription(debugDescription + " _edgeanno_")
{
  for(int i=0; i < gs.size(); i++)
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
  if(currentRange != searchRanges.end() && it != currentRange->second)
  {
    const Edge& matchingEdge = it->second;
    currentMatchBuffer = nodeAnnoMatchGenerator(matchingEdge.source);
    it++;
    if(it == currentRange->second)
    {
      currentRange++;
      if(currentRange != searchRanges.end())
      {
        it = currentRange->first;
      }
    }
    return true;
  }
  else
  {
    return false;
  }
}
