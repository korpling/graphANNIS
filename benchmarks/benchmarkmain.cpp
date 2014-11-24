#include <hayai.hpp>
#include <boost/format.hpp>

#include <humblelogging/api.h>

#include <db.h>
#include <annotationsearch.h>
#include <operators/precedence.h>
#include <operators/inclusion.h>

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

// pos="NN" & norm="Blumen" & #1 _i_ #2
BENCHMARK_F(Ridges, PosNNIncludesNormBlumen, 5, 1) {


  AnnotationNameSearch n1(db, "default_ns", "pos", "NN");
  AnnotationNameSearch n2(db, "default_ns", "norm", "Blumen");

  annis::Inclusion join(db, n1, n2);
  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// pos="NN" .2,10 pos="ART"
BENCHMARK_F(Ridges, NNPreceedingART, 5, 1) {

  AnnotationNameSearch n1(db, "default_ns", "pos", "NN");
  AnnotationNameSearch n2(db, "default_ns", "pos", "ART");

  Precedence join(db, n1, n2, 2, 10);
  for(BinaryMatch m=join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// tok .2,10 tok
BENCHMARK_F(Ridges, TokPreceedingTok, 5, 1) {

  AnnotationNameSearch n1(db, annis::annis_ns, annis::annis_tok);
  AnnotationNameSearch n2(db, annis::annis_ns,annis::annis_tok);

  Precedence join(db, n1, n2, 2, 10);

  for(BinaryMatch m = join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// tok .2,10 tok
BENCHMARK_F(Ridges, ClassicTok, 5, 1) {

  unsigned int counter=0;

  AnnotationNameSearch n1(db, annis::annis_ns, "tok");

  Annotation anyTokAnno = Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());

  std::pair<bool, uint32_t> n2_namespaceID = db.strings.findID(annis::annis_ns);
  std::pair<bool, uint32_t> n2_nameID = db.strings.findID("tok");
  if(n2_nameID.first && n2_namespaceID.first)
  {
    Component cOrder = Init::initComponent(ComponentType::ORDERING, annis_ns, "");


    const EdgeDB* edbOrder = db.getEdgeDB(cOrder);
    if(edbOrder != NULL)
    {
      while(n1.hasNext())
      {
        Match m1 = n1.next();

        // find all token in the range 2-10
        EdgeIterator* itConnected = edbOrder->findConnected(m1.node, 2, 10);
        for(std::pair<bool, std::uint32_t> tok2 = itConnected->next();
            tok2.first; tok2 = itConnected->next())
        {
          // check if the node has the correct annotations
          for(const Annotation& anno : db.getNodeAnnotationsByID(tok2.second))
          {
            if(checkAnnotationEqual(anyTokAnno, anno))
            {
              counter++;
              break; // we don't have to search for other annotations
            }
          }
        }
        delete itConnected;
      }
    }
  } // end if

}

// pos="NN" .2,10 pos="ART"
BENCHMARK_F(RidgesFallback, NNPreceedingART, 5, 1) {

  AnnotationNameSearch n1(db, "default_ns", "pos", "NN");
  AnnotationNameSearch n2(db, "default_ns", "pos", "ART");

  Precedence join(db, n1, n2, 2, 10);
  for(BinaryMatch m=join.next(); m.found; m = join.next())
  {
    counter++;
  }
}

// tok .2,10 tok
BENCHMARK_F(RidgesFallback, TokPreceedingTok, 5, 1) {

  unsigned int counter=0;

  AnnotationNameSearch n1(db, annis::annis_ns, annis::annis_tok);
  AnnotationNameSearch n2(db, annis::annis_ns,annis::annis_tok);

  Precedence join(db, n1, n2, 2, 10);

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
