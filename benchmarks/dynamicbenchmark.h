/*
 * To change this license header, choose License Headers in Project Properties.
 * To change this template file, choose Tools | Templates
 * and open the template in the editor.
 */

/* 
 * File:   DynamicBenchmark.h
 * Author: thomas
 *
 * Created on 4. Januar 2016, 11:54
 */

#ifndef DYNAMICBENCHMARK_H
#define DYNAMICBENCHMARK_H

#include <json/jsonqueryparser.h>
#include "benchmark.h"
#include "db.h"
#include "query.h"
#include "DBCache.h"

#include <boost/filesystem.hpp>
#include <boost/optional.hpp>
#include <boost/format.hpp>
#include <sstream>

namespace annis {

  class FixedValueFixture : public ::celero::TestFixture
  {
  public:

    FixedValueFixture(std::map<int64_t, uint64_t> fixedValues)
      : fixedValues(fixedValues), currentFixedVal(0) 
      {
        for(auto it=fixedValues.begin(); it != fixedValues.end(); it++)
        {
          expValues.push_back({it->first, 0});
        }
      }
      
    virtual uint64_t run(uint64_t threads, uint64_t iterations, int64_t experimentValue)
    {
      auto itFixedValue = fixedValues.find(experimentValue);
      if(itFixedValue == fixedValues.end())
      {
        currentFixedVal = 0;
      }
      else
      {
        currentFixedVal = itFixedValue->second;
      }
      return ::celero::TestFixture::run(threads, iterations, experimentValue);
    }
    
    virtual std::vector<std::pair<int64_t, uint64_t>> getExperimentValues() const { return expValues;}

    virtual uint64_t HardCodedMeasurement() const
    {
      return currentFixedVal;
    }

    virtual ~FixedValueFixture() {};
  private:
    std::map<int64_t, uint64_t> fixedValues;
    uint64_t currentFixedVal;
    std::vector<std::pair<int64_t, uint64_t> > expValues;
  };
  
  class DynamicCorpusFixture : public ::celero::TestFixture {
  public:

    DynamicCorpusFixture(
            bool forceFallback,
            std::string corpus,
            std::map<Component, std::string> overrideImpl,
            std::map<int64_t, std::string> json,
            std::string benchmarkName,
            unsigned int numberOfSamples,
            std::map<int64_t, unsigned int> expectedCount = std::map<int64_t, unsigned int>())
    : forceFallback(forceFallback), corpus(corpus), overrideImpl(overrideImpl),
    json(json), benchmarkName(benchmarkName), counter(0),
    numberOfSamples(numberOfSamples), executionCounter(0),
    expectedCountByExp(expectedCount) {
    }

    const DB& getDB() {
      return dbCache->get(corpus, forceFallback, overrideImpl);
    }
    
    virtual std::vector<std::pair<int64_t, uint64_t>> getExperimentValues() const;

    virtual void setUp(int64_t experimentValue) override {
      counter = 0;
      q.reset();
      
      // find the correct query
      auto it = json.find(experimentValue);
      if(it != json.end())
      {
        // create query
        std::istringstream jsonAsStream(it->second);
        q = JSONQueryParser::parse(getDB(), jsonAsStream);
      }
      auto itCount = expectedCountByExp.find(experimentValue);
      if(itCount == expectedCountByExp.end())
      {
        expectedCount = boost::optional<unsigned int>();
      }
      else
      {
        expectedCount = itCount->second;
      }

      if (!q) {
        std::cerr << "FATAL ERROR: no query given for benchmark " << benchmarkName << std::endl;
        std::cerr << "" << __FILE__ << ":" << __LINE__ << std::endl;
        exit(-1);
      }
    }

    virtual void tearDown() override;

    virtual void UserBenchmark() override;

    virtual ~DynamicCorpusFixture() {
    }

  protected:

  private:
    std::string corpus;
    bool forceFallback;
    std::map<Component, std::string> overrideImpl;
    std::map<int64_t, std::string> json;
    std::shared_ptr<Query> q;
    std::string benchmarkName;
    unsigned int numberOfSamples;
    unsigned int executionCounter;
    unsigned int counter;

    std::map<int64_t, unsigned int> expectedCountByExp;
    boost::optional<unsigned int> expectedCount;
    
    
    static std::shared_ptr<DBCache> dbCache;

  };

  class DynamicCorpusFixtureFactory : public celero::Factory {
  public:

    DynamicCorpusFixtureFactory(std::shared_ptr<celero::TestFixture> fixture)
    : fixture(fixture) {
    }

    std::shared_ptr<celero::TestFixture> Create() override {
      return fixture;
    }
  private:
    std::shared_ptr<celero::TestFixture> fixture;
  };

  class DynamicBenchmark {
  public:

    DynamicBenchmark(std::string queriesDir, std::string corpusName, 
       bool multipleExperiments=false);

    DynamicBenchmark(const DynamicBenchmark& orig) = delete;


    void registerFixture(
            std::string fixtureName,
            bool forceFallback = false,
            std::map<Component, std::string> overrideImpl = std::map<Component, std::string>()
            );

    virtual ~DynamicBenchmark() {
    }

  private:

    void registerFixtureInternal(
            bool baseline,
            std::string fixtureName,
            bool forceFallback = false,
            std::map<Component, std::string> overrideImpl = std::map<Component, std::string>()
            );

  private:
    std::string corpus;

    std::list<boost::filesystem::path> foundJSONFiles;
    
    bool multipleExperiments;
    
    void addBenchmark(
            bool baseline,
            std::string benchmarkName,
            std::map<int64_t, const boost::filesystem::path>& paths,
            std::string fixtureName, bool forceFallback,
            std::map<Component, std::string> overrideImpl);
  };



} // end namespace annis
#endif /* DYNAMICBENCHMARK_H */

