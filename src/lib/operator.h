#ifndef OPERATOR_H
#define OPERATOR_H

#include <list>
#include <memory>

#include "iterators.h"


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

  /**
   * @brief Return if this operator is reflexive.
   * Reflexive means that the result can contain the same match as LHS and RHS.
   * "Same" is defined as having the same node ID and an equal annotation.
   * Per default an operator is reflexive, if you want to change this behavior overrride this function.
   * @return
   */
  virtual bool isReflexive() {return true;}

  virtual ~Operator() {}
};
} // end namespace annis

#endif // OPERATOR_H
