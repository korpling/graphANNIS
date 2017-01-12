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

#include <annis/filter.h>

using namespace annis;


Filter::Filter(std::shared_ptr<Operator> op, std::shared_ptr<Iterator> inner,
  size_t lhsIdx, size_t rhsIdx)
  : op(op), inner(inner), lhsIdx(lhsIdx), rhsIdx(rhsIdx)
{

}

// TODO: explicitly test the filter function
bool Filter::next(std::vector<Match>& tuple)
{
  tuple.clear();
  bool found = false;

  if(op && inner)
  {
    std::vector<Match> innerMatch;
    while(!found && inner->next(innerMatch))
    {
      if(op->filter(innerMatch[lhsIdx], innerMatch[rhsIdx]))
      {
        tuple.reserve(innerMatch.size());
        tuple.insert(tuple.end(), innerMatch.begin(), innerMatch.end());
        found = true;
      }
    }
  }

  return found;
}

void Filter::reset()
{
  if(inner)
  {
    inner->reset();
  }
}

Filter::~Filter()
{

}
