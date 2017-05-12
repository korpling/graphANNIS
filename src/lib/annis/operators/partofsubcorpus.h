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

#include <annis/operators/abstractedgeoperator.h>  // for AbstractEdgeOperator
#include <annis/types.h>                           // for ComponentType, Com...

namespace annis { class GraphStorageHolder; }
namespace annis { class StringStorage; }


namespace annis
{

class PartOfSubCorpus : public AbstractEdgeOperator
{
public:
  PartOfSubCorpus(GraphStorageHolder &gsh, const StringStorage &strings);

  virtual std::string operatorString() override
  {
    // TODO: find a good string
    return "???";
  }

  
  virtual ~PartOfSubCorpus();
private:

};
} // end namespace annis
