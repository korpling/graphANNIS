#include <celero/Celero.h>
#include <humblelogging/api.h>

#include "dynamicbenchmark.h"

using namespace annis;

int main(int argc, char **argv) {

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

  DynamicBenchmark benchmarksRidges(dir + "/Benchmarks/ridges", "ridges");

  std::map<Component, std::string> prepostRidges;
  prepostRidges.insert({
    {ComponentType::COVERAGE, annis_ns, ""}, GraphStorageRegistry::prepostorderO32L32});
  prepostRidges.insert({
    {ComponentType::COVERAGE, "default_ns", ""}, GraphStorageRegistry::prepostorderO32L32});
  prepostRidges.insert({
    {ComponentType::ORDERING, annis_ns, ""}, GraphStorageRegistry::prepostorderO32L32});

  benchmarksRidges.registerFixture("PrePost", prepostRidges);

  // PARLAMENT //
  DynamicBenchmark benchmarksParlament(dir + "/Benchmarks/parlament", "parlament");

  // TIGER2 //
  DynamicBenchmark benchmarksTiger2(dir + "/Benchmarks/tiger2", "tiger2");
  
  // TuebaDZ6 //
  DynamicBenchmark benchmarksTuebadz6(dir + "/Benchmarks/tuebadz6", "tuebadz6");

  celero::Run(argc, argv);
  return 0;
}
