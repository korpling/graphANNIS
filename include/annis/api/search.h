#pragma once

#include <memory>
#include <vector>
#include <list>

#include <annis/db.h>
#include <annis/DBCache.h>
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

  Search();
   ~Search();

  /**
   * Count all occurences of an AQL query in a single corpus.
   *
   * @param corpus
   * @param queryAsJSON
   * @return
   */
  long long count(std::vector<std::string> corpora,
                  std::string queryAsJSON);

  std::vector<std::string> find(std::vector< std::string > corpora, std::string queryAsJSON);

private:
  std::string databaseDir;
  std::unique_ptr<DBCache> cache;
};

}} // end namespace annis
