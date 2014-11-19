#include <hayai.hpp>
#include <boost/format.hpp>

#include <humblelogging/api.h>

#include <db.h>
#include <annotationsearch.h>
#include <operators/precedence.h>
#include <operators/inclusion.h>

HUMBLE_LOGGER(logger, "default");

using namespace annis;

class TigerTestFixture
    : public ::hayai::Fixture
{
public:

  virtual void SetUp()
  {
    char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
    std::string dataDir("data");
    if(testDataEnv != NULL)
    {
      dataDir = testDataEnv;
    }
    dbLoaded = db.load(dataDir + "/tiger2");
    counter = 0;
  }

  /// After each run, clear the vector of random integers.
  virtual void TearDown()
  {
     HL_INFO(logger, (boost::format("result %1%") % counter).str());
  }

  DB db;
  bool dbLoaded;

  unsigned int counter;

};

class RidgesTestFixture
    : public ::hayai::Fixture
{
public:

  virtual void SetUp()
  {
    char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
    std::string dataDir("data");
    if(testDataEnv != NULL)
    {
      dataDir = testDataEnv;
    }
    dbLoaded = db.load(dataDir + "/ridges");
    counter = 0;
  }

  /// After each run, clear the vector of random integers.
  virtual void TearDown()
  {
     HL_INFO(logger, (boost::format("result %1%") % counter).str());
  }

  DB db;
  bool dbLoaded;

  unsigned int counter;

};


BENCHMARK_F(TigerTestFixture, Cat, 5, 1)
{
  AnnotationNameSearch search(db, "cat");
  counter=0;
  while(search.hasNext())
  {
    search.next();
    counter++;
  }
}

// pos="NN" & norm="Blumen" & #1 _i_ #2
BENCHMARK_F(RidgesTestFixture, PosNNIncludesNormBlumen, 5, 1) {


  AnnotationNameSearch n1(db, "default_ns", "pos", "NN");
  AnnotationNameSearch n2(db, "default_ns", "norm", "Blumen");

  annis::Inclusion join(db, n1, n2);
  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// pos="NN" .2,10 pos="ART"
BENCHMARK_F(RidgesTestFixture, NNPreceedingART, 5, 1) {

  AnnotationNameSearch n1(db, "default_ns", "pos", "NN");
  AnnotationNameSearch n2(db, "default_ns", "pos", "ART");

  Precedence join(db, n1, n2, 2, 10);
  for(BinaryMatch m=join.next(); m.found; m = join.next())
  {
    counter++;
  }
}


int main()
{
  humble::logging::Factory &fac = humble::logging::Factory::getInstance();

  fac.setDefaultLogLevel(humble::logging::LogLevel::Info);
  fac.registerAppender(new humble::logging::FileAppender("benchmark_annis4.log"));


  hayai::ConsoleOutputter consoleOutputter;

  hayai::Benchmarker::AddOutputter(consoleOutputter);
  hayai::Benchmarker::RunAllTests();
  return 0;
}
