#include "componenttypeiterator.h"

using namespace annis;

ComponentTypeIterator::ComponentTypeIterator(const DB& db, ComponentType type, nodeid_t sourceNode)
  :sourceNode(sourceNode), currentEdgeIterator(NULL), currentComponent(0)
{
  components = db.getAllEdgeDBForType(type);
  currentEdgeIterator = components[currentComponent]->findConnected(sourceNode);
}

std::pair<bool, nodeid_t> ComponentTypeIterator::next()
{
  while(currentComponent < components.size())
  {
    std::pair<bool, nodeid_t> internal = currentEdgeIterator->next();
    if(internal.first)
    {
      return internal;
    }
    else
    {
      currentComponent++;
      delete currentEdgeIterator;
      if(currentComponent < components.size())
      {
        currentEdgeIterator = components[currentComponent]->findConnected(sourceNode);
      }
      else
      {
        currentEdgeIterator = NULL;
      }


    }
  }
  std::pair<bool, nodeid_t> result;
  result.first = false;
  return result;
}

ComponentTypeIterator::~ComponentTypeIterator()
{
  delete currentEdgeIterator;
}
