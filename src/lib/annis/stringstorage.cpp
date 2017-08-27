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


#include <graphannis-capi.h>

annis::StringStorage::StringStorage()
  :impl(NULL)
{
  impl = annis_stringstorage_new();
}

annis::StringStorage::~StringStorage()
{
  annis_stringstorage_free(impl);
}

const std::string annis::StringStorage::str(uint32_t id) const
{
  annis_Option_String result = annis_stringstorage_str(impl, id);
  if(result.valid)
  {
    return std::string(result.value.s, result.value.length);
  }
  else
  {
    throw("Unknown string ID");
  }
}

boost::optional<std::string> annis::StringStorage::strOpt(uint32_t id) const
{
  annis_Option_String result = annis_stringstorage_str(impl, id);
  if(result.valid)
  {
    return std::string(result.value.s, result.value.length);
  }
  else
  {
    return boost::optional<std::string>();
  }
}

boost::optional<uint32_t> annis::StringStorage::findID(const std::string &str) const
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

uint32_t annis::StringStorage::add(const std::string &str)
{
  return annis_stringstorage_add(impl, str.c_str());
}

void annis::StringStorage::clear()
{
  annis_stringstorage_clear(impl);
}

size_t annis::StringStorage::size() const
{
  return annis_stringstorage_len(impl);
}

double annis::StringStorage::avgLength()
{
  return annis_stringstorage_avg_length(impl);
}

size_t annis::StringStorage::estimateMemorySize() const
{
  return annis_stringstorage_estimate_memory(impl);
}

void annis::StringStorage::loadFromFile(const std::string &path)
{
  annis_stringstorage_load_from_file(impl, path.c_str());
}

void annis::StringStorage::saveToFile(const std::string &path)
{
  annis_stringstorage_save_to_file(impl, path.c_str());
}
