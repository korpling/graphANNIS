/* 
 * File:   DynamicBenchmark.cpp
 * Author: thomas
 * 
 * Created on 4. Januar 2016, 11:54
 */

#include "dynamicbenchmark.h"
#include <annis/json/jsonqueryparser.h>

#include <humblelogging/api.h>
#include <boost/filesystem.hpp>
#include <boost/filesystem/fstream.hpp>
#include <string>
#include <stddef.h>

using namespace annis;

HUMBLE_LOGGER(benchLogger, "DynamicBenchmark");

std::shared_ptr<DBCache> DynamicCorpusFixture::dbCache
  = std::make_shared<DBCache>(0);

void DynamicCorpusFixture::UserBenchmark()
{
  while (q->next())
  {
    counter++;
  }
  HL_INFO(benchLogger, (boost::format("result %1%") % counter).str());
  if (expectedCount && counter != *expectedCount)
  {
    std::cerr << "FATAL ERROR: query " << benchmarkName << " should have count " << *expectedCount << " but was " << counter << std::endl;
    std::cerr << "" << __FILE__ << ":" << __LINE__ << std::endl;
    exit(-1);
  }
}

std::vector<std::pair<int64_t, uint64_t> > DynamicCorpusFixture::getExperimentValues() const
{
  std::vector<std::pair<int64_t, uint64_t> > result;

  for (auto it : json)
  {
    result.push_back({it.first, 0});
  }

  return result;
}

void DynamicCorpusFixture::tearDown()
{
}

DynamicBenchmark::DynamicBenchmark(std::string queriesDir,
  std::string corpusPath, std::string benchmarkName, bool multipleExperimentsParam)
  : corpusPath(corpusPath), benchmarkName(benchmarkName), multipleExperiments(multipleExperimentsParam)
{
  // find all file ending with ".json" in the folder
  boost::filesystem::directory_iterator fileEndIt;

  boost::filesystem::directory_iterator itFiles(queriesDir);
  while (itFiles != fileEndIt)
  {
    const auto filePath = itFiles->path();
    if (filePath.extension().string() == ".json")
    {
      if(multipleExperiments)
      {
        // check if the file name is a valid number
        std::string name = filePath.filename().stem().string();
        try
        {
          std::stol(name);
        }
        catch(std::invalid_argument invalid)
        {
          // not a number, don't assume we have multiple experiments
          multipleExperiments = false;
        }
      }
       
      foundJSONFiles.push_back(filePath);
    }
    itFiles++;
  }
  
  if(foundJSONFiles.empty())
  {
    multipleExperiments = false;
  }

  QueryConfig baselineConfig;
  baselineConfig.forceFallback = true;
  registerFixtureInternal(true, "Baseline", baselineConfig);
}

void DynamicBenchmark::registerFixture(std::string fixtureName, const QueryConfig config)
{
  registerFixtureInternal(false, fixtureName, config);
}

void DynamicBenchmark::registerFixtureInternal(
  bool baseline,
  std::string fixtureName, const QueryConfig config)
{
  if (multipleExperiments)
  {
    std::map<int64_t, const boost::filesystem::path> paths;
    for (const auto& filePath : foundJSONFiles)
    {
      // try to get a numerical ID from the file name
      std::string name = filePath.filename().stem().string();
      auto id = std::stol(name);
      paths.insert({id, filePath});
    }
    addBenchmark(baseline, benchmarkName, paths, fixtureName, config);
  }
  else
  {
    for (const auto& filePath : foundJSONFiles)
    {
      std::map<int64_t, const boost::filesystem::path> paths;
      paths.insert({0, filePath});
      auto subBenchmarkName = benchmarkName + "_" + filePath.stem().string();
      addBenchmark(baseline, subBenchmarkName, paths, fixtureName, config);
    }
  }
}


void DynamicBenchmark::addBenchmark(bool baseline,
  std::string benchmarkName,
  std::map<int64_t, const boost::filesystem::path>& paths,
  std::string fixtureName,
  QueryConfig config)
{
  unsigned int numberOfSamples = 5;

  HL_INFO(benchLogger, (boost::format("adding benchmark %1%") % benchmarkName).str());

  std::map<int64_t, std::string> allQueries;
  std::map<int64_t, unsigned int> expectedCount;
  std::map<int64_t, uint64_t> fixedValues;

  for (auto p : paths)
  {
    auto countPath = p.second.parent_path() /= (p.second.stem().string() + ".count");

    boost::filesystem::ifstream stream;

    stream.open(countPath);
    if (stream.is_open())
    {
      unsigned int tmp;
      stream >> tmp;
      stream.close();
      expectedCount.insert({p.first, tmp});
    }

    stream.open(p.second);
    std::string queryJSON(
      (std::istreambuf_iterator<char>(stream)),
      (std::istreambuf_iterator<char>()));
    stream.close();
    
    allQueries.insert({p.first, queryJSON});
    
    if(baseline)
    {
      uint64_t timeVal = 0;
      auto timePath = p.second.parent_path() /= (p.second.stem().string() + ".time");
      stream.open(timePath);
      if (stream.is_open())
      {
        stream >> timeVal;
        stream.close();
      }
      if(timeVal == 0)
      {
        // we would divide by zero later
        timeVal = 1;
      }
      // since celero uses microseconds an ANNIS milliseconds the value needs to be converted
      fixedValues.insert({p.first, timeVal*1000});
    }
  }
  std::shared_ptr<::celero::TestFixture> fixture(
    new DynamicCorpusFixture(corpusPath, config, allQueries,
    benchmarkName + " (" + fixtureName + ")",
    expectedCount));

  if (baseline)
  {
    if(fixedValues.size() > 0)
    {
      std::shared_ptr<::celero::TestFixture> fixedFixture(new FixedValueFixture(fixedValues));
      celero::RegisterBaseline(benchmarkName.c_str(), fixtureName.c_str(), numberOfSamples, 1, 1,
        std::make_shared<DynamicCorpusFixtureFactory>(fixedFixture));
      
    }
    else
    {
     celero::RegisterBaseline(benchmarkName.c_str(), fixtureName.c_str(), numberOfSamples, 1, 1,
        std::make_shared<DynamicCorpusFixtureFactory>(fixture));
    }
  }
  else
  {
    celero::RegisterTest(benchmarkName.c_str(), fixtureName.c_str(), numberOfSamples, 1, 1,
      std::make_shared<DynamicCorpusFixtureFactory>(fixture));
  }
}

