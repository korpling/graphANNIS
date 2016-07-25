#pragma once

#include <memory>
#include <vector>
#include <list>

#include <annis/db.h>
#include <annis/DBCache.h>
#include <annis/json/jsonqueryparser.h>


namespace annis
{
class API
{
public:

  typedef std::vector<std::string> StringVector;

  API()
    : databaseDir("/tmp/graphANNIS")
  {
    cache = std::unique_ptr<DBCache>(new DBCache());
  }
   ~API() {}

  /**
   * Count all occurences of an AQL query in a single corpus.
   *
   * @param corpus
   * @param queryAsJSON
   * @return
   */
  long long count(std::string corpus,
                  std::string queryAsJSON)
  {
    long long result = 0;


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
  std::string databaseDir;
  std::unique_ptr<DBCache> cache;
};

} // end namespace annis
