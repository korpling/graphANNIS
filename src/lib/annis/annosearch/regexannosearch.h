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
#include <annis/annosearch/annotationsearch.h>   // for AnnotationSearch
namespace annis { class DB; }



namespace annis
{

  class RegexAnnoSearch : public AnnotationSearch
  {
    using AnnoItType = AnnoStorage<nodeid_t>::InverseAnnoMap_t::const_iterator;
    using Range = std::pair<AnnoItType, AnnoItType>;

  public:
    RegexAnnoSearch(const DB& db, const std::string &name, const std::string &valRegex);
    RegexAnnoSearch(const DB& db, const std::string &ns, const std::string &name, const std::string &valRegex);

    virtual const std::unordered_set<Annotation>& getValidAnnotations() override
    {
      if (!validAnnotationsInitialized)
      {
        initValidAnnotations();
      }
      return validAnnotations;
    }

    virtual const std::set<AnnotationKey>& getValidAnnotationKeys()
    {
      return validAnnotationKeys;
    }
    
    virtual bool next(Match& result) override;
    virtual void reset() override;

    std::int64_t guessMaxCount() const override;

    virtual std::string debugString() const override {return debugDescription;}

    virtual ~RegexAnnoSearch();
  private:
    const DB& db;
    std::unordered_set<Annotation> validAnnotations;
    bool validAnnotationsInitialized;

    // always empty
    std::set<AnnotationKey> validAnnotationKeys;

    std::string valRegex;
    RE2 compiledValRegex;
    std::vector<Annotation> annoTemplates;

    std::list<Range> searchRanges;
    std::list<Range>::const_iterator currentRange;
    AnnoItType it;

    const std::string debugDescription;

  private:
    
    void initValidAnnotations();
    
  };
} // end namespace annis
