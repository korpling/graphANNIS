#pragma once

#include <annis/types.h>
#include <annis/iterators.h>

#include <queue>
#include <list>
#include <memory>

#include <annis/annosearch/annotationsearch.h>

namespace annis
{

  /**
   * @brief Helper class which has an internal list of matches and wraps it as a AnnoIt
   * Thus this class is a kind of materialized result
   */
  class ListWrapper : public AnnoIt
  {
  public:

    ListWrapper();

    void addMatch(const Match& m)
    {
      orig.push_back(m);
    }

    void addMatch(const nodeid_t& m)
    {
      orig.push_back({m,
        {0, 0, 0}});
    }

    virtual bool next(Match& result) override
    {
      if(orig.empty())
      {
        return false;
      }
      else
      {
        result = std::move(orig.front());
        orig.pop_front();
        return true;
      }
    }

    virtual void reset() override
    {
      orig.clear();
    }

    virtual ~ListWrapper();

  protected:

    bool internalEmpty()
    {
      return orig.empty();
    }

  private:
    std::deque<Match > orig;
  };

  class JoinWrapIterator : public ListWrapper
  {
  public:

    JoinWrapIterator(std::shared_ptr<Iterator> wrappedJoin,
      size_t lhsIdx, size_t rhsIdx,
      bool wrapLeftOperand = false)
      : wrappedJoin(wrappedJoin),
        lhsIdx(lhsIdx), rhsIdx(rhsIdx),
        wrapLeftOperand(wrapLeftOperand)
    {

    }

    virtual bool next(Match& result) override
    {
      checkIfNextCallNeeded();
      return ListWrapper::next(result);
    }

    virtual void reset() override;

    virtual void setOther(std::weak_ptr<JoinWrapIterator> other)
    {
      otherInnerWrapper = other;
    }

    virtual ~JoinWrapIterator()
    {
    }

  private:
    std::shared_ptr<Iterator> wrappedJoin;
    std::weak_ptr<JoinWrapIterator> otherInnerWrapper;
    
    size_t lhsIdx;
    size_t rhsIdx;
    
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
  class ConstAnnoWrapper : public EstimatedSearch
  {
  public:

    ConstAnnoWrapper(Annotation constAnno, std::shared_ptr<EstimatedSearch> delegate)
      : constAnno(constAnno), delegate(delegate)
    {

    }

    virtual bool next(Match& m) override
    {
      bool found = delegate->next(m);
      m.anno = constAnno;
      return found;
    }

    virtual void reset() override
    {
      delegate->reset();
    }

    std::shared_ptr<EstimatedSearch> getDelegate()
    {
      return delegate;
    }
    
    std::int64_t guessMaxCount() const override
    {
      return delegate->guessMaxCount();
    }


    virtual std::string debugString() const override
    {
      return delegate->debugString();
    }


    virtual ~ConstAnnoWrapper()
    {
    }
  private:
    Annotation constAnno;
    std::shared_ptr<EstimatedSearch> delegate;
  };

  /**
   * Similar to ListWrapper but only wraps a single element
   */
  class SingleElementWrapper : public AnnoIt
  {
  public:

    SingleElementWrapper(const Match& m)
      : m(m), valid(true)
    {

    }

    virtual bool next(Match& result) override
    {
      if(valid)
      {
        valid = false;
        result = m;
        return true;
      }
      else
      {
        return false;
      }
    }

    virtual void reset() override
    {
      valid = true;
    }

    virtual ~SingleElementWrapper()
    {
    }

  private:
    Match m;
    bool valid;
  };

} // end namespace annis

