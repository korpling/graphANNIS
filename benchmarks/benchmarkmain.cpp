#include <celero/Celero.h>
#include <humblelogging/api.h>

HUMBLE_LOGGER(logger, "default");


int main(int argc, char **argv)
{

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();


  fac.setDefaultLogLevel(humble::logging::LogLevel::Info);
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date]- %m (%lls, %filename:%line)\n"));
  fac.registerAppender(new humble::logging::FileAppender("benchmark_annis4.log"));

  celero::Run(argc, argv);
  return 0;
}
