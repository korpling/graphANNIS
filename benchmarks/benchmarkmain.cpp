#include <celero/Celero.h>
#include <humblelogging/api.h>

#include "dynamicbenchmark.h"

using namespace annis;

int main(int argc, char **argv)
{

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();
  fac.setConfiguration(humble::logging::DefaultConfiguration::createFromString(
    "logger.level(*)=info\n"
  ));  
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date]- %m (%lls, %filename:%line)\n"));
  fac.registerAppender(new humble::logging::FileAppender("benchmark_annis4.log", true));

  char* testQueriesEnv = std::getenv("ANNIS4_TEST_QUERIES");
  std::string dir("queries");
  if (testQueriesEnv != NULL) {
    dir = testQueriesEnv;
  }
  
  // RIDGES //
  
  DynamicBenchmark benchmarksRidges("ridges");
  benchmarksRidges.registerDefaultFixtures(dir + "/Benchmark_ridges");
  
  std::map<Component, std::string> prepostRiges;
  prepostRiges.insert({{ComponentType::COVERAGE, annis_ns, ""}, GraphStorageRegistry::prepostorderO32L32});
  prepostRiges.insert({{ComponentType::COVERAGE, "default_ns", ""}, GraphStorageRegistry::prepostorderO32L32});
  prepostRiges.insert({{ComponentType::ORDERING, annis_ns, ""}, GraphStorageRegistry::prepostorderO32L32});
  
  benchmarksRidges.registerFixture(dir + "/Benchmark_ridges", 
          "PrePost", prepostRiges);
  
  celero::Run(argc, argv);
  return 0;
}
