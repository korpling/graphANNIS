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

#include <annis/db.h>
#include <annis/iterators.h>
#include <annis/util/comparefunctions.h>

#include <set>
#include <unordered_set>

namespace annis
{

class EstimatedSearch : public AnnoIt
{
public:
  virtual std::int64_t guessMaxCount() const {return -1;}

  virtual std::string debugString() const {return "";}
};

class AnnotationSearch : public EstimatedSearch
{
public:
  virtual const std::unordered_set<Annotation>& getValidAnnotations() = 0;
  
  virtual ~AnnotationSearch() {}
};

class AnnotationKeySearch : public EstimatedSearch
{
public:
  virtual const std::set<AnnotationKey>& getValidAnnotationKeys() = 0;
  
  virtual ~AnnotationKeySearch() {}
};

} // end namespace annis
