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

#include <annis/annostorage.h>  // for AnnoStorage, AnnoStorage<>::InverseAn...
#include <re2/re2.h>            // for RE2
#include <stdint.h>             // for int64_t
#include <list>                 // for list, list<>::const_iterator
#include <set>                  // for set
#include <string>               // for string
#include <unordered_set>        // for unordered_set
#include <utility>              // for pair
#include <vector>               // for vector
#include <annis/types.h>        // for Annotation, AnnotationKey, Match (ptr...
#include <annis/annosearch/estimatedsearch.h>   // for EstimatedSearch
namespace annis { class DB; }



namespace annis
{

  class RegexAnnoSearch : public EstimatedSearch
  {
    using AnnoItType = AnnoStorage<nodeid_t>::InverseAnnoMap_t::const_iterator;
    using Range = std::pair<AnnoItType, AnnoItType>;

  public:
    RegexAnnoSearch(const DB& db, const std::string &name, const std::string &valRegex);
    RegexAnnoSearch(const DB& db, const std::string &ns, const std::string &name, const std::string &valRegex);


    const std::set<AnnotationKey>& getValidAnnotationKeys() const
    {
      return annoKeys;
    }

    bool valueMatches(const std::string& str)
    {
      if(compiledValRegex.ok())
      {
        return RE2::FullMatch(str, compiledValRegex);
      }
      return false;
    }

    bool valueMatchesAllStrings() const;

    boost::optional<std::string> getAnnoKeyNamespace() const
    {
      return annoKeyNamespace;
    }

    std::string getAnnoKeyName() const
    {
      return annoKeyName;
    }

    
    virtual bool next(Match& result) override;
    virtual void reset() override;

    std::int64_t guessMaxCount() const override;

    virtual std::string debugString() const override {return debugDescription;}

    virtual ~RegexAnnoSearch();
  private:
    const DB& db;

    boost::optional<std::string> annoKeyNamespace;
    std::string annoKeyName;

    std::string valRegex;
    RE2 compiledValRegex;
    std::set<AnnotationKey> annoKeys;

    std::list<Range> searchRanges;
    std::list<Range>::const_iterator currentRange;
    AnnoItType it;

    const std::string debugDescription;

    std::unordered_set<nodeid_t> uniqueResultFilter;

  private:
    void initializeValidAnnotationKeys();
    
  };
} // end namespace annis
