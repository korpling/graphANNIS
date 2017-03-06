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

#include <celero/Celero.h>

#include <forward_list>

#include <annis/query.h>
#include <annis/annosearch/exactannokeysearch.h>
#include <annis/annosearch/exactannovaluesearch.h>
#include <annis/annosearch/regexannosearch.h>

#include <annis/operators/pointing.h>
#include <annis/operators/precedence.h>
#include <annis/operators/dominance.h>
#include <annis/operators/identicalcoverage.h>

#include <annis/util/threadpool.h>

#ifdef ENABLE_VALGRIND
  #include <valgrind/callgrind.h>
#else
  #define CALLGRIND_STOP_INSTRUMENTATION

  #define CALLGRIND_START_INSTRUMENTATION
#endif // ENABLE_VALGRIND


using namespace annis;

int main(int argc, char** argv) {
  try
  {
    celero::Run(argc, argv);
    return 0;
  }
  catch(std::string ex)
  {
    std::cerr << "ERROR: " << ex << std::endl;
  }
  catch(char const* ex)
  {
    std::cerr << "ERROR: " << ex << std::endl;
  }
  catch(...)
  {
    std::cerr << "Some exception was thrown!" << std::endl;
  }

  return -1;
}

static std::shared_ptr<ThreadPool> benchmarkThreadPool = std::make_shared<ThreadPool>(32);

class GUMFixture : public celero::TestFixture
{
    public:
        GUMFixture()
          : count_PosDepPos(246), count_UsedTo(1), count_ComplexNested(3)
        {
        }

        /*
        virtual std::vector<std::pair<int64_t, uint64_t>> getExperimentValues() const override
        {
            std::vector<std::pair<int64_t, uint64_t>> problemSpace;

            for(int64_t i=1; i <= std::thread::hardware_concurrency(); i++)
            {
              problemSpace.push_back(std::make_pair(i, uint64_t(0)));
            }
            return problemSpace;
        }
        */


        /// Before each run, build a vector of random integers.
        virtual void setUp(int64_t experimentValue)
        {
          CALLGRIND_STOP_INSTRUMENTATION;
          char* testDataEnv = std::getenv("ANNIS4_TEST_DATA");
          std::string dataDir("data");
          if (testDataEnv != NULL) {
            dataDir = testDataEnv;
          }
          db.load(dataDir + "/GUM", true);

          nonParallelConfig.numOfBackgroundTasks = 0;
          nonParallelConfig.threadPool = nullptr;


//          taskConfigs.resize(9);
          threadConfigs.resize(32);

          for(size_t i=1; i < threadConfigs.size(); i++)
          {
            threadConfigs[i].threadPool = benchmarkThreadPool;
            threadConfigs[i].numOfBackgroundTasks = i;
          }
        }

        std::shared_ptr<Query> query_PosDepPos(QueryConfig config)
        {
          std::shared_ptr<Query> result = std::make_shared<Query>(db, config);

          result->addNode(std::make_shared<ExactAnnoKeySearch>(db, "pos"));
          result->addNode(std::make_shared<ExactAnnoKeySearch>(db, "pos"));

          Annotation edgeAnno = {db.strings.add("func"), 0, db.strings.add("dep")};
          result->addOperator(std::make_shared<Pointing>(db.edges, db.strings, "", "dep", edgeAnno), 0, 1);

          return result;
        }

