#pragma once

#include <memory>
#include <vector>
#include <list>

#include <annis/db.h>
#include <annis/dbcache.h>
#include <annis/json/jsonqueryparser.h>


namespace annis
{
namespace api
{
/**
 * An API for searching in a corpus.
 */
class Search
{
public:

  typedef std::vector<std::string> StringVector;

  Search(std::string databaseDir);
   ~Search();

  /**
   * Count all occurrences of an AQL query in a single corpus.
   *
   * @param corpus
   * @param queryAsJSON
   * @return
   */
  long long count(std::vector<std::string> corpora,
                  std::string queryAsJSON);

  /**
   * Find occurrences of an AQL query in a single corpus.
   * @param corpora
   * @param queryAsJSON
   * @param offset
   * @param limit
   * @return
   */
  std::vector<std::string> find(std::vector< std::string > corpora, std::string queryAsJSON, long long offset=0, long long limit=10);

private:
  const std::string databaseDir;
  std::unique_ptr<DBCache> cache;
};

}} // end namespace annis
