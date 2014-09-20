#ifndef ANNOTATIONITERATOR_H
#define ANNOTATIONITERATOR_H

#include "types.h"

namespace annis
{

class AnnotationIterator
{
public:
  virtual bool hasNext() = 0;
  virtual Match next() = 0;
};

class EdgeIterator
{
public:
  virtual std::pair<bool, nodeid_t> next() = 0;
};

} // end namespace annis

#endif // ANNOTATIONITERATOR_H
