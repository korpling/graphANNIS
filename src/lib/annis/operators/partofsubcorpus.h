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
#include <annis/db.h>

namespace annis { class StringStorage; }


namespace annis
{

class PartOfSubCorpus : public AbstractEdgeOperator
{
public:
  PartOfSubCorpus(DB::GetGSFuncT getGraphStorageFunc, const DB& db);
  PartOfSubCorpus(DB::GetGSFuncT getGraphStorageFunc, const DB& db, unsigned int maxDistance);

  virtual std::string operatorString() override
  {
    // TODO: find a good string
    return "part_of_subcorpus";
  }

  virtual double selectivity() override;

  virtual bool isReflexive() override
  {
    return false;
  }

  
  virtual ~PartOfSubCorpus();
private:
  std::shared_ptr<const ReadableGraphStorage> gs;
};
} // end namespace annis
