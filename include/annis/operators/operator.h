#pragma once

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
   * Return if this operator is commutative, thus both arguments can be exchanged
   * without changing the result. Per default this is "false".
   */
  virtual bool isCommutative() {return false;}
  
  /**
   * If an operator after construction already knows it can't ever produce
   * any results (e.g. because an edge component does not exist) it can
   * return "false" here to indicate this to the join.
   * @return 
   */
  virtual bool valid() const {return true;}
  
  /**
   * A descripte string of the state of the operator used for debugging.
   * @return 
   */
  virtual std::string description() {return "";}
  
  virtual double selectivity() { return 0.1; }

  virtual ~Operator() {}
};
} // end namespace annis

