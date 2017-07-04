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

#include "regexannosearch.h"

#include <google/btree.h>                // for btree_iterator
#include <google/btree_map.h>            // for btree_map
#include <boost/container/flat_map.hpp>  // for flat_multimap
#include <boost/container/vector.hpp>    // for vec_iterator, operator!=
#include <cstdint>                       // for uint32_t, int64_t
#include "annis/annostorage.h"           // for AnnoStorage
#include "annis/db.h"                    // for DB
#include "annis/stringstorage.h"         // for StringStorage
#include "annis/types.h"                 // for Annotation, AnnotationKey


using namespace annis;

RegexAnnoSearch::RegexAnnoSearch(const DB &db, const std::string& ns,
                                 const std::string& name, const std::string& valRegex)
  : db(db),
    annoKeyNamespace(ns), annoKeyName(name),
    valRegex(valRegex),
    compiledValRegex(valRegex, RE2::Quiet),
    debugDescription(ns + ":" + name + "=/" + valRegex + "/")
{
  std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
  std::pair<bool, std::uint32_t> namespaceID = db.strings.findID(ns);
  if(nameID.first && namespaceID.first)
  {
    if(compiledValRegex.ok())
    {
      annoKeys.insert({nameID.second, namespaceID.second});

      auto lower = db.nodeAnnos.inverseAnnotations.lower_bound({nameID.second, namespaceID.second, 0});
      auto upper = db.nodeAnnos.inverseAnnotations.lower_bound({nameID.second, namespaceID.second, uintmax});
      searchRanges.push_back(Range(lower, upper));
    }
  }

  currentRange = searchRanges.begin();
  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
  }
}

bool RegexAnnoSearch::valueMatchesAllStrings() const
{
  if(compiledValRegex.ok())
  {
    if(compiledValRegex.pattern() == ".*")
    {
      return true;
    }
  }
  return false;
}


RegexAnnoSearch::RegexAnnoSearch(const DB &db,
                                 const std::string& name, const std::string& valRegex)
  : db(db),
    annoKeyName(name),
    valRegex(valRegex),
    compiledValRegex(valRegex, RE2::Quiet),
    debugDescription(name + "=/" + valRegex + "/")
{
  if(compiledValRegex.ok())
  {
    std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
    if(nameID.first)
    {
      auto keysLower = db.nodeAnnos.annoKeys.lower_bound({nameID.second, 0});
      auto keysUpper = db.nodeAnnos.annoKeys.upper_bound({nameID.second, uintmax});
      for(auto itKey = keysLower; itKey != keysUpper; itKey++)
      {
        annoKeys.insert({itKey->first.name, itKey->first.ns});
        
        auto lowerAnno = db.nodeAnnos.inverseAnnotations.lower_bound({itKey->first.name, itKey->first.ns, 0});
        auto upperAnno = db.nodeAnnos.inverseAnnotations.lower_bound({itKey->first.name, itKey->first.ns, uintmax});
        searchRanges.push_back(Range(lowerAnno, upperAnno));
      }
    }
  } // end if the regex is ok
  currentRange = searchRanges.begin();

  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
  }
}

bool RegexAnnoSearch::next(Match& result)
{
  if(compiledValRegex.ok())
  {
    while(currentRange != searchRanges.end())
    {
      while(it != currentRange->second)
      {
        if(RE2::FullMatch(db.strings.str(it->first.val), compiledValRegex))
        {
          result = {it->second, it->first};
          it++;
          if(getConstAnnoValue())
          {
            /*
             * When we replace the resulting annotation with a constant value it is possible that duplicates
             * can occur. Therfore we must check that each node is only included once as a result
             */
            if(uniqueResultFilter.find(result.node) == uniqueResultFilter.end())
            {
              uniqueResultFilter.insert(result.node);

              result.anno = *getConstAnnoValue();

              return true;
            }
          }
          else
          {
            return true;
          }
          return true;
        }
        // skip to the next available key (we don't want to iterate over each value of the multimap)
        it = db.nodeAnnos.inverseAnnotations.upper_bound(it->first);

      } // end for each item in search range
      currentRange++;
      if(currentRange != searchRanges.end())
      {
        it = currentRange->first;
      }
    } // end for each search range
  }
  
  return false;
}

void RegexAnnoSearch::reset()
{
  uniqueResultFilter.clear();

  currentRange = searchRanges.begin();
  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
  }
}


RegexAnnoSearch::~RegexAnnoSearch()
{

}



std::int64_t RegexAnnoSearch::guessMaxCount() const
{
  std::int64_t sum = 0;
  
  for(const AnnotationKey& anno : annoKeys)
  {
    sum += db.nodeAnnos.guessMaxCountRegex(db.strings, db.strings.str(anno.ns), db.strings.str(anno.name), compiledValRegex);
  }
  
  return sum;
}



