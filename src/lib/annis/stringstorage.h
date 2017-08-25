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

#include <cereal/cereal.hpp>
#include <cereal/types/string.hpp>
#include <cereal/types/unordered_map.hpp>
#include <annis/serializers.h>

#include <google/btree.h>               // for btree_iterator
#include <google/btree_container.h>     // for btree_unique_container<>::con...
#include <google/btree_map.h>           // for btree_map
#include <stddef.h>                     // for size_t
#include <boost/optional/optional.hpp>  // for optional
#include <cstdint>                      // for uint32_t
#include <set>                          // for set
#include <string>                       // for string
#include <unordered_map>                // for unordered_map, _Node_const_it...
#include <utility>                      // for pair

#include <graphannis-api.h>


namespace annis
{
const std::uint32_t STRING_STORAGE_ANY = 0;

class StringStorage
{
public:
  StringStorage();
  ~StringStorage();

  const std::string str(std::uint32_t id) const
  {
    OptionalString result = annis_stringstorage_str(impl, id);
    if(result.valid)
    {
      return std::string(result.value, result.length);
    }
    else
    {
      throw("Unknown string ID");
    }
  }

  boost::optional<std::string> strOpt(std::uint32_t id) const
  {
    OptionalString result = annis_stringstorage_str(impl, id);
    if(result.valid)
    {
      return std::string(result.value, result.length);
    }
    else
    {
      return boost::optional<std::string>();
    }
  }

  boost::optional<std::uint32_t> findID(const std::string& str) const
  {
    typedef btree::btree_map<std::string, std::uint32_t>::const_iterator ItType;
    boost::optional<std::uint32_t> result;
    ItType it = byValue.find(str);
    if(it != byValue.end())
    {
      result = it->second;
    }
    return result;
  }

  std::unordered_set<std::uint32_t> findRegex(const std::string& str) const;

  std::uint32_t add(const std::string& str)
  {
    return annis_stringstorage_add(impl, str.c_str());
  }

  void clear()
  {
    annis_stringstorage_clear(impl);
  }

  size_t size() const
  {
    return annis_stringstorage_len(impl);
  }

  double avgLength();

  size_t estimateMemorySize() const;

  template<class Archive>
  void serialize(Archive & archive)
  {
    archive(byID, byValue);
  }

private:
  StringStoragePtr* impl;
  std::unordered_map<std::uint32_t, std::string> byID;
  btree::btree_map<std::string, std::uint32_t> byValue;

};
}

