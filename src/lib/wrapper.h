#ifndef WRAPPER_H
#define WRAPPER_H

#include "iterators.h"
#include "db.h"

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
  }

  void addMatch(const Match& m)
  {
    orig.push(m);
  }

  void addMatch(const nodeid_t& m)
  {
    orig.push(Init::initMatch({0, 0, 0}, m));
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

  virtual ~JoinWrapIterator() {}

private:
  std::shared_ptr<BinaryIt> wrappedJoin;
  std::weak_ptr<JoinWrapIterator> otherInnerWrapper;
  bool wrapLeftOperand;

  void checkIfNextCallNeeded();
};

  /**
   * An annotation iterator that wraps another annotation iterator, but replaces
   * the node annotation value with a constant value.
   * The node ID will be the same as given by the wrapped iterator.
   * @param db
   * @param delegate
   */
  class ConstAnnoWrapper : public AnnoIt
  {
  public:

    ConstAnnoWrapper(Annotation constAnno, std::shared_ptr<AnnoIt> delegate)
      : constAnno(constAnno), delegate(delegate)
    {

    }

    virtual bool hasNext()
    {
      return delegate->hasNext();
    }

    virtual Match next()
    {
      Match m = delegate->next();
      m.anno = constAnno;
      return m;
    }

    virtual void reset()
    {
      delegate->reset();
    }
    
    std::shared_ptr<AnnoIt> getDelegate() { return delegate;}

    virtual ~ConstAnnoWrapper()
    {
    }
  private:
    Annotation constAnno;
    std::shared_ptr<AnnoIt> delegate;
  };

} // end namespace annis


#endif // WRAPPER_H