        std::shared_ptr<Query> query_UsedTo(QueryConfig config)
        {
          std::shared_ptr<Query> result = std::make_shared<Query>(db, config);

          result->addNode(std::make_shared<RegexAnnoSearch>(db, "pos", "NN.*"));
          result->addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_tok, "used"));
          result->addNode(std::make_shared<ExactAnnoValueSearch>(db, annis_ns, annis_tok, "to"));

          result->addOperator(std::make_shared<Precedence>(db, db.edges), 0, 1);
          result->addOperator(std::make_shared<Precedence>(db, db.edges), 1, 2);
          return result;
        }

        // entity ->coref[type="coref"] infstat & cat > tok & #1 _=_ #3 & tok & #5 ->dep[func="prep"] #4
        std::shared_ptr<Query> query_ComplexNested(QueryConfig config)
        {
          std::shared_ptr<Query> result = std::make_shared<Query>(db, config);

          Annotation edgeAnnoCoref = {db.strings.add("type"), 0, db.strings.add("coref")};
          Annotation edgeAnnoPrep = {db.strings.add("func"), 0, db.strings.add("prep")};

          result->addNode(std::make_shared<ExactAnnoKeySearch>(db, "entity"));
          result->addNode(std::make_shared<ExactAnnoKeySearch>(db, "infstat"));
          result->addNode(std::make_shared<ExactAnnoKeySearch>(db, "cat"));
          result->addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_tok));
          result->addNode(std::make_shared<ExactAnnoKeySearch>(db, annis_ns, annis_tok));

          result->addOperator(std::make_shared<Pointing>(db.edges, db.strings, "", "coref", edgeAnnoCoref), 0,1);
          result->addOperator(std::make_shared<Dominance>(db.edges, db.strings, "", ""), 2,3);
          result->addOperator(std::make_shared<IdenticalCoverage>(db, db.edges),0,2);
          result->addOperator(std::make_shared<Pointing>(db.edges, db.strings, "", "dep", edgeAnnoPrep), 4,3);

          return result;
        }

        DB db;
        QueryConfig nonParallelConfig;
        std::vector<QueryConfig> threadConfigs;

        const int count_PosDepPos;
        const int count_UsedTo;
        const int count_ComplexNested;


};

