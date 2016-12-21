#include <celero/Celero.h>

#include <annis/query.h>
#include <annis/annosearch/exactannokeysearch.h>
#include <annis/annosearch/exactannovaluesearch.h>
#include <annis/annosearch/regexannosearch.h>

#include <annis/operators/pointing.h>
#include <annis/operators/precedence.h>

using namespace annis;

CELERO_MAIN

class GUMFixture : public celero::TestFixture
{
    public:
        GUMFixture()
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

        std::shared_ptr<Query> query_PosDepPos(QueryConfig config)
        {
          std::shared_ptr<Query> query = std::make_shared<Query>(db, config);

          query->addNode(std::make_shared<ExactAnnoKeySearch>(db, "pos"));
          query->addNode(std::make_shared<ExactAnnoKeySearch>(db, "pos"));

          Annotation edgeAnno = {db.strings.add("func"), 0, db.strings.add("dep")};
          query->addOperator(std::make_shared<Pointing>(db.edges, db.strings, "", "dep", edgeAnno), 0, 1);
          return query;
        }

        std::shared_ptr<Query> query_UsedTo(QueryConfig config)
        {
          std::shared_ptr<Query> query = std::make_shared<Query>(db, config);

          query->addNode(std::make_shared<RegexAnnoSearch>(db, "pos", "NN.*"));
          query->addNode(std::make_shared<ExactAnnoValueSearch>(db, "annis4_internal", "tok", "used"));
          query->addNode(std::make_shared<ExactAnnoValueSearch>(db, "annis4_internal", "tok", "to"));

          query->addOperator(std::make_shared<Precedence>(db, db.edges), 0, 1);
          query->addOperator(std::make_shared<Precedence>(db, db.edges), 1, 2);

          return query;
        }

};


BASELINE_F(PosDepPos, Baseline, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 1;
  std::shared_ptr<Query> query = query_PosDepPos(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(PosDepPos, N2, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 2;
  std::shared_ptr<Query> query = query_PosDepPos(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(PosDepPos, N3, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 3;
  std::shared_ptr<Query> query = query_PosDepPos(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(PosDepPos, N4, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 4;
  std::shared_ptr<Query> query = query_PosDepPos(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(PosDepPos, N5, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 5;
  std::shared_ptr<Query> query = query_PosDepPos(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(PosDepPos, N6, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 6;
  std::shared_ptr<Query> query = query_PosDepPos(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(PosDepPos, N7, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 7;
  std::shared_ptr<Query> query = query_PosDepPos(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(PosDepPos, N8, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 8;
  std::shared_ptr<Query> query = query_PosDepPos(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BASELINE_F(UsedTo, Baseline, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 1;
  std::shared_ptr<Query> query = query_UsedTo(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(UsedTo, N2, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 2;
  std::shared_ptr<Query> query = query_UsedTo(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(UsedTo, N3, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 3;
  std::shared_ptr<Query> query = query_UsedTo(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(UsedTo, N4, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 4;
  std::shared_ptr<Query> query = query_UsedTo(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(UsedTo, N5, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 5;
  std::shared_ptr<Query> query = query_UsedTo(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(UsedTo, N6, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 6;
  std::shared_ptr<Query> query = query_UsedTo(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(UsedTo, N7, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 7;
  std::shared_ptr<Query> query = query_UsedTo(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

BENCHMARK_F(UsedTo, N8, GUMFixture, 10, 100)
{
  QueryConfig config;
  config.numOfParallelTasks = 8;
  std::shared_ptr<Query> query = query_UsedTo(config);
  int counter=0;
  while(query->next()) {
    counter++;
  }
}

