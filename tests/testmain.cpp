#include "gtest/gtest.h"


#include <humblelogging/api.h>
HUMBLE_LOGGER(logger, "default");

#include "LoadTest.h"
#include "SearchTestPcc2.h"
#include "SearchTestRidges.h"
#include "SearchTestTiger.h"
#include "SearchTestTueBaDZ.h"

int main(int argc, char **argv)
{

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();


  fac.setDefaultLogLevel(humble::logging::LogLevel::Warn);
//  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date] %m\n"));
  fac.registerAppender(new humble::logging::FileAppender("testexecution_annis4.log"));

  humble::logging::ConsoleAppender* console = new humble::logging::ConsoleAppender();
  fac.registerAppender(console);
  console->setFormatter(new humble::logging::PatternFormatter("[%date][%lls] %m\n"));


  ::testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}
