#ifndef ANNOTATIONITERATOR_H
#define ANNOTATIONITERATOR_H

#include "types.h"
#include <memory>

namespace annis
{

class AnnoIt
{
public:
  virtual bool hasNext() = 0;
  virtual Match next() = 0;
  virtual void reset() = 0;

  virtual const Annotation& getAnnotation() = 0;

  virtual ~AnnoIt() {}
};

class CacheableAnnoIt : public AnnoIt
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

class BinaryIt
{
public:
  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs) = 0;
  virtual BinaryMatch next() = 0;
  virtual void reset() = 0;

  virtual ~BinaryIt() {}
};

} // end namespace annis

#endif // ANNOTATIONITERATOR_H
