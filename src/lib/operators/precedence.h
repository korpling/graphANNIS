#ifndef PRECEDENCE_H
#define PRECEDENCE_H

#include "db.h"
#include "../annotationiterator.h"

namespace annis
{

class Precedence : public BinaryOperatorIterator
{
public:
  Precedence(DB &db, AnnotationIterator& left, AnnotationIterator& right,
             unsigned int minDistance=1, unsigned int maxDistance=1);
  virtual ~Precedence();

  virtual BinaryMatch next();
  virtual void reset();

private:
  AnnotationIterator& left;
  AnnotationIterator& right;
  unsigned int minDistance;
  unsigned int maxDistance;

  BinaryOperatorIterator* actualIterator;
};

} // end namespace annis

#endif // PRECEDENCE_H
