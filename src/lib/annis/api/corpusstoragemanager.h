/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#pragma once

#include <annis/api/graphupdate.h>
#include <annis/api/graph.h>

#include <stddef.h>                        // for size_t
#include <map>                             // for map
#include <memory>                          // for shared_ptr
#include <mutex>                           // for mutex
#include <string>                          // for string
#include <vector>                          // for vector


namespace annis { class DBLoader; }
namespace annis { namespace api { class GraphUpdate; } }

namespace boost { class thread;}


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

  struct CorpusInfo
  {
    std::string loadStatus;
    long long memoryUsageInBytes;
  };

  CorpusStorageManager(std::string databaseDir, size_t maxAllowedCacheSize = 1073741824);
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
   * @brief Return a sub-graph consisting of the nodes given as argument.s
   * @param corpus
   * @param nodeIDs The IDs/names of the nodes to include.
   * @return
   */
  std::vector<Node> subgraph(std::string corpus, std::vector<std::string> &nodeIDs);

  /**
   * @brief Lists the name of all corpora.
   * @return
   */
  std::vector<std::string> list();

  void importCorpus(std::string pathToCorpus, std::string newCorpusName);
  void exportCorpus(std::string corpusName, std::string exportPath);

  void importRelANNIS(std::string pathToCorpus, std::string newCorpusName);

  bool deleteCorpus(std::string corpusName);

  CorpusInfo info(std::string corpusName);


private:
  const std::string databaseDir;
  const size_t maxAllowedCacheSize;

  std::mutex mutex_corpusCache;
  std::map<std::string, std::shared_ptr<DBLoader>> corpusCache;

  std::mutex mutex_writerThreads;
  std::map<std::string, boost::thread> writerThreads;

private:


  /**
   * @brief Writes a new version of the corpus in the background to the disk,
   * This will start a background thread which is stored in the writerThreads map.
   * Before any update can occur, the writing thread has to be killBackgroundWriter().
   * @param corpusPath
   */
  void startBackgroundWriter(std::string corpusPath, std::shared_ptr<DBLoader> &loader);
  /**
   * @brief Stops a background writer for a corpus. Will return as the thread is successfully stopped.
   * @param corpusPath
   */
  void killBackgroundWriter(std::string corpus);

  std::shared_ptr<DBLoader> getCorpusFromCache(std::string name);

};

}} // end namespace annis
