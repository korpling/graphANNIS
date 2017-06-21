/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/
#pragma once

#include <annis/annosearch/annotationsearch.h>  // for EstimatedSearch
#include <annis/iterators.h>                    // for AnnoIt
#include <annis/types.h>                        // for Match, Annotation
#include <stddef.h>                             // for size_t
#include <stdint.h>                             // for int64_t
#include <algorithm>                            // for move
#include <deque>                                // for deque
#include <memory>                               // for shared_ptr, __shared_ptr
#include <string>                               // for string

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

