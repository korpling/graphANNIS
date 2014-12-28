#ifndef OPERATOR_H
#define OPERATOR_H

#include "annotationiterator.h"

namespace annis
{
class Operator : public BinaryIt
{
public:
  bool filter(const Match& lhs, const Match& rhs) = 0;
};
} // end namespace annis

#endif // OPERATOR_H
