#ifndef BENCHMARK
#define BENCHMARK

#include <celero/Celero.h>

#include <boost/format.hpp>

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

HUMBLE_LOGGER(logger, "default");

using namespace annis;

template<bool optimized, char const* corpusName>
class CorpusFixture : public ::celero::TestFixture
{
public:
  CorpusFixture()
    : corpus(corpusName),
      db(optimized)
  {

  }

  virtual void setUp(int64_t experimentValue)
  {
    char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
    std::string dataDir("data");
    if(testDataEnv != NULL)
    {
      dataDir = testDataEnv;
    }
    dbLoaded = db.load(dataDir + "/" + corpus);
    counter = 0;
  }

  virtual void tearDown()
  {
     HL_INFO(logger, (boost::format("result %1%") % counter).str());
  }

  virtual ~CorpusFixture() {}

public:
  DB db;
  unsigned int counter;
private:
  const std::string corpus;

  bool dbLoaded;

};



#endif // BENCHMARK

