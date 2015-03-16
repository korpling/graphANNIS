#ifndef WRAPPER_H
#define WRAPPER_H

#include "iterators.h"

#include <queue>
#include <list>
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
  {
    reset();
  }

  void addMatch(const Match& m)
  {
    orig.push(m);
  }

  void addMatch(const nodeid_t& m)
  {
    static const Annotation anyAnno = Init::initAnnotation();
    orig.push(Init::initMatch(anyAnno, m));
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

  virtual ~ListWrapper() {}

protected:
  size_t internalListSize()
  {
    return orig.size();
  }

private:
  std::queue<Match, std::list<Match> > orig;
};


class JoinWrapIterator : public ListWrapper
{
public:

  JoinWrapIterator(std::shared_ptr<BinaryIt> wrappedJoin,
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

  virtual void setOther(std::weak_ptr<JoinWrapIterator> otherInnerWrapper)
  {
    JoinWrapIterator::otherInnerWrapper = otherInnerWrapper;
  }

  virtual ~JoinWrapIterator() {};

private:
  std::shared_ptr<BinaryIt> wrappedJoin;
  std::weak_ptr<JoinWrapIterator> otherInnerWrapper;
  bool wrapLeftOperand;

  void checkIfNextCallNeeded();
};
} // end namespace annis


#endif // WRAPPER_H
