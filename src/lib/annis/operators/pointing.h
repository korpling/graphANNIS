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

#include <string>                  // for string
#include "abstractedgeoperator.h"  // for AbstractEdgeOperator

namespace annis
{

class GraphStorageHolder;
class StringStorage;
struct Annotation;

class Pointing : public AbstractEdgeOperator
{
public:
  Pointing(std::string ns, std::string name,
           DB::GetGSFuncT getGraphStorageFunc,
           const StringStorage &strings,
           unsigned int minDistance = 1, unsigned int maxDistance = 1);

  Pointing(std::string name,
           DB::GetAllGSFuncT getAllGraphStorageFunc,
           const StringStorage &strings,
           unsigned int minDistance = 1, unsigned int maxDistance = 1);

  Pointing(std::string ns, std::string name,
           DB::GetGSFuncT getGraphStorageFunc,
           const StringStorage& strings,
           const Annotation& edgeAnno);


  Pointing(std::string name,
           DB::GetAllGSFuncT getAllGraphStorageFunc,
           const StringStorage& strings,
           const Annotation& edgeAnno);

  virtual std::string operatorString() override
  {
    return "->";
  }
  
  virtual ~Pointing();
private:
};
} // end namespace annis
