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

#include "donothingjoin.h"

#include <algorithm>                      // for move
#include "annis/iterators.h"              // for AnnoIt, Iterator
#include "annis/operators/operator.h"     // for Operator
#include "annis/types.h"                  // for Match, Annotation, nodeid_t
#include "annis/util/comparefunctions.h"  // for checkAnnotationKeyEqual
namespace annis { class DB; }


using namespace annis;

DoNothingJoin::DoNothingJoin()
{
}

DoNothingJoin::~DoNothingJoin()
{

}

bool DoNothingJoin::next(std::vector<Match>& tuple)
{
  tuple.clear();
  return false;
}

void DoNothingJoin::reset()
{
}

