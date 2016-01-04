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

void DynamicCorpusFixture::UserBenchmark() {
  counter = 0;
  
  while (q->hasNext()) {
    q->next();
    counter++;
  }
  HL_INFO(logger, (boost::format("result %1%") % counter).str());
  if (expectedCount && counter != *expectedCount) {
    std::cerr << "FATAL ERROR: query " << benchmarkName << " should have count " << *expectedCount << " but was " << counter << std::endl;
    std::cerr << "" << __FILE__ << ":" << __LINE__ << std::endl;
    exit(-1);
  }
}

void DynamicCorpusFixture::tearDown() {
  
}

DynamicBenchmark::DynamicBenchmark(std::string corpusName)
:  corpus(corpusName) {

}

void DynamicBenchmark::registerDefaultBenchmarks(std::string queriesDir) {
  registerFixtureInternal(true, queriesDir, "Fallback", true);
  registerFixtureInternal(false, queriesDir, "Optimized", false);
}

void DynamicBenchmark::registerFixture(std::string queriesDir, std::string fixtureName, 
        bool forceFallback, std::map<Component, std::string> overrideImpl) {
  registerFixtureInternal(false, queriesDir, fixtureName, forceFallback, overrideImpl);
}


void DynamicBenchmark::registerFixtureInternal(
        bool baseline, std::string queriesDir, 
        std::string fixtureName, bool forceFallback, 
        std::map<Component, std::string> overrideImpl) {
  
   // find all file ending with ".json" in the folder
  boost::filesystem::directory_iterator fileEndIt;

  boost::filesystem::directory_iterator itFiles(queriesDir);
  while (itFiles != fileEndIt) {
    const auto filePath = itFiles->path();
    if (filePath.extension().string() == ".json") {
      addBenchmark(baseline, filePath, fixtureName, forceFallback);
    }
    itFiles++;
  }
}


void DynamicBenchmark::addBenchmark(
        bool baseline,
        const boost::filesystem::path& path,
        std::string fixtureName, bool forceFallback) {

  HL_INFO(logger, (boost::format("adding benchmark %1%") % path.string()).str());
  
  

  // check if we need to load the databases
  if (dbByFixture.find(fixtureName) == dbByFixture.end()) {
    dbByFixture[fixtureName] = initDB(forceFallback);
  }

  std::string benchmarkName = path.filename().stem().string() + "_" + corpus;

  boost::optional<unsigned int> expectedCount;
  auto countPath = path.parent_path() /= (path.stem().string() + ".count");

  boost::filesystem::ifstream stream;

  stream.open(countPath);
  if (stream.is_open()) {
    unsigned int tmp;
    stream >> tmp;
    stream.close();
    expectedCount = tmp;
  }

  stream.open(path);
  std::string queryJSON(
    (std::istreambuf_iterator<char>(stream)),
    (std::istreambuf_iterator<char>()));  
  stream.close();

  if(baseline) {
    celero::RegisterBaseline(benchmarkName.c_str(), fixtureName.c_str(), 5, 5, 1,
            std::make_shared<DynamicCorpusFixtureFactory> (queryJSON,
            benchmarkName + " (" + fixtureName + ")",
            *(dbByFixture[fixtureName]),
            expectedCount));
  } else {
    celero::RegisterTest(benchmarkName.c_str(), fixtureName.c_str(), 5, 5, 1,
            std::make_shared<DynamicCorpusFixtureFactory> (queryJSON,
            benchmarkName + " (" + fixtureName + ")",
            *(dbByFixture[fixtureName]),
            expectedCount));
  }
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
