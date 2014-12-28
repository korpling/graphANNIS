#ifndef OPERATOR_H
#define OPERATOR_H

#include "annotationiterator.h"
#include <list>

namespace annis
{

/**
 * @brief Helper class to wrap a list of matches and make it an AnnoIt
 */
class ListIterator : public AnnoIt
{
public:

  ListIterator(const std::list<Match>& orig)
    : orig(orig), anyAnno(Init::initAnnotation())
  {
    reset();
  }

  virtual bool hasNext()
  {
    return origIt != orig.end();
  }

  virtual Match next()
  {
    Match result = *origIt;
    origIt++;
    return result;
  }
  virtual void reset()
  {
    origIt = orig.begin();
  }

  virtual const Annotation& getAnnotation()
  {
    // TODO: what kind of annotation can we return here?
    // maybe it's even better to remove this function from the interface
    // as soon as operators are no BinaryIt any longer.
    return anyAnno;
  }

  virtual ~ListIterator() {}
private:
  const std::list<Match>& orig;
  std::list<Match>::const_iterator origIt;

  Annotation anyAnno;
};

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
