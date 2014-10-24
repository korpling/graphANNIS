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
  virtual void reset() = 0;

  virtual ~AnnotationIterator() {}
};

class EdgeIterator
{
public:
  virtual std::pair<bool, nodeid_t> next() = 0;

  virtual ~EdgeIterator() {}
};

class BinaryOperatorIterator
{
public:
  virtual BinaryMatch next() = 0;
  virtual void reset() = 0;

  virtual ~BinaryOperatorIterator() {}
};

} // end namespace annis

#endif // ANNOTATIONITERATOR_H
