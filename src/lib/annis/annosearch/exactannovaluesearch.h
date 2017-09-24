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

#include <stdint.h>             // for int64_t
#include <list>                 // for list, list<>::const_iterator
#include <string>               // for string
#include <unordered_set>        // for unordered_set
#include <utility>              // for pair
#include <annis/annostorage.h>  // for AnnoStorage, AnnoStorage<>::InverseAn...
#include <annis/types.h>        // for Annotation (ptr only), Match (ptr only)
#include <annis/annosearch/estimatedsearch.h>   // for EstimatedSearch

namespace annis { class DB; }

namespace annis
{

class ExactAnnoValueSearch : public EstimatedSearch
{
  using ItType = AnnoStorage<nodeid_t>::InverseAnnoMap_t::const_iterator;
  using Range = std::pair<ItType, ItType>;

public:

  /**
   * @brief Find annotations by name
   * @param db
   * @param annoName
   */
  ExactAnnoValueSearch(const DB &db, const std::string& annoNamspace, const std::string& annoName, const std::string& annoValue);
  ExactAnnoValueSearch(const DB &db, const std::string& annoName, const std::string& annoValue);

  virtual ~ExactAnnoValueSearch();


  virtual bool next(Match& result) override;
  virtual void reset() override;

  const std::unordered_set<Annotation>& getValidAnnotations()
  {
    if(!validAnnotationInitialized)
    {
      initializeValidAnnotations();
    }
    return validAnnotations;
  }
  
  std::int64_t guessMaxCount() const override;

  virtual std::string debugString() const override {return debugDescription;}


private:
  const DB& db;

  std::list<Range> searchRanges;
  std::list<Range>::const_iterator currentRange;
  ItType it;

  bool validAnnotationInitialized;
  std::unordered_set<Annotation> validAnnotations;

  const std::string debugDescription;

  std::unordered_set<nodeid_t> uniqueResultFilter;

private:

  void initializeValidAnnotations();

};


} // end namespace annis

