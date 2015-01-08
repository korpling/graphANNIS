#ifndef WRAPPER_H
#define WRAPPER_H

#include <queue>
#include "../annotationiterator.h"

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
    orig.push(m);
  }

  virtual bool hasNext()
  {
    return !orig.empty();
  }

  virtual Match next()
  {
    Match result = orig.front();
    orig.pop();
    return result;
  }

  virtual void reset()
  {
    while(!orig.empty())
    {
      orig.pop();
    }
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
  std::queue<Match> orig;

  Annotation anyAnno;
};


class JoinWrapIterator : public ListWrapper
{
public:

  JoinWrapIterator(std::shared_ptr<Join> wrappedJoin,
                        bool wrapLeftOperand = false)
    : wrappedJoin(wrappedJoin),
      wrapLeftOperand(wrapLeftOperand)
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

  virtual void setOther(std::shared_ptr<JoinWrapIterator> otherInnerWrapper)
  {
    JoinWrapIterator::otherInnerWrapper = otherInnerWrapper;
  }

private:
  std::shared_ptr<Join> wrappedJoin;
  std::shared_ptr<JoinWrapIterator> otherInnerWrapper;
  bool wrapLeftOperand;

  void checkIfNextCallNeeded();
};
} // end namespace annis


#endif // WRAPPER_H
