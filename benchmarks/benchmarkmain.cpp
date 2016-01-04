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

  // test stuff
  annis::DynamicBenchmark dynamicTest("queries/Benchmark_ridges", "ridges");
  dynamicTest.registerBenchmarks();
  //return 0;
  
  celero::Run(argc, argv);
  return 0;
}
