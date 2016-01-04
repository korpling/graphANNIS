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
            const DB& db,
            std::string queryJson,
            std::string benchmarkName,
            boost::optional<unsigned int> expectedCount = boost::optional<unsigned int>())
    : db(db), queryJson(queryJson), benchmarkName(benchmarkName), counter(0), expectedCount(expectedCount) {
    }

    virtual void setUp(int64_t experimentValue) override {
      counter = 0;
      // create query
      std::istringstream jsonAsStream(queryJson);
      q = JSONQueryParser::parse(db, jsonAsStream);
      
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
    const DB& db;
    std::string queryJson;
    std::shared_ptr<Query> q;
    std::string benchmarkName;
    unsigned int counter;
    boost::optional<unsigned int> expectedCount;

  };

  class DynamicBenchmark {
  public:

    DynamicBenchmark(std::string queriesDir, std::string corpusName);

    DynamicBenchmark(const DynamicBenchmark& orig) = delete;

    void registerBenchmarks();

    virtual ~DynamicBenchmark() {
    }
  protected:

    void addOverride(ComponentType ctype, std::string layer, std::string name, std::string implementation) {
      overrideImpl.insert(
              std::pair<Component, std::string>({ctype, layer, name}, implementation)
              );
    }
  private:
    std::string queriesDir;
    std::string corpus;
    std::map<Component, std::string> overrideImpl;

    std::unique_ptr<DB> fallbackDB;
    std::unique_ptr<DB> optimizedDB;

    void addBenchmark(const boost::filesystem::path& path);

    std::unique_ptr<DB> initDB(bool forceFallback);
  };

  class DynamicCorpusFixtureFactory : public celero::Factory {
  public:

    DynamicCorpusFixtureFactory(
        std::string queryJson,
        std::string benchmarkName, const DB& db,
        boost::optional<unsigned int> expectedCount = boost::optional<unsigned int>())
      : queryJson(queryJson), benchmarkName(benchmarkName), db(db), expectedCount(expectedCount) {
    }

    std::shared_ptr<celero::TestFixture> Create() override {
      return std::shared_ptr<celero::TestFixture>(
            new DynamicCorpusFixture(db, queryJson, benchmarkName, expectedCount)
            );
    }
  private:
    std::string queryJson;
    std::string benchmarkName;
    const DB& db;
    boost::optional<unsigned int> expectedCount;
  };

} // end namespace annis
#endif /* DYNAMICBENCHMARK_H */

