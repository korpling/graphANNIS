#ifndef COMPONENTCHAINJOIN_H
#define COMPONENTCHAINJOIN_H

#include "../edgedb.h"
#include "../annotationiterator.h"

#include <list>

namespace annis
{

struct ComponentChainEntry
{
  const EdgeDB* edb;
  unsigned int minDistance;
  unsigned int maxDistance;
};

class ComponentChainJoin : public BinaryOperatorIterator
{
public:
  ComponentChainJoin(AnnotationIterator& lhs, AnnotationIterator& rhs,
                     const std::list<ComponentChainEntry>& entries);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~ComponentChainJoin();
private:
  AnnotationIterator& lhs;
  AnnotationIterator& rhs;
  const std::list<ComponentChainEntry> entries;

  std::list<ComponentChainEntry>::const_iterator itEntries;
};
} // end namespace annis
#endif // COMPONENTCHAINJOIN_H
