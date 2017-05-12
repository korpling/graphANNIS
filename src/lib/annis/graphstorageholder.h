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

#include <annis/graphstorageregistry.h>  // for GraphStorageRegistry
#include <stddef.h>                      // for size_t
#include <map>                           // for map, map<>::const_iterator
#include <memory>                        // for shared_ptr
#include <string>                        // for string
#include <vector>                        // for vector
#include "annis/types.h"                 // for ComponentType, Component
namespace annis { class ReadableGraphStorage; }
namespace annis { class StringStorage; }
namespace annis { class WriteableGraphStorage; }

namespace annis
{

class GraphStorageHolder
{
  using GraphStorageIt = std::map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator;

public:

  GraphStorageHolder(StringStorage& strings);
  virtual ~GraphStorageHolder();

  std::shared_ptr<const ReadableGraphStorage> getGraphStorage(const Component& component);

  std::shared_ptr<const ReadableGraphStorage> getGraphStorage(ComponentType type, const std::string& layer, const std::string& name);
  std::vector<std::shared_ptr<const ReadableGraphStorage>> getGraphStorage(ComponentType type, const std::string& name);
  std::vector<std::shared_ptr<const ReadableGraphStorage>> getGraphStorage(ComponentType type);

  std::shared_ptr<annis::WriteableGraphStorage> createWritableGraphStorage(ComponentType ctype, const std::string& layer,
                       const std::string& name);

  size_t estimateMemorySize() const;
  std::string info();

  bool allComponentsLoaded()
  {
    return notLoadedLocations.empty();
  }

private:
  friend class DB;

  bool load(std::string dirPath, bool preloadComponents);
  bool save(const std::string &dirPath);
  void clear();

  bool ensureComponentIsLoaded(const Component& c);

  std::string debugComponentString(const Component& c);


private:

  StringStorage& strings;

  /**
   * Map containing all available graph storages.
   */
  std::map<Component, std::shared_ptr<ReadableGraphStorage>> container;
  /**
   * A map from not yet loaded components to it's location on disk.
   */
  std::map<Component, std::string> notLoadedLocations;
  GraphStorageRegistry registry;


};

} // end namespace annis
