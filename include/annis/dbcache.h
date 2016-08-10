#pragma once

#include "types.h"

#include <annis/db.h>

#include <map>
#include <set>
#include <memory>
#include <iostream>

#include <tuple>

#if defined(__linux__) || defined(__linux) || defined(linux) || defined(__gnu_linux__)
  #include <malloc.h>
#endif // LINUX

namespace annis {

  struct DBCacheKey {
    std::string corpusPath;
    bool forceFallback;
    std::map<Component, std::string> overrideImpl;
  };

  inline bool operator<(const struct DBCacheKey &a, const struct DBCacheKey &b)
  {
    return std::tie(a.corpusPath, a.forceFallback, a.overrideImpl) < std::tie(b.corpusPath, b.forceFallback, b.overrideImpl);
  }

  class DB;

  class DBCache {
  public:

    struct CorpusSize
    {
      size_t measured;
      size_t estimated;
    };

  public:
    DBCache(size_t maxSizeBytes=1073741824);
    DBCache(const DBCache& orig) = delete;

    std::weak_ptr<DB> get(const std::string& corpusPath, bool preloadEdges, bool forceFallback = false,
            std::map<Component, std::string> overrideImpl = std::map<Component, std::string>()) {
      DBCacheKey key = {corpusPath, forceFallback, overrideImpl};
      std::map<DBCacheKey, std::shared_ptr<DB>>::iterator it = cache.find(key);
      if (it == cache.end()) {
        // not included yet, we have to load this database

        // make sure we don't exceed the maximal allowed memory size
        cleanup();
        // create a new one
        cache[key] = initDB(key, preloadEdges);
        return cache[key];
      }
      // already in cache
      if(preloadEdges)
      {
        std::shared_ptr<DB>& db = it->second;
        db->ensureAllComponentsLoaded();
      }
      return it->second;
    }

    void release(const std::string& corpusPath, bool forceFallback = false,
            std::map<Component, std::string> overrideImpl = std::map<Component, std::string>()) {
      release({corpusPath, forceFallback, overrideImpl});
    }

    void releaseAll() {
      cache.clear();
      loadedDBSize.clear();

      #if defined(__linux__) || defined(__linux) || defined(linux) || defined(__gnu_linux__)
        // HACK: to make the estimates accurate we have to give back the used memory after each release
        if(malloc_trim(0) != 1)
        {
          std::cerr << "Could not release overhead memory." << std::endl;
        }
      #endif // LINUX
    }
    
    void cleanup(std::set<DBCacheKey> ignore = std::set<DBCacheKey>()) {
      bool deletedSomething = true;

      updateCorpusSizeEstimations();

      while(deletedSomething && !cache.empty() && calculateTotalSize().estimated > maxLoadedDBSize) {
        deletedSomething = false;
        for(auto it=cache.begin(); it != cache.end(); it++) {
          if(ignore.find(it->first) == ignore.end()) {
            release(it->first);
            deletedSomething = true;
            break;
          }
        }
      }
    }

    CorpusSize calculateTotalSize() const;
    const std::map<DBCacheKey, CorpusSize>& estimateCorpusSizes()
    {
      updateCorpusSizeEstimations();
      return loadedDBSize;
    }


    virtual ~DBCache();
  private:
    std::map<DBCacheKey, std::shared_ptr<DB>> cache;
    std::map<DBCacheKey, CorpusSize> loadedDBSize;
    const size_t maxLoadedDBSize;
    
  private:
    
    std::shared_ptr<DB> initDB(const DBCacheKey& key, bool preloadEdges);

    void updateCorpusSizeEstimations();

    void release(DBCacheKey key) {
      cache.erase(key);

      auto itSize = loadedDBSize.find(key);
      if(itSize != loadedDBSize.end()) {
        loadedDBSize.erase(itSize);
      }

      #if defined(__linux__) || defined(__linux) || defined(linux) || defined(__gnu_linux__)
        // HACK: to make the estimates accurate we have to give back the used memory after each release
        if(malloc_trim(0) != 1)
        {
          std::cerr << "Could not release overhead memory." << std::endl;
        }
      #endif // LINUX
    }
  };

} // end namespace annis
