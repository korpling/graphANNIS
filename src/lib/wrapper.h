#ifndef WRAPPER_H
#define WRAPPER_H

#include "iterators.h"
#include "join.h"

#include <deque>
#include <memory>

namespace annis
{

/**
 * @brief Helper class which has an internal list of matches and wraps it as a AnnoIt
 * Thus this class is a kind of materialized result
 */
class ListWrapper : public AnnoIt
{
public:

  ListWrapper()
    : anyAnno(Init::initAnnotation())
  {
    reset();
  }

  void addMatch(const Match& m)
  {
    orig.push_back(m);
  }

  void addMatch(const nodeid_t& m)
  {
    orig.push_back(Init::initMatch(anyAnno, m));
  }

  virtual bool hasNext()
  {
    return !orig.empty();
  }

  virtual Match next()
  {
    Match result = orig.front();
    orig.pop_front();
    return result;
  }

  virtual void reset()
  {
    orig.clear();
  }

  virtual const Annotation& getAnnotation()
  {
    // TODO: what kind of annotation can we return here?
    // maybe it's even better to remove this function from the interface
    // as soon as operators are no BinaryIt any longer.
    return anyAnno;
  }

  virtual ~ListWrapper() {}
protected:
  size_t internalListSize()
  {
    return orig.size();
  }

private:
  std::deque<Match> orig;

  Annotation anyAnno;
};


class JoinWrapIterator : public ListWrapper
{
public:

  JoinWrapIterator(std::shared_ptr<Join> wrappedJoin, const Annotation& rightAnno = Init::initAnnotation(),
                        bool wrapLeftOperand = false)
    : wrappedJoin(wrappedJoin),
      wrapLeftOperand(wrapLeftOperand),
      rightAnno(rightAnno)
  {

  }

  virtual Match next()
  {
    checkIfNextCallNeeded();
    return ListWrapper::next();
  }

  virtual bool hasNext()
  {
    checkIfNextCallNeeded();
    return ListWrapper::hasNext();
  }

  virtual void reset();

  virtual const Annotation& getAnnotation()
  {
    return rightAnno;
  }

  virtual void setOther(std::shared_ptr<JoinWrapIterator> otherInnerWrapper)
  {
    JoinWrapIterator::otherInnerWrapper = otherInnerWrapper;
  }

  virtual ~JoinWrapIterator() {};

private:
  std::shared_ptr<Join> wrappedJoin;
  std::shared_ptr<JoinWrapIterator> otherInnerWrapper;
  bool wrapLeftOperand;
  const Annotation& rightAnno;

  void checkIfNextCallNeeded();
};
} // end namespace annis


#endif // WRAPPER_H
