#ifndef WRAPPER_H
#define WRAPPER_H

#include <list>
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
    orig.push_back(m);
    origIt = orig.begin();
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

  virtual ~ListWrapper() {}
private:
  std::list<Match> orig;
  std::list<Match>::const_iterator origIt;

  Annotation anyAnno;
};

class EdgeIteratorWrapper : public AnnoIt
{
public:
  EdgeIteratorWrapper(EdgeIterator* orig)
    : orig(orig), anyAnno(Init::initAnnotation())
  {
    reset();
  }

  virtual bool hasNext()
  {
    return current.first;
  }

  virtual Match next()
  {
    Match result;
    if(current.first)
    {
      result.node = current.second;
      result.anno = Init::initAnnotation(); // match any annotation

      current = orig->next();
    }
    return result;
  }
  virtual void reset()
  {
    orig->reset();
    current = orig->next();
  }

  virtual const Annotation& getAnnotation()
  {
    // TODO: what kind of annotation can we return here?
    // maybe it's even better to remove this function from the interface
    // as soon as operators are no BinaryIt any longer.
    return anyAnno;
  }

  virtual ~EdgeIteratorWrapper() {}
private:
  EdgeIterator* orig;
  std::pair<bool, nodeid_t> current;
  Annotation anyAnno;
};

/**
 * @brief Wrap a join as an annotation iterator.
 */
class JoinWrapIterator : public CacheableAnnoIt
{
public:

  JoinWrapIterator(std::shared_ptr<BinaryIt> wrappedIterator, bool wrapLeftOperand = false);

  virtual bool hasNext();
  virtual Match next();
  virtual void reset();

  virtual Match current();

  // TODO: is there any good way of defining this?
  virtual const Annotation& getAnnotation() {return matchAllAnnotation;}

  virtual ~JoinWrapIterator() {}
private:
  Annotation matchAllAnnotation;
  std::shared_ptr<BinaryIt> wrappedIterator;
  BinaryMatch currentMatch;
  bool wrapLeftOperand;
};
} // end namespace annis

#endif // WRAPPER_H
