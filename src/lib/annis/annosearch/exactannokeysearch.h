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

#include <annis/annosearch/annotationsearch.h>

#include <annis/annostorage.h>

namespace annis
{

class ExactAnnoKeySearch : public AnnotationKeySearch
{
  using ItAnnoNode = AnnoStorage<nodeid_t>::InverseAnnoMap_t::const_iterator;
  using ItAnnoKey = btree::btree_map<AnnotationKey, std::uint64_t>::const_iterator;

public:
  /**
   * @brief Find all annotations.
   * @param db
   */
  ExactAnnoKeySearch(const DB& db);
  /**
   * @brief Find annotations by name
   * @param db
   * @param annoName
   */
  ExactAnnoKeySearch(const DB& db, const std::string& annoName);
  ExactAnnoKeySearch(const DB& db, const std::string& annoNamspace, const std::string& annoName);

  virtual ~ExactAnnoKeySearch();

  virtual bool next(Match& result) override;
  virtual void reset() override;

  const std::set<AnnotationKey>& getValidAnnotationKeys() override
  {
    if(!validAnnotationKeysInitialized)
    {
      initializeValidAnnotationKeys();
    }
    return validAnnotationKeys;
  }
  
  virtual std::int64_t guessMaxCount() const override;

  virtual std::string debugString() const override {return debugDescription;}

private:
  const DB& db;

  ItAnnoNode it;
  ItAnnoNode itBegin;
  ItAnnoNode itEnd;

  ItAnnoKey itKeyBegin;
  ItAnnoKey itKeyEnd;

  bool validAnnotationKeysInitialized;
  std::set<AnnotationKey> validAnnotationKeys;

  const std::string debugDescription;
private:
  void initializeValidAnnotationKeys();

};


} // end namespace annis
