#include "pointingrelation.h"
#include "wrapper.h"

using namespace annis;

PointingRelation::PointingRelation(const DB &db, std::string ns, std::string name,
                                   unsigned int minDistance, unsigned int maxDistance)
  : db(db), minDistance(minDistance), maxDistance(maxDistance)
{
  if(ns == "")
  {
    edb = db.getEdgeDB(ComponentType::POINTING, name);
  }
  else
  {
    // directly add the only known edge storage
    const EdgeDB* e = db.getEdgeDB(ComponentType::POINTING, ns, name);
    if(e != nullptr)
    {
      edb.push_back(db.getEdgeDB(ComponentType::POINTING, ns, name));
    }
  }
}

std::unique_ptr<AnnoIt> PointingRelation::retrieveMatches(const Match &lhs)
{
  ListWrapper* w = new ListWrapper();

  // add the rhs nodes of all of the edge storages
  for(auto e : edb)
  {
    EdgeIterator* it = e->findConnected(lhs.node, minDistance, maxDistance);
    for(auto m = it->next(); m.first; m = it->next())
    {
      w->addMatch(m.second);
    }
    delete it;
  }

  return std::unique_ptr<AnnoIt>(w);
}

bool PointingRelation::filter(const Match &lhs, const Match &rhs)
{
  // check if the two nodes are connected in *any* of the edge storages
  for(auto e : edb)
  {
    if(e->isConnected(Init::initEdge(lhs.node, rhs.node), minDistance, maxDistance))
    {
      return true;
    }
  }
  return false;
}

PointingRelation::~PointingRelation()
{

}

