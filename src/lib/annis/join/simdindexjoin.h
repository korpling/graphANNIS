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

#include <annis/iterators.h>  // for Iterator
#include <annis/types.h>      // for Match, nodeid_t
#include <stddef.h>           // for size_t
#include <functional>         // for function
#include <future>             // for future
#include <list>               // for list
#include <memory>             // for shared_ptr, make_shared
#include <thread>             // for thread
#include <vector>             // for vector

#include <annis/annostorage.h>

#include <Vc/Vc>

namespace annis { class Operator; }


namespace annis
{

class SIMDIndexJoin : public Iterator
{
public:


public:
  SIMDIndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                std::shared_ptr<Operator> op,
                const AnnoStorage<nodeid_t>& annos,
                Annotation rhsAnnoToFind);

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~SIMDIndexJoin();
private:

  std::shared_ptr<Iterator> lhs;
  const size_t lhsIdx;

  std::shared_ptr<Operator> op;
  const AnnoStorage<nodeid_t>& annos;
  const Annotation rhsAnnoToFind;

  std::list<nodeid_t> matchBuffer;
  std::vector<Match> currentLHS;

  std::vector<uint32_t, Vc::Allocator<uint32_t>> annoVals;
  std::vector<nodeid_t, Vc::Allocator<uint32_t>> reachableNodes;


private:

  bool fillMatchBuffer();

  inline void collectResults(Vc::Mask<uint32_t>& v_valid, const size_t& offset)
  {
    if(Vc::any_of(v_valid))
    {
      for(size_t j=0; j < Vc::Mask<uint32_t>::size() && (offset+j) < reachableNodes.size(); j++)
      {
        matchBuffer.emplace_back(reachableNodes[offset+j]);
      }
    }
  }
};
}

