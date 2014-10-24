#ifndef NESTEDLOOPJOIN_H
#define NESTEDLOOPJOIN_H

#include "types.h"
#include "annotationiterator.h"
#include "edgedb.h"

namespace annis
{

/** A join that checks all combinations of the left and right matches if their are connected. */
class NestedLoopJoin : public BinaryOperatorIterator
{
public:
  NestedLoopJoin(const EdgeDB* edb, AnnotationIterator &left, AnnotationIterator &right,
                 unsigned int minDistance = 1, unsigned int maxDistance = 1);
  virtual ~NestedLoopJoin();

  virtual BinaryMatch next();
  virtual void reset();
private:
  const EdgeDB* edb;
  AnnotationIterator& left;
  AnnotationIterator& right;
  unsigned int minDistance;
  unsigned int maxDistance;

  Match matchLeft;
  Match matchRight;

};

} // end namespace annis

#endif // NESTEDLOOPJOIN_H
