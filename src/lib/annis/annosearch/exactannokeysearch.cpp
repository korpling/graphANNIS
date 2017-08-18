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

#include "exactannokeysearch.h"
#include <annis/db.h>                    // for DB
#include <google/btree.h>                // for btree_iterator
#include <boost/container/flat_map.hpp>  // for flat_multimap
#include <boost/container/vector.hpp>    // for operator!=, vec_iterator
#include <cstdint>                       // for uint32_t, int64_t
#include <limits>                        // for numeric_limits
#include <utility>                       // for pair
#include "annis/annostorage.h"           // for AnnoStorage
#include "annis/stringstorage.h"         // for StringStorage
#include <annis/types.h>

using namespace annis;
using namespace std;

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB &db)
  : db(db),
    validAnnotationKeysInitialized(false), debugDescription("node")
{
  itBegin = db.nodeAnnos.inverseAnnotations.begin();
  itEnd = db.nodeAnnos.inverseAnnotations.end();
  it = itBegin;

  itKeyBegin = db.nodeAnnos.annoKeys.begin();
  itKeyBegin = db.nodeAnnos.annoKeys.end();
}

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB& db, const string& annoName)
  : db(db),
    validAnnotationKeysInitialized(false), debugDescription(annoName)
{
  auto searchResult = db.strings.findID(annoName);

  if(searchResult)
  {
    Annotation lowerKey;
    lowerKey.name = *searchResult;
    lowerKey.ns = numeric_limits<uint32_t>::min();
    lowerKey.val = numeric_limits<uint32_t>::min();

    Annotation upperKey;
    upperKey.name = *searchResult;
    upperKey.ns = numeric_limits<uint32_t>::max();
    upperKey.val = numeric_limits<uint32_t>::max();

    itBegin = db.nodeAnnos.inverseAnnotations.lower_bound(lowerKey);
    it = itBegin;
    itEnd = db.nodeAnnos.inverseAnnotations.upper_bound(upperKey);

    itKeyBegin = db.nodeAnnos.annoKeys.lower_bound({*searchResult, 0});
    itKeyEnd = db.nodeAnnos.annoKeys.upper_bound({*searchResult, uintmax});
  }
  else
  {
    itBegin = itEnd = it = db.nodeAnnos.inverseAnnotations.end();
    itKeyBegin = itKeyEnd = db.nodeAnnos.annoKeys.end();
  }
}

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB &db, const string &annoNamspace, const string &annoName)
  : db(db),
    validAnnotationKeysInitialized(false), debugDescription(annoNamspace + ":" + annoName)
{
  auto nameID = db.strings.findID(annoName);
  auto namespaceID = db.strings.findID(annoNamspace);

  if(nameID && namespaceID)
  {
    Annotation lowerKey;
    lowerKey.name = *nameID;
    lowerKey.ns = *namespaceID;
    lowerKey.val = numeric_limits<uint32_t>::min();

    Annotation upperKey;
    upperKey.name = *nameID;
    upperKey.ns = *namespaceID;
    upperKey.val = numeric_limits<uint32_t>::max();

    itBegin = db.nodeAnnos.inverseAnnotations.lower_bound(lowerKey);
    it = itBegin;
    itEnd = db.nodeAnnos.inverseAnnotations.upper_bound(upperKey);

    itKeyBegin = db.nodeAnnos.annoKeys.lower_bound({*nameID, *namespaceID});
    itKeyEnd = db.nodeAnnos.annoKeys.upper_bound({*nameID, *namespaceID});
  }
  else
  {
    itBegin = itEnd = it = db.nodeAnnos.inverseAnnotations.end();
    itKeyBegin = itKeyEnd = db.nodeAnnos.annoKeys.end();
  }
}

bool ExactAnnoKeySearch::next(Match& result)
{
  while(it != db.nodeAnnos.inverseAnnotations.end() && it != itEnd)
  {
    result.node = it->second; // node ID
    result.anno = it->first; // annotation itself
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
  } // end while something found in inverse annotation map

  return false;

}

void ExactAnnoKeySearch::reset()
{
  uniqueResultFilter.clear();
  it = itBegin;
}

void ExactAnnoKeySearch::initializeValidAnnotationKeys()
{
  for(ItAnnoKey itKey = itKeyBegin; itKey != itKeyEnd; itKey++)
  {
    validAnnotationKeys.insert(itKey->first);
  }
  validAnnotationKeysInitialized = true;
}

std::int64_t ExactAnnoKeySearch::guessMaxCount() const
{ 
  std::int64_t sum = 0;
  for(auto itKey = itKeyBegin; itKey != itKeyEnd; itKey++)
  {
    sum += itKey->second;
  }
  return sum;
}


ExactAnnoKeySearch::~ExactAnnoKeySearch()
{

}
