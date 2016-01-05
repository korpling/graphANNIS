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
        // create a new one
        cache[key] = initDB(key);
        return *cache[key];
      }
      return *(it->second);
    }

    void release(const std::string& corpus, bool forceFallback = false,
            std::map<Component, std::string> overrideImpl = std::map<Component, std::string>()) {
      cache.erase({corpus, forceFallback, overrideImpl});
    }

    std::unique_ptr<DB> initDB(const DBCacheKey& key) {
      //std::cerr << "INIT DB " << corpus << " in " << (forceFallback ? "fallback" : "default") << " mode" <<  std::endl;
      std::unique_ptr<DB> result = std::unique_ptr<DB>(new DB());

      char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
      std::string dataDir("data");
      if (testDataEnv != NULL) {
        dataDir = testDataEnv;
      }
      bool loaded = result->load(dataDir + "/" + key.corpus);
      if (!loaded) {
        std::cerr << "FATAL ERROR: no load corpus " << key.corpus << std::endl;
        std::cerr << "" << __FILE__ << ":" << __LINE__ << std::endl;
        exit(-1);
      }

      if (key.forceFallback) {
        // manually convert all components to fallback implementation
        auto components = result->getAllComponents();
        for (auto c : components) {
          result->convertComponent(c, GraphStorageRegistry::fallback);
        }
      } else {
        //        result->optimizeAll(key.overrideImpl);
      }

      return result;
    }

    virtual ~DBCache();
  private:
    std::map<DBCacheKey, std::unique_ptr<DB>> cache;
  };

} // end namespace annis

#endif /* DBCACHE_H */

