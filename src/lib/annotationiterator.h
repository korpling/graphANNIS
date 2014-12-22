#ifndef ANNOTATIONITERATOR_H
#define ANNOTATIONITERATOR_H

#include "types.h"
#include <memory>

namespace annis
{

class AnnotationIterator
{
public:
  virtual bool hasNext() = 0;
  virtual Match next() = 0;
  virtual void reset() = 0;

  virtual const Annotation& getAnnotation() = 0;

  virtual ~AnnotationIterator() {}
};

class CacheableAnnoIt : public AnnotationIterator
{
public:
  virtual Match current() = 0;
  virtual ~CacheableAnnoIt() {}
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
  virtual void init(std::shared_ptr<AnnotationIterator> lhs, std::shared_ptr<AnnotationIterator> rhs) = 0;
  virtual BinaryMatch next() = 0;
  virtual void reset() = 0;

  virtual ~BinaryOperatorIterator() {}
};

} // end namespace annis

#endif // ANNOTATIONITERATOR_H
