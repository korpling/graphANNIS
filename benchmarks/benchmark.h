#ifndef BENCHMARK
#define BENCHMARK

#include <celero/Celero.h>

#include <boost/format.hpp>
#include <memory>

#include <humblelogging/api.h>

#include <db.h>
#include <query.h>
#include <annotationsearch.h>
#include <regexannosearch.h>
#include <operators/precedence.h>
#include <operators/inclusion.h>
#include <operators/dominance.h>
#include <operators/overlap.h>
#include <operators/pointing.h>
#include <wrapper.h>
#include <graphstorageregistry.h>

HUMBLE_LOGGER(logger, "default");

namespace annis {

  class BenchmarkDBHolder {
  public:
    static std::string corpus;
    static std::unique_ptr<DB> db;
    static bool forceFallback;
  };

} // end namespace ANNIS

using namespace annis;

#define DBGETTER virtual const DB& getDB() {\
  checkBenchmarkDBHolder();\
  return *(BenchmarkDBHolder::db);\
}

template<bool forceFallback>
class CorpusFixture : public ::celero::TestFixture {
public:
  
  CorpusFixture()
  : corpus("")
  {
    
  }

  CorpusFixture(std::string corpusName)
  : corpus(corpusName) {
  }

  void checkBenchmarkDBHolder() {
    if (!BenchmarkDBHolder::db || BenchmarkDBHolder::corpus != corpus
            || BenchmarkDBHolder::forceFallback != forceFallback) {
      BenchmarkDBHolder::db = initDB();
      BenchmarkDBHolder::corpus = corpus;
      BenchmarkDBHolder::forceFallback = forceFallback;
    }
  }

  virtual void setUp(int64_t experimentValue) {
    counter = 0;
  }

  virtual void tearDown() {
    HL_INFO(logger, (boost::format("result %1%") % counter).str());
  }

  void addOverride(ComponentType ctype, std::string layer, std::string name, std::string implementation) {
    overrideImpl.insert(
            std::pair<Component, std::string>({ctype, layer, name}, implementation)
            );
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

  virtual const DB& getDB() = 0;

  virtual ~CorpusFixture() {
  }

public:
  unsigned int counter;

protected:

private:
  const std::string corpus;
  std::map<Component, std::string> overrideImpl;


};



#endif // BENCHMARK

