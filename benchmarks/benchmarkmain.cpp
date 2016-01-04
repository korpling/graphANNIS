#include <celero/Celero.h>
#include <humblelogging/api.h>

#include "dynamicbenchmark.h"

int main(int argc, char **argv)
{

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();
  fac.setConfiguration(humble::logging::DefaultConfiguration::createFromString(
    "logger.level(*)=info\n"
  ));  
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date]- %m (%lls, %filename:%line)\n"));
  fac.registerAppender(new humble::logging::FileAppender("benchmark_annis4.log", true));

  char* testQueriesEnv = std::getenv("ANNIS4_TEST_QUERIES");
  std::string queriesDir("queries");
  if (testQueriesEnv != NULL) {
    queriesDir = testQueriesEnv;
  }
  
  annis::DynamicBenchmark benchmarksRidges("ridges");
  benchmarksRidges.registerDefaultBenchmarks(queriesDir + "/Benchmark_ridges");
  
  celero::Run(argc, argv);
  return 0;
}
