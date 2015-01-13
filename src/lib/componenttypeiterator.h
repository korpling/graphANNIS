#ifndef COMPONENTTYPEITERATOR_H
#define COMPONENTTYPEITERATOR_H

#include "iterators.h"
#include "db.h"

#include <vector>
#include <list>

namespace annis
{

/**
 * @brief An iterator over all components of a type.
 */
class ComponentTypeIterator : public EdgeIterator
{
public:
  ComponentTypeIterator(const DB& db, ComponentType type, nodeid_t sourceNode);

  virtual std::pair<bool, nodeid_t> next();

  virtual ~ComponentTypeIterator();
private:
  nodeid_t sourceNode;
  std::unique_ptr<EdgeIterator> currentEdgeIterator;
  std::vector<const EdgeDB*> components;
  size_t currentComponent;
};

} // end namespace annis

#endif // COMPONENTTYPEITERATOR_H
