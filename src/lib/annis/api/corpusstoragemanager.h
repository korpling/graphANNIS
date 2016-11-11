#pragma once

#include <memory>
#include <vector>
#include <list>

#include <mutex>
#include <boost/thread.hpp>

#include <annis/db.h>
#include <annis/dbcache.h>
#include <annis/json/jsonqueryparser.h>

#include <annis/api/graphupdate.h>

namespace annis
{
namespace api
{
/**
 * An API for managing corpora stored in a common location on the file system.
 */
class CorpusStorageManager
{
public:

  struct CountResult
  {
    long long matchCount;
    long long documentCount;
  };

  CorpusStorageManager(std::string databaseDir);
   ~CorpusStorageManager();

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
   * Count all occurrences of an AQL query in a single corpus.
   *
   * @param corpus
   * @param queryAsJSON
   * @return
   */
  CountResult countExtra(std::vector<std::string> corpora,
                  std::string queryAsJSON);


  /**
   * Find occurrences of an AQL query in a single corpus.
   * @param corpora
   * @param queryAsJSON
   * @param offset
   * @param limit
   * @return
   */
  std::vector<std::string> find(std::vector< std::string > corpora, std::string queryAsJSON, long long offset=0,
                                long long limit=0);

  void applyUpdate(std::string corpus, GraphUpdate &update);

  /**
   * @brief Lists the name of all corpora.
   * @return
   */
  std::vector<std::string> list();

  void importCorpus(std::string pathToCorpus, std::string newCorpusName);
  void exportCorpus(std::string corpusName, std::string exportPath);

  bool deleteCorpus(std::string corpusName);

private:
  const std::string databaseDir;
  std::unique_ptr<DBCache> cache;

  std::mutex mutex_writerThreads;
  std::map<std::string, boost::thread> writerThreads;
private:


  /**
   * @brief Writes a new version of the corpus in the background to the disk,
   * This will start a background thread which is stored in the writerThreads map.
   * Before any update can occur, the writing thread has to be killBackgroundWriter().
   * @param corpusPath
   */
  void startBackgroundWriter(std::string corpusPath, std::shared_ptr<DB> db);
  /**
   * @brief Stops a background writer for a corpus. Will return as the thread is successfully stopped.
   * @param corpusPath
   */
  void killBackgroundWriter(std::string corpusPath);
};

}} // end namespace annis
