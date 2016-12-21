#include <celero/Celero.h>

#include <annis/query.h>
#include <annis/annosearch/exactannokeysearch.h>
#include <annis/operators/pointing.h>

using namespace annis;

CELERO_MAIN

class QueryFixture : public celero::TestFixture
{
    public:
        QueryFixture()
        {
        }

        /*
        virtual std::vector<std::pair<int64_t, uint64_t>> getExperimentValues() const override
        {
            std::vector<std::pair<int64_t, uint64_t>> problemSpace;
            problemSpace.push_back(std::make_pair(1, uint64_t(0)));

            return problemSpace;
        }
        */

        /// Before each run, build a vector of random integers.
        virtual void setUp(int64_t experimentValue)
        {
          char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
          std::string dataDir("data");
          if (testDataEnv != NULL) {
            dataDir = testDataEnv;
          }
          db.load(dataDir + "/GUM", true);
        }

        DB db;

        std::shared_ptr<Query> createQuery(QueryConfig config)
        {
          std::shared_ptr<Query> query = std::make_shared<Query>(db, config);

          query->addNode(std::make_shared<ExactAnnoKeySearch>(db, "pos"));
          query->addNode(std::make_shared<ExactAnnoKeySearch>(db, "pos"));

          Annotation edgeAnno = {db.strings.add("func"), 0, db.strings.add("dep")};
          query->addOperator(std::make_shared<Pointing>(db.edges, db.strings, "", "dep", edgeAnno), 0, 1);
          return query;
        }
};


BASELINE_F(Parallel, Baseline, QueryFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 1;
  std::shared_ptr<Query> query = createQuery(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(Parallel, N2, QueryFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 2;
  std::shared_ptr<Query> query = createQuery(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(Parallel, N3, QueryFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 3;
  std::shared_ptr<Query> query = createQuery(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(Parallel, N4, QueryFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 4;
  std::shared_ptr<Query> query = createQuery(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(Parallel, N5, QueryFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 5;
  std::shared_ptr<Query> query = createQuery(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(Parallel, N6, QueryFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 6;
  std::shared_ptr<Query> query = createQuery(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(Parallel, N7, QueryFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 7;
  std::shared_ptr<Query> query = createQuery(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(Parallel, N8, QueryFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 8;
  std::shared_ptr<Query> query = createQuery(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

