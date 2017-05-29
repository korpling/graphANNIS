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

#include "stringstorage.h"
#include <annis/util/size_estimator.h>  // for element_size
#include <re2/re2.h>                    // for RE2, RE2::CannedOptions::Quiet
#include <re2/stringpiece.h>            // for StringPiece
#include <iterator>                     // for reverse_iterator

using namespace annis;
using namespace std;

StringStorage::StringStorage()
{
}

std::unordered_set<std::uint32_t> StringStorage::findRegex(const string &str) const
{
  using ItType = btree::btree_map<string, uint32_t>::const_iterator;
  std::unordered_set<std::uint32_t> result;

  RE2 re(str, RE2::Quiet);
  if(re.ok())
  {
    // get the size of the last element so we know how large our prefix needs to be
    size_t prefixSize = 10;
    const std::string& lastString = stringStorageByValue.rbegin()->first;
    size_t lastStringSize = lastString.size()+1;
    if(lastStringSize > prefixSize)
    {
      prefixSize = lastStringSize;
    }

    std::string minPrefix;
    std::string maxPrefix;
    re.PossibleMatchRange(&minPrefix, &maxPrefix, prefixSize);

    ItType upperBound = stringStorageByValue.upper_bound(maxPrefix);

    for(ItType it=stringStorageByValue.lower_bound(minPrefix);
        it != upperBound; it++)
    {
      if(RE2::FullMatch(it->first, re))
      {
        result.insert(it->second);
      }
    }
  }

  return std::move(result);
}

uint32_t StringStorage::add(const string &str)
{
  typedef btree::btree_map<string, uint32_t>::const_iterator ItType;
  ItType it = stringStorageByValue.find(str);
  if(it == stringStorageByValue.end())
  {
    // non-existing
    uint32_t id = stringStorageByID.size() + 1; // since 0 is taken as ANY value begin with 1
    // make sure the ID is really not taken yet
    while(stringStorageByID.find(id) != stringStorageByID.end())
    {
      id++;
    }
    stringStorageByID.insert(pair<uint32_t, string>(id, str));
    stringStorageByValue.insert(pair<string, uint32_t>(str, id));
    return id;
  }
  else
  {
    // already existing, return the original ID
    return it->second;
  }
}

void StringStorage::clear()
{

  stringStorageByID.clear();
  stringStorageByValue.clear();

}


double annis::StringStorage::avgLength()
{
  size_t sum=0;
  for(const auto& v : stringStorageByValue)
  {
    sum += v.first.size();
  }
  return (double) sum / (double) stringStorageByValue.size();
}

size_t StringStorage::estimateMemorySize() const
{
  size_t strSize = 0;
  for(const auto& v : stringStorageByValue)
  {
    const std::string& s = v.first;
    strSize += s.capacity();
  }
  return
      size_estimation::element_size(stringStorageByID)
      + size_estimation::element_size(stringStorageByValue)
      + (strSize*2);
}
