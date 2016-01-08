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

std::shared_ptr<DBCache> DynamicCorpusFixture::dbCache 
        = std::make_shared<DBCache>();

void DynamicCorpusFixture::UserBenchmark() {
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

std::vector<std::pair<int64_t, uint64_t> > DynamicCorpusFixture::getExperimentValues() {
  std::vector<std::pair<int64_t, uint64_t> > result;
  
  for(auto it : json)
  {
    result.push_back({it.first, 0});
  }
  
  return result;
}


void DynamicCorpusFixture::tearDown() {
  executionCounter++;
  if(executionCounter >= numberOfSamples) {
    // delete the database after all runs are complete
    dbCache->release(corpus, forceFallback, overrideImpl);
  }
}

DynamicBenchmark::DynamicBenchmark(std::string queriesDir, 
        std::string corpusName, bool registerOptimized)
:  corpus(corpusName) {
  // find all file ending with ".json" in the folder
  boost::filesystem::directory_iterator fileEndIt;

  boost::filesystem::directory_iterator itFiles(queriesDir);
  while (itFiles != fileEndIt) {
    const auto filePath = itFiles->path();
    if (filePath.extension().string() == ".json") {
      foundJSONFiles.push_back(filePath);
    }
    itFiles++;
  }
  
  registerFixtureInternal(true, "Fallback", true);
  if(registerOptimized) {
    registerFixtureInternal(false, "Optimized", false);
  }
}

void DynamicBenchmark::registerFixture(std::string fixtureName,
        std::map<Component, std::string> overrideImpl) {
  registerFixtureInternal(false, fixtureName, false, overrideImpl);
}


void DynamicBenchmark::registerFixtureInternal(
        bool baseline, 
        std::string fixtureName, bool forceFallback, 
        std::map<Component, std::string> overrideImpl) {

  for(const auto& filePath : foundJSONFiles) {
    addBenchmark(baseline, filePath, fixtureName, forceFallback, overrideImpl);
  }
}


void DynamicBenchmark::addBenchmark(
        bool baseline,
        const boost::filesystem::path& path,
        std::string fixtureName, bool forceFallback,
        std::map<Component, std::string> overrideImpl) {

  HL_INFO(logger, (boost::format("adding benchmark %1%") % path.string()).str());

  unsigned int numberOfSamples = 5;

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
  
  std::map<int64_t, std::string> allQueries;
  allQueries[0] = queryJSON;
  
  std::shared_ptr<celero::TestFixture> fixture(
    new DynamicCorpusFixture(forceFallback, corpus, overrideImpl, allQueries,
          benchmarkName + " (" + fixtureName + ")",
          numberOfSamples,
          expectedCount));

  if(baseline) {
    celero::RegisterBaseline(benchmarkName.c_str(), fixtureName.c_str(), numberOfSamples, 1, 1, 
            std::make_shared<DynamicCorpusFixtureFactory>(fixture));
  } else {
    celero::RegisterTest(benchmarkName.c_str(), fixtureName.c_str(), numberOfSamples, 1, 1,
            std::make_shared<DynamicCorpusFixtureFactory>(fixture));
  }
}

