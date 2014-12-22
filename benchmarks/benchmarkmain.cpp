#include <hayai.hpp>
#include <boost/format.hpp>

#include <humblelogging/api.h>

#include <db.h>
#include <annotationsearch.h>
#include <operators/precedence.h>
#include <operators/inclusion.h>
#include <operators/overlap.h>

HUMBLE_LOGGER(logger, "default");

using namespace annis;

class Tiger
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

class TigerFallback
    : public ::hayai::Fixture
{
public:

  TigerFallback()
    :db(false)
  {

  }

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

class Ridges
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

class RidgesFallback
    : public ::hayai::Fixture
{
public:

  RidgesFallback()
    :db(false)
  {

  }

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


BENCHMARK_F(Tiger, Cat, 5, 1)
{
  AnnotationNameSearch search(db, "cat");
  counter=0;
  while(search.hasNext())
  {
    search.next();
    counter++;
  }
}

// cat="S" & tok="Bilharziose" & #1 >* #2
BENCHMARK_F(Tiger, BilharzioseSentence, 5, 1)
{
  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, "tiger", "cat", "S"));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, annis_ns, annis_tok, "Bilharziose"));

  const EdgeDB* edbDom = db.getEdgeDB(ComponentType::DOMINANCE, "tiger", "edge");
  NestedLoopJoin n1Dom2(edbDom, n1, n2, 1, uintmax);

  for(BinaryMatch m=n1Dom2.next(); m.found; m=n1Dom2.next())
  {
    counter++;
  }

}

// pos="NN" .2,10 pos="ART" . pos="NN"
BENCHMARK_F(Tiger, NNPreARTPreNN, 5, 1) {

  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, "tiger", "pos", "NN"));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, "tiger", "pos", "ART"));
  std::shared_ptr<AnnotationIterator> n3(std::make_shared<AnnotationNameSearch>(db, "tiger", "pos", "NN"));

  std::shared_ptr<BinaryOperatorIterator> join1(std::make_shared<Precedence>(db, n1, n2, 2, 10));
  std::shared_ptr<AnnotationIterator> wrappedJoin1(std::make_shared<JoinWrapIterator>(join1));

  Precedence join2(db, wrappedJoin1, n3);
  for(BinaryMatch m = join2.next(); m.found; m = join2.next())
  {
    counter++;
  }
}

// pos="NN" .2,10 pos="ART" . pos="NN"
BENCHMARK_F(TigerFallback, NNPreARTPreNN, 5, 1) {

  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, "tiger", "pos", "NN"));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, "tiger", "pos", "ART"));
  std::shared_ptr<AnnotationIterator> n3(std::make_shared<AnnotationNameSearch>(db, "tiger", "pos", "NN"));

  std::shared_ptr<BinaryOperatorIterator> join1(std::make_shared<Precedence>(db, n1, n2, 2, 10));
  std::shared_ptr<AnnotationIterator> wrappedJoin1(std::make_shared<JoinWrapIterator>(join1));
  Precedence join2(db, wrappedJoin1, n3);
  for(BinaryMatch m = join2.next(); m.found; m = join2.next())
  {
    counter++;
  }
}

// pos="NN" & norm="Blumen" & #1 _i_ #2
BENCHMARK_F(Ridges, PosNNIncludesNormBlumen, 5, 1) {


  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, "default_ns", "pos", "NN"));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, "default_ns", "norm", "Blumen"));

  annis::Inclusion join(db, n2, n1);
  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// pos="NN" & norm="Blumen" & #1 _o_ #2
BENCHMARK_F(Ridges, PosNNOverlapsNormBlumen, 5, 1) {
  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, "default_ns", "pos", "NN"));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, "default_ns", "norm", "Blumen"));

  SeedOverlap join(db, n2, n1);
  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// pos="NN" .2,10 pos="ART"
BENCHMARK_F(Ridges, NNPreceedingART, 5, 1) {

  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, "default_ns", "pos", "NN"));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, "default_ns", "pos", "ART"));

  Precedence join(db, n1, n2, 2, 10);
  for(BinaryMatch m=join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// tok .2,10 tok
BENCHMARK_F(Ridges, TokPreceedingTok, 5, 1) {

  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, annis::annis_ns, annis::annis_tok));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, annis::annis_ns,annis::annis_tok));

  Precedence join(db, n1, n2, 2, 10);

  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// pos="NN" .2,10 pos="ART"
BENCHMARK_F(RidgesFallback, NNPreceedingART, 5, 1) {

  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, "default_ns", "pos", "NN"));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, "default_ns", "pos", "ART"));

  Precedence join(db, n1, n2, 2, 10);
  for(BinaryMatch m=join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// tok .2,10 tok
BENCHMARK_F(RidgesFallback, TokPreceedingTok, 5, 1) {

  unsigned int counter=0;

  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, annis::annis_ns, annis::annis_tok));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, annis::annis_ns,annis::annis_tok));

  Precedence join(db, n1, n2, 2, 10);

  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// pos="NN" & norm="Blumen" & #1 _i_ #2
BENCHMARK_F(RidgesFallback, PosNNIncludesNormBlumen, 5, 1) {


  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, "default_ns", "pos", "NN"));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, "default_ns", "norm", "Blumen"));

  annis::Inclusion join(db, n2, n1);
  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// pos="NN" & norm="Blumen" & #1 _o_ #2
BENCHMARK_F(RidgesFallback, PosNNOverlapsNormBlumen, 5, 1) {
  std::shared_ptr<AnnotationIterator> n1(std::make_shared<AnnotationNameSearch>(db, "default_ns", "pos", "NN"));
  std::shared_ptr<AnnotationIterator> n2(std::make_shared<AnnotationNameSearch>(db, "default_ns", "norm", "Blumen"));

  SeedOverlap join(db, n2, n1);
  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

int main(int argc, char** argv)
{
  humble::logging::Factory &fac = humble::logging::Factory::getInstance();

  fac.setDefaultLogLevel(humble::logging::LogLevel::Info);
  fac.registerAppender(new humble::logging::FileAppender("benchmark_annis4.log"));


  hayai::ConsoleOutputter consoleOutputter;

  hayai::Benchmarker::AddOutputter(consoleOutputter);
  if(argc >= 2)
  {
    for(int i=1; i < argc; i++)
    {
      std::cout << "adding include filter" << argv[i] << std::endl;
      hayai::Benchmarker::AddIncludeFilter(argv[i]);
    }
  }
  hayai::Benchmarker::RunAllTests();
  return 0;
}