#define COUNT_BASELINE(group) \
  BASELINE_F(group, N0, GUMFixture, 0, 0) \
  { \
  CALLGRIND_START_INSTRUMENTATION;\
    std::shared_ptr<Query> q = query_##group(nonParallelConfig);\
    int counter=0; \
    while(q->next()) { \
      counter++; \
    } \
    if(counter != count_##group)\
    {\
      throw "Invalid count for N0, was " + std::to_string(counter) + " but should have been  " + std::to_string(count_##group);\
    }\
  CALLGRIND_STOP_INSTRUMENTATION;\
  }

#define COUNT_BENCH(group, idx) \
  BENCHMARK_F(group, N##idx, GUMFixture, 0, 0) \
  { \
  CALLGRIND_START_INSTRUMENTATION;\
    std::shared_ptr<Query> q = query_##group(threadConfigs[idx]);\
    int counter=0; \
    while(q->next()) { \
      counter++; \
    } \
    if(counter != count_##group)\
    {\
      throw "Invalid count for Thread_" #idx ", was " + std::to_string(counter) + " but should have been  " + std::to_string(count_##group);\
    }\
  CALLGRIND_STOP_INSTRUMENTATION;\
  }

COUNT_BASELINE(PosDepPos)
COUNT_BENCH(PosDepPos, 2)
COUNT_BENCH(PosDepPos, 4)
COUNT_BENCH(PosDepPos, 6)
COUNT_BENCH(PosDepPos, 8)
COUNT_BENCH(PosDepPos, 10)
COUNT_BENCH(PosDepPos, 12)

COUNT_BASELINE(UsedTo)
COUNT_BENCH(UsedTo, 2)
COUNT_BENCH(UsedTo, 4)
COUNT_BENCH(UsedTo, 6)
COUNT_BENCH(UsedTo, 8)
COUNT_BENCH(UsedTo, 10)
COUNT_BENCH(UsedTo, 12)

COUNT_BASELINE(ComplexNested)
COUNT_BENCH(ComplexNested, 2)
COUNT_BENCH(ComplexNested, 4)
COUNT_BENCH(ComplexNested, 6)
COUNT_BENCH(ComplexNested, 8)
COUNT_BENCH(ComplexNested, 10)
COUNT_BENCH(ComplexNested, 12)

BASELINE(CreateThreadPool, N1, 0, 0)
{
  ThreadPool t(1);
}

BENCHMARK(CreateThreadPool, N2, 0, 0)
{
  ThreadPool t(2);
}

BENCHMARK(CreateThreadPool, N3, 0, 0)
{
  ThreadPool t(3);
}

BENCHMARK(CreateThreadPool, N4, 0, 0)
{
  ThreadPool t(4);
}

BENCHMARK(CreateThreadPool, N5, 0, 0)
{
  ThreadPool t(5);
}

BENCHMARK(CreateThreadPool, N6, 0, 0)
{
  ThreadPool t(6);
}

BENCHMARK(CreateThreadPool, N7, 0, 0)
{
  ThreadPool t(7);
}

BENCHMARK(CreateThreadPool, N8, 0, 0)
{
  ThreadPool t(8);
}

BASELINE(MatchQueue, Vector, 0, 0)
{
  std::list<std::vector<Match>> queue;
  for(int i=0; i < 1000; i++)
  {
    std::vector<Match> m(2);
    queue.emplace_back(m);
  }


  std::vector<Match> m;
  while(!queue.empty())
  {
    m = queue.front();
    queue.pop_front();
  }
}

BENCHMARK(MatchQueue, VectorMove, 0, 0)
{
  std::list<std::vector<Match>> queue;
  for(int i=0; i < 1000; i++)
  {
    std::vector<Match> m(2);
    queue.emplace_back(m);
  }


  std::vector<Match> m;
  while(!queue.empty())
  {
    m = std::move(queue.front());
    queue.pop_front();
  }
}

BENCHMARK(MatchQueue, VectorMoveDeque, 0, 0)
{
  std::deque<std::vector<Match>> queue;
  for(int i=0; i < 1000; i++)
  {
    std::vector<Match> m(2);
    queue.emplace_back(m);
  }


  std::vector<Match> m;
  while(!queue.empty())
  {
    m = std::move(queue.front());
    queue.pop_front();
  }
}

BENCHMARK(MatchQueue, VectorSwap, 0, 0)
{
  std::list<std::vector<Match>> queue;
  for(int i=0; i < 1000; i++)
  {
    std::vector<Match> m(2);
    queue.emplace_back(m);
  }


  std::vector<Match> m;
  while(!queue.empty())
  {
    m.swap(queue.front());
    queue.pop_front();
  }
}


BENCHMARK(MatchQueue, VectorSwapDeque, 0, 0)
{
  std::deque<std::vector<Match>> queue;
  for(int i=0; i < 1000; i++)
  {
    std::vector<Match> m(2);
    queue.emplace_back(m);
  }


  std::vector<Match> m;
  while(!queue.empty())
  {
    m.swap(queue.front());
    queue.pop_front();
  }
}


BENCHMARK(MatchQueue, Deque, 0, 0)
{
  std::list<std::deque<Match>> queue;
  for(int i=0; i < 1000; i++)
  {
    std::deque<Match> m(2);
    queue.emplace_back(m);
  }

  std::deque<Match> m;
  while(!queue.empty())
  {
    m = std::move(queue.front());
    queue.pop_front();
  }
}

BENCHMARK(MatchQueue, DequeSwap, 0, 0)
{
  std::list<std::deque<Match>> queue;
  for(int i=0; i < 1000; i++)
  {
    std::deque<Match> m(2);
    queue.emplace_back(m);
  }

  std::deque<Match> m;
  while(!queue.empty())
  {
    m.swap(queue.front());
    queue.pop_front();
  }
}

BENCHMARK(MatchQueue, DequeSwapDeque, 0, 0)
{
  std::deque<std::deque<Match>> queue;
  for(int i=0; i < 1000; i++)
  {
    std::deque<Match> m(2);
    queue.emplace_back(m);
  }

  std::deque<Match> m;
  while(!queue.empty())
  {
    m.swap(queue.front());
    queue.pop_front();
  }
}

BENCHMARK(MatchQueue, List, 0, 0)
{
  std::list<std::list<Match>> queue;
  for(int i=0; i < 1000; i++)
  {
    std::list<Match> m(2);
    queue.emplace_back(m);
  }

  std::list<Match> m;
  while(!queue.empty())
  {
    m = std::move(queue.front());
    queue.pop_front();
  }
}



