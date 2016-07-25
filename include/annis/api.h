#pragma once

#include <memory>

#include <annis/db.h>
#include <annis/DBCache.h>
#include <annis/json/jsonqueryparser.h>


namespace annis
{
class API
{
public:
  API(const std::string databaseDir)
    : databaseDir(databaseDir)
  {
    cache = std::unique_ptr<DBCache>(new DBCache());
  }
   ~API() {}

  /**
   * @brief Count all occurences of an AQL query in a single corpus
   *
   * @param corpus
   * @param queryAsJSON
   * @return
   */
  std::int64_t count(std::string corpus, std::string queryAsJSON)
  {
    std::int64_t result = 0;

    std::weak_ptr<DB> dbWeakPtr = cache->get(databaseDir + "/" + corpus);
    if(std::shared_ptr<DB> db = dbWeakPtr.lock())
    {
      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, ss);
      while(q->next())
      {
        result++;
      }
    }

    return result;
  }

private:
  const std::string databaseDir;
  std::unique_ptr<DBCache> cache;
};

} // end namespace annis
