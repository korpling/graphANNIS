#pragma once

#include "types.h"

#include <map>
#include <set>
#include <memory>
#include <iostream>

#if defined(__linux__) || defined(__linux) || defined(linux) || defined(__gnu_linux__)
  #include <malloc.h>
#endif // LINUX

namespace annis {

  struct DBCacheKey {
    std::string corpusPath;
    bool forceFallback;
    std::map<Component, std::string> overrideImpl;
  };
}

namespace std {

  template<>
  struct less<annis::DBCacheKey> {

    bool operator()(const struct annis::DBCacheKey &a, const struct annis::DBCacheKey &b) const {
      ANNIS_STRUCT_COMPARE(a.corpusPath, b.corpusPath);
      ANNIS_STRUCT_COMPARE(a.forceFallback, b.forceFallback);
      const auto& mapA = a.overrideImpl;
      const auto& mapB = b.overrideImpl;
      ANNIS_STRUCT_COMPARE(mapA.size(), mapB.size());
      // more expensive checks using the actual entries
      auto itA = mapA.begin();
      auto itB = mapB.begin();
      while(itA != mapA.end() && itB != mapB.end()) {
        
        // compare key
        std::less<annis::Component> lessCmp;
        const auto& compA = itA->first;
        const auto& compB = itB->first;        
        {if(lessCmp(compA, compB)) {return true;} else if(lessCmp(compB, compA)) {return false;}}
        
        // compare value
        ANNIS_STRUCT_COMPARE(itA->second, itB->second);
        
        itA++;
        itB++;
      }
      
      // they are equal
      return false;
    }
  };
}

namespace annis {

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
      auto it = cache.find(key);
      if (it == cache.end()) {
        // cleanup the cache
        cleanup();
        // create a new one
        cache[key] = initDB(key, preloadEdges);
        return cache[key];
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
    const std::map<DBCacheKey, CorpusSize> estimateCorpusSizes();


    virtual ~DBCache();
  private:
    std::map<DBCacheKey, std::shared_ptr<DB>> cache;
    std::map<DBCacheKey, CorpusSize> loadedDBSize;
    const size_t maxLoadedDBSize;
    
  private:
    
    std::shared_ptr<DB> initDB(const DBCacheKey& key, bool preloadEdges);

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
