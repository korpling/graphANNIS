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
  StringStorage()
    :impl(NULL)
  {
    impl = annis_stringstorage_new();
  }
  ~StringStorage()
  {
    annis_stringstorage_free(impl);
  }

  const std::string str(std::uint32_t id) const
  {
    annis_OptionalString result = annis_stringstorage_str(impl, id);
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
    annis_OptionalString result = annis_stringstorage_str(impl, id);
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
    annis_Option_u32 result = annis_stringstorage_find_id(impl, str.c_str());
    if(result.valid)
    {
      return result.value;
    }
    else
    {
      return boost::optional<std::uint32_t>();
    }
  }

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

  double avgLength()
  {
    return annis_stringstorage_avg_length(impl);
  }

  size_t estimateMemorySize() const
  {
    return annis_stringstorage_estimate_memory(impl);
  }

  void loadFromFile(const std::string& path)
  {
    annis_stringstorage_load_from_file(impl, path.c_str());
  }

  void saveToFile(const std::string& path)
  {
    annis_stringstorage_save_to_file(impl, path.c_str());
  }

private:
  annis_StringStoragePtr* impl;

};
}

