#pragma once

#include "types.h"
#include "db.h"
#include <map>
#include <memory>

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

  class DBCache {
  public:
    DBCache(size_t maxSizeBytes=1073741824);
    DBCache(const DBCache& orig) = delete;

    std::weak_ptr<DB> get(const std::string& corpusPath, bool forceFallback = false,
            std::map<Component, std::string> overrideImpl = std::map<Component, std::string>()) {
      DBCacheKey key = {corpusPath, forceFallback, overrideImpl};
      auto it = cache.find(key);
      if (it == cache.end()) {
        // cleanup the cache
        cleanup();
        // create a new one
        cache[key] = initDB(key);
        return cache[key];
      }
      return it->second;
    }

    void release(const std::string& corpusPath, bool forceFallback = false,
            std::map<Component, std::string> overrideImpl = std::map<Component, std::string>()) {
      release({corpusPath, forceFallback, overrideImpl});
    }
    
    void cleanup(std::set<DBCacheKey> ignore = std::set<DBCacheKey>()) {
      bool deletedSomething = true;
      while(deletedSomething && !cache.empty() && loadedDBSizeTotal > maxLoadedDBSize) {
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

    size_t size() { return loadedDBSizeTotal;}
    const std::map<DBCacheKey, size_t>& corpusSizes() const { return loadedDBSize;}


    virtual ~DBCache();
  private:
    std::map<DBCacheKey, std::shared_ptr<DB>> cache;
    std::map<DBCacheKey, size_t> loadedDBSize;
    size_t loadedDBSizeTotal;
    const size_t maxLoadedDBSize;
    
  private:
    
    std::shared_ptr<DB> initDB(const DBCacheKey& key);

    void release(DBCacheKey key) {
      cache.erase(key);
      auto itSize = loadedDBSize.find(key);
      if(itSize != loadedDBSize.end()) {
        size_t oldSize = itSize->second;
        loadedDBSize.erase(itSize);
        loadedDBSizeTotal -= oldSize;
      }
    }
  };

} // end namespace annis
