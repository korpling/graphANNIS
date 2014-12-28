#ifndef WRAPPER_H
#define WRAPPER_H

#include "../annotationiterator.h"

namespace annis
{



/**
 * @brief Wrap a join as an annotation iterator.
 */
class JoinWrapIterator : public CacheableAnnoIt
{
public:

  JoinWrapIterator(std::shared_ptr<BinaryIt> wrappedIterator, bool wrapLeftOperand = false);

  virtual bool hasNext();
  virtual Match next();
  virtual void reset();

  virtual Match current();

  // TODO: is there any good way of defining this?
  virtual const Annotation& getAnnotation() {return matchAllAnnotation;}

  virtual ~JoinWrapIterator() {}
private:
  Annotation matchAllAnnotation;
  std::shared_ptr<BinaryIt> wrappedIterator;
  BinaryMatch currentMatch;
  bool wrapLeftOperand;
};
} // end namespace annis

#endif // WRAPPER_H
