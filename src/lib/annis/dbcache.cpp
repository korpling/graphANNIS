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

#include <annis/dbcache.h>
#include <annis/db.h>

using namespace annis;

extern size_t getCurrentRSS( );
extern size_t getCurrentVirtualMemory( );

DBCache::DBCache(size_t maxSizeBytes)
  : maxLoadedDBSize(maxSizeBytes) {
}

DBCache::CorpusSize DBCache::calculateTotalSize()
{
  CorpusSize total = {0,0};
  for(const std::pair<DBCacheKey, CorpusSize>& c : loadedDBSize)
  {
    total.estimated += c.second.estimated;
    total.measured += c.second.measured;
  }
  return total;
}

std::shared_ptr<DB> DBCache::initDB(const DBCacheKey& key, bool preloadEdges) {
  std::shared_ptr<DB> result = std::make_shared<DB>();

  auto oldProcessMemory = getCurrentRSS();
  bool loaded = result->load(key.corpusPath, preloadEdges);
  if (!loaded) {
    std::cerr << "FATAL ERROR: coult not load corpus from " << key.corpusPath << std::endl;
    std::cerr << "" << __FILE__ << ":" << __LINE__ << std::endl;
    exit(-1);
  }

  if (key.forceFallback) {
    // manually convert all components to fallback implementation
    auto components = result->getAllComponents();
    for (auto c : components) {
      result->convertComponent(c, GraphStorageRegistry::fallback);
    }
  }

  auto newProcessMemory = getCurrentRSS();

  size_t measuredSize = 1L;
  if(newProcessMemory >  oldProcessMemory)
  {
    measuredSize = newProcessMemory - oldProcessMemory;
  }

  size_t estimatedSize = result->estimateMemorySize();
  loadedDBSize[key] = {measuredSize, estimatedSize};

  return result;
}

void DBCache::updateCorpusSizeEstimations()
{
  for(auto itLoaded = loadedDBSize.begin(); itLoaded != loadedDBSize.end(); itLoaded++)
  {
    CorpusSize& c = itLoaded->second;
    std::map<DBCacheKey, std::shared_ptr<DB>>::const_iterator itCache = cache.find(itLoaded->first);
    if(itCache != cache.end())
    {
      // corpus is also contained in the cache
      c.estimated = itCache->second->estimateMemorySize();
    }
  }
}

DBCache::~DBCache() {
}

