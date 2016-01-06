/* 
 * File:   DBCache.h
 * Author: thomas
 *
 * Created on 5. Januar 2016, 17:17
 */

#ifndef DBCACHE_H
#define DBCACHE_H

#include "types.h"
#include "db.h"
#include <map>
#include <memory>

namespace annis {

  struct DBCacheKey {
    std::string corpus;
    bool forceFallback;
    std::map<Component, std::string> overrideImpl;
  };
}

namespace std {

  template<>
  struct less<annis::DBCacheKey> {

    bool operator()(const struct annis::DBCacheKey &a, const struct annis::DBCacheKey &b) const {
      ANNIS_STRUCT_COMPARE(a.corpus, b.corpus);
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
    DBCache();
    DBCache(const DBCache& orig) = delete;

    DB& get(const std::string& corpus, bool forceFallback = false,
            std::map<Component, std::string> overrideImpl = std::map<Component, std::string>()) {
      DBCacheKey key = {corpus, forceFallback, overrideImpl};
      auto it = cache.find(key);
      if (it == cache.end()) {
        // cleanup the cache
        cleanup();
        // create a new one
        cache[key] = initDB(key);
        return *cache[key];
      }
      return *(it->second);
    }

    void release(const std::string& corpus, bool forceFallback = false,
            std::map<Component, std::string> overrideImpl = std::map<Component, std::string>()) {
      release({corpus, forceFallback, overrideImpl});
    }
    
    void cleanup(std::set<DBCacheKey> ignore = std::set<DBCacheKey>()) {
      bool deletedSomething = true;
      while(deletedSomething && !cache.empty() && loadedDBSizeTotal > maxLoadedDBSize) {
        deletedSomething = false;
        for(auto it=cache.begin(); it != cache.end(); it++) {
          if(ignore.find(it->first) == ignore.end()) {
            release(it->first);
            deletedSomething;
            break;
          }
        }
      }
    }


    virtual ~DBCache();
  private:
    std::map<DBCacheKey, std::unique_ptr<DB>> cache;
    std::map<DBCacheKey, size_t> loadedDBSize;
    size_t loadedDBSizeTotal;
    size_t maxLoadedDBSize;
    
  private:
    
    std::unique_ptr<DB> initDB(const DBCacheKey& key);
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

#endif /* DBCACHE_H */

