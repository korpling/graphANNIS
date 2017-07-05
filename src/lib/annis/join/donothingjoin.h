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
#include <annis/types.h>      // for Annotation, Match, nodeid_t
#include <stddef.h>           // for size_t
#include <functional>         // for function
#include <list>               // for list
#include <memory>             // for shared_ptr, unique_ptr
#include <vector>             // for vector
namespace annis { class DB; }
namespace annis { class Operator; }

namespace annis
{

/**
 * A join that takes the left argument as a seed, finds all connected nodes
 * (probably using and index of the graph storage) and checks the condition for each node.
 * This join is not parallized.
 */
class DoNothingJoin : public Iterator
{
public:
  DoNothingJoin();
  virtual ~DoNothingJoin();

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;
private:


private:
};



} // end namespace annis

