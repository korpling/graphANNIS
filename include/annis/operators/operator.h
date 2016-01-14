#ifndef OPERATOR_H
#define OPERATOR_H

#include <list>
#include <memory>

#include <annis/iterators.h>


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
  
  /**
   * If an operator after construction already knows it can't ever produce
   * any results (e.g. because an edge component does not exist) it can
   * return "false" here to indicate this to the join.
   * @return 
   */
  virtual bool valid() const {return true;}

  virtual ~Operator() {}
};
} // end namespace annis

#endif // OPERATOR_H
