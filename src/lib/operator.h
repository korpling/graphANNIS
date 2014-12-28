#ifndef OPERATOR_H
#define OPERATOR_H

#include "annotationiterator.h"
#include <list>

namespace annis
{


class Operator
{
public:

  /**
   * @brief Return all matches for a certain left-hand-side
   * @param lhs
   * @return
   */
  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs) = 0;
  /**
   * @brief Filter two match candidates.
   * @param lhs
   * @param rhs
   * @return
   */
  virtual bool filter(const Match& lhs, const Match& rhs) = 0;

  virtual ~Operator() {}
};
} // end namespace annis

#endif // OPERATOR_H
