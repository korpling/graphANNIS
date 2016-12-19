#include "nodebyedgeannosearch.h"



using namespace annis;

NodeByEdgeAnnoSearch::NodeByEdgeAnnoSearch(const ReadableGraphStorage& gs, std::set<Annotation> validEdgeAnnos,
                                           std::function<std::list<Match> (nodeid_t)> nodeAnnoMatchGenerator)
 : gs(gs), validEdgeAnnos(validEdgeAnnos), nodeAnnoMatchGenerator(nodeAnnoMatchGenerator)
{

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
}

NodeByEdgeAnnoSearch::~NodeByEdgeAnnoSearch()
{

}

bool NodeByEdgeAnnoSearch::nextMatchBuffer()
{
  return false;
}
