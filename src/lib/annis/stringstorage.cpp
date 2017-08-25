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
  : impl(NULL)
{
  impl = annis_stringstorage_new();

}

StringStorage::~StringStorage()
{
  annis_stringstorage_free(impl);
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
    const std::string& lastString = byValue.rbegin()->first;
    size_t lastStringSize = lastString.size()+1;
    if(lastStringSize > prefixSize)
    {
      prefixSize = lastStringSize;
    }

    std::string minPrefix;
    std::string maxPrefix;
    re.PossibleMatchRange(&minPrefix, &maxPrefix, prefixSize);

    ItType upperBound = byValue.upper_bound(maxPrefix);

    for(ItType it=byValue.lower_bound(minPrefix);
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


double annis::StringStorage::avgLength()
{
  size_t sum=0;
  for(const auto& v : byValue)
  {
    sum += v.first.size();
  }
  return (double) sum / (double) byValue.size();
}

size_t StringStorage::estimateMemorySize() const
{
  size_t strSize = 0;
  for(const auto& v : byValue)
  {
    const std::string& s = v.first;
    strSize += s.capacity();
  }
  return
      size_estimation::element_size(byID)
      + size_estimation::element_size(byValue)
      + (strSize*2);
}
