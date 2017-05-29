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

#ifndef DYNAMICBENCHMARK_H
#define DYNAMICBENCHMARK_H

#include <annis/json/jsonqueryparser.h>
#include <annis/db.h>
#include <annis/query/query.h>
#include <annis/dbcache.h>
#include <annis/queryconfig.h>

#include <boost/filesystem.hpp>
#include <boost/optional.hpp>
#include <boost/format.hpp>
#include <sstream>

#include <celero/Celero.h>

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
            std::string corpusPath,
            QueryConfig config,
            std::map<int64_t, std::string> json,
            std::string benchmarkName,
            std::map<int64_t, unsigned int> expectedCount = std::map<int64_t, unsigned int>())
    : corpusPath(corpusPath), config(config),
    json(json), benchmarkName(benchmarkName), counter(0),
    expectedCountByExp(expectedCount), currentExperimentValue(0) {
    }

    const std::weak_ptr<DB> getDB() {
      return dbCache->get(corpusPath, true, config.forceFallback, config.overrideImpl);
    }
    
    virtual std::vector<std::pair<int64_t, uint64_t>> getExperimentValues() const override;

    virtual void setUp(int64_t experimentValue) override {

      currentExperimentValue = experimentValue;

      counter = 0;
      q.reset();

      
      // find the correct query
      auto it = json.find(experimentValue);
      if(it != json.end())
      {
        // create query
        std::istringstream jsonAsStream(it->second);
        if(auto dbPtr = getDB().lock())
        {
          DB& db = *dbPtr ;
          q = JSONQueryParser::parse(db, jsonAsStream, config);
        }
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

    std::string corpusPath;
    const QueryConfig config;
    std::map<int64_t, std::string> json;
    std::shared_ptr<Query> q;
    std::string benchmarkName;
    unsigned int counter;

    std::map<int64_t, unsigned int> expectedCountByExp;
    boost::optional<unsigned int> expectedCount;
    int64_t currentExperimentValue;
    
    
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

    DynamicBenchmark(std::string queriesDir, std::string corpusPath, std::string benchmarkName,
       bool multipleExperiments=false);

    DynamicBenchmark(const DynamicBenchmark& orig) = delete;


    void registerFixture(std::string fixtureName,
            const QueryConfig config = QueryConfig()
            );

    virtual ~DynamicBenchmark() {
    }

  private:

    void registerFixtureInternal(bool baseline,
            std::string fixtureName,
            const QueryConfig config = QueryConfig());

  private:
    std::string corpusPath;
    std::string benchmarkName;

    std::list<boost::filesystem::path> foundJSONFiles;
    
    bool multipleExperiments;
    
    void addBenchmark(bool baseline,
            std::string benchmarkName,
            std::map<int64_t, const boost::filesystem::path>& paths,
            std::string fixtureName,
            QueryConfig config);
  };



} // end namespace annis
#endif /* DYNAMICBENCHMARK_H */

