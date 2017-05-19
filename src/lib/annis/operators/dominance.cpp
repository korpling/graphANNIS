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

#include "dominance.h"
#include "annis/operators/abstractedgeoperator.h"  // for AbstractEdgeOperator
#include "annis/types.h"                           // for ComponentType, Com...

namespace annis { class StringStorage; }

using namespace annis;

Dominance::Dominance(std::string ns, std::string name,
                   GraphStorageHolder::GetFuncT getGraphStorageFunc,
                   const StringStorage& strings,
                   unsigned int minDistance, unsigned int maxDistance)
  : AbstractEdgeOperator(ComponentType::DOMINANCE, ns, name,
                         getGraphStorageFunc, strings, minDistance, maxDistance)
{
}

Dominance::Dominance(std::string name,
                   GraphStorageHolder::GetAllFuncT getAllGraphStorageFunc,
                   const StringStorage& strings,
                   unsigned int minDistance, unsigned int maxDistance)
  : AbstractEdgeOperator(ComponentType::DOMINANCE, name,
                         getAllGraphStorageFunc, strings, minDistance, maxDistance)
{
}

Dominance::Dominance(std::string ns, std::string name,
                   GraphStorageHolder::GetFuncT getGraphStorageFunc,
                   const StringStorage &strings, const Annotation &edgeAnno)
  : AbstractEdgeOperator(ComponentType::DOMINANCE, ns, name,
                         getGraphStorageFunc, strings, edgeAnno)
{
}

Dominance::Dominance(std::string name,
                   GraphStorageHolder::GetAllFuncT getAllGraphStorageFunc,
                   const StringStorage &strings, const Annotation &edgeAnno)
  : AbstractEdgeOperator(ComponentType::DOMINANCE, name,
                         getAllGraphStorageFunc, strings, edgeAnno)
{
}
Dominance::~Dominance()
{

}

