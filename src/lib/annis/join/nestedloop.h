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

#include <annis/types.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/db.h>
#include <annis/iterators.h>
#include <deque>

namespace annis 
{
  class Operator;

  /** 
   * A join that checks all combinations of the left and right matches if their are connected. 
   * 
   * @param lhsIdx the column of the LHS tuple to join on
   * @param rhsIdx the column of the RHS tuple to join on
   */
  class NestedLoopJoin : public Iterator
  {
  public:
    NestedLoopJoin(std::shared_ptr<Operator> op,
      std::shared_ptr<Iterator> lhs, std::shared_ptr<Iterator> rhs,
      size_t lhsIdx, size_t rhsIdx,
      bool materializeInner=true,
      bool leftIsOuter=true);
    virtual ~NestedLoopJoin();

    virtual bool next(std::vector<Match>& tuple) override;
    virtual void reset() override;
  private:
    std::shared_ptr<Operator> op;

    const bool materializeInner;
    const bool leftIsOuter;
    bool initialized;
    
    std::vector<Match> matchOuter;
    std::vector<Match> matchInner;

    std::shared_ptr<Iterator> outer;
    std::shared_ptr<Iterator> inner;
    
    const size_t outerIdx;
    const size_t innerIdx;
    
    bool firstOuterFinished;
    std::deque<std::vector<Match>> innerCache;
    std::deque<std::vector<Match>>::const_iterator itInnerCache;
  private:
    bool fetchNextInner();

  };


} // end namespace annis

