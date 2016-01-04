#include <celero/Celero.h>
#include <humblelogging/api.h>

#include "dynamicbenchmark.h"

int main(int argc, char **argv)
{

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();
  fac.setDefaultLogLevel(humble::logging::LogLevel::All);
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%lls] %m\n"));
  fac.registerAppender(new humble::logging::ConsoleAppender());

  // test stuff
  //annis::DynamicBenchmark dynamicTest("data", "queries/Benchmark_ridges", "ridges");
  
  fac.setDefaultLogLevel(humble::logging::LogLevel::Info);
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date]- %m (%lls, %filename:%line)\n"));
  fac.registerAppender(new humble::logging::FileAppender("benchmark_annis4.log"));

  celero::Run(argc, argv);
  return 0;
}
