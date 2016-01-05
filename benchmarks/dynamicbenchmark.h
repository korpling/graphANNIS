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

#include "jsonqueryparser.h"
#include "benchmark.h"
#include "db.h"
#include "query.h"
#include <boost/filesystem.hpp>
#include <boost/optional.hpp>
#include <boost/format.hpp>
#include <sstream>

namespace annis {

  class DynamicCorpusFixture : public ::celero::TestFixture {
  public:

    DynamicCorpusFixture(
            bool forceFallback,
            std::string corpus,
            std::map<Component, std::string> overrideImpl,
            std::string queryJson,
            std::string benchmarkName,
            boost::optional<unsigned int> expectedCount = boost::optional<unsigned int>())
    : forceFallback(forceFallback), corpus(corpus), overrideImpl(overrideImpl),
    queryJson(queryJson), benchmarkName(benchmarkName), counter(0), expectedCount(expectedCount) {
    }

    std::unique_ptr<DB> initDB() {
      //    std::cerr << "INIT DB " << corpus << " in " << (forceFallback ? "fallback" : "default") << " mode" <<  std::endl;
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
    
    const DB& getDB() {
      if(!db) {
        db = initDB();
      }
      return *db;
    }

    virtual void setUp(int64_t experimentValue) override {
      counter = 0;
      // create query
      std::istringstream jsonAsStream(queryJson);
      q = JSONQueryParser::parse(getDB(), jsonAsStream);

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
    std::unique_ptr<DB> db;
    std::string corpus;
    bool forceFallback;
    std::map<Component, std::string> overrideImpl;
    std::string queryJson;
    std::shared_ptr<Query> q;
    std::string benchmarkName;
    unsigned int counter;
    boost::optional<unsigned int> expectedCount;

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

    DynamicBenchmark(std::string queriesDir, std::string corpusName, bool registerOptimized = true);

    DynamicBenchmark(const DynamicBenchmark& orig) = delete;


    void registerFixture(
            std::string fixtureName,
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

    void addBenchmark(
            bool baseline,
            const boost::filesystem::path& path,
            std::string fixtureName, bool forceFallback,
            std::map<Component, std::string> overrideImpl);
  };



} // end namespace annis
#endif /* DYNAMICBENCHMARK_H */

