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

#include "estimatedsearch.h"

using namespace annis;

BufferedEstimatedSearch::BufferedEstimatedSearch(bool maximalOneNodeAnno)
  : maximalOneNodeAnno(maximalOneNodeAnno)
{

}

bool BufferedEstimatedSearch::next(Match &m)
{
  do
  {
    if(!currentMatchBuffer.empty())
    {
      m = currentMatchBuffer.front();
      currentMatchBuffer.pop_front();
      return true;
    }
  } while(nextMatchBuffer(currentMatchBuffer));

  return false;
}

void BufferedEstimatedSearch::reset()
{
  currentMatchBuffer.clear();
}

BufferedEstimatedSearch::~BufferedEstimatedSearch()
{

}
