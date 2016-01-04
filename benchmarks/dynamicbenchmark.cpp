/* 
 * File:   DynamicBenchmark.cpp
 * Author: thomas
 * 
 * Created on 4. Januar 2016, 11:54
 */

#include "dynamicbenchmark.h"
#include "jsonqueryparser.h"

#include <humblelogging/api.h>
#include <boost/filesystem.hpp>
#include <boost/filesystem/fstream.hpp>

using namespace annis;

void DynamicCorpusFixture::tearDown() {
  HL_INFO(logger, (boost::format("result %1%") % counter).str());
  if (expectedCount && counter != *expectedCount) {
    std::cerr << "FATAL ERROR: query " << benchmarkName << " should have count " << *expectedCount << " but was " << counter << std::endl;
    std::cerr << "" << __FILE__ << ":" << __LINE__ << std::endl;
    exit(-1);
  }
}


DynamicBenchmark::DynamicBenchmark(std::string queriesDir, std::string corpusName)
: queriesDir(queriesDir), corpus(corpusName) {

}

void DynamicBenchmark::registerBenchmarks() {
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

  // check if we need to load the databases
  if (!fallbackDB) {
    fallbackDB = initDB(true);
  }
  if (!optimizedDB) {
    optimizedDB = initDB(false);
  }

  std::string benchmarkName = path.filename().stem().string() + "_" + corpus;

  boost::filesystem::ifstream jsonInput;
  jsonInput.open(path);
  std::shared_ptr<Query> queryFallback = JSONQueryParser::parse(*fallbackDB, jsonInput);
  jsonInput.close();
  
  jsonInput.open(path);
  std::shared_ptr<Query> queryOptimized = JSONQueryParser::parse(*optimizedDB
          , jsonInput);
  jsonInput.close();
  
  // register both a fallback and an optimized benchmark
  celero::RegisterBaseline(benchmarkName.c_str(), "Fallback", 5, 5, 1,
          std::make_shared<DynamicCorpusFixtureFactory> (queryFallback, benchmarkName, *fallbackDB));
  celero::RegisterTest(benchmarkName.c_str(), "Optimized", 5, 5, 1,
          std::make_shared<DynamicCorpusFixtureFactory> (queryOptimized, benchmarkName, *optimizedDB));
}

std::unique_ptr<DB> DynamicBenchmark::initDB(bool forceFallback) {
  //std::cerr << "INIT DB " << corpus << " in " << (forceFallback ? "fallback" : "default") << " mode" << std::endl;
  std::unique_ptr<DB> result = std::unique_ptr<DB>(new DB());

  char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
  std::string dataDir("data");
  if (testDataEnv != NULL) {
    dataDir = testDataEnv;
  }
  result->load(dataDir + "/" + corpus);

  if (forceFallback) {
    // manually convert all components to fallback implementation
    auto components = result->getAllComponents();
    for (auto c : components) {
      result->convertComponent(c, GraphStorageRegistry::fallback);
    }
  } else {
    result->optimizeAll(overrideImpl);
  }

  return result;
}