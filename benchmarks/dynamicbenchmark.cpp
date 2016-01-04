/* 
 * File:   DynamicBenchmark.cpp
 * Author: thomas
 * 
 * Created on 4. Januar 2016, 11:54
 */

#include "dynamicbenchmark.h"


#include <humblelogging/api.h>
#include <boost/filesystem.hpp>
#include <boost/format.hpp>

using namespace annis;


DynamicBenchmark::DynamicBenchmark(std::string dataDir, std::string queriesDir, std::string corpusName)
: dataDir(dataDir), queriesDir(queriesDir), corpus(corpusName) {

  // find all file ending with ".json" in the folder
  boost::filesystem::directory_iterator fileEndIt;

  boost::filesystem::directory_iterator itFiles(queriesDir);
  while (itFiles != fileEndIt) {
    const auto filePath = itFiles->path();
    if (filePath.extension().string() == ".json") {
      addBenchmark(filePath);
    }
    itFiles++;
  }
}

void DynamicBenchmark::addBenchmark(const boost::filesystem::path& path) {
  HL_INFO(logger, (boost::format("adding benchmark %1%") % path.string()).str());
  // create fallback fixture
}