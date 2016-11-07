#include <gtest/gtest.h>


#include <humblelogging/api.h>
HUMBLE_LOGGER(logger, "default");

#include "LoadTest.h"
#include "SearchTestPcc2.h"
#include "SearchTestRidges.h"
#include "SearchTestTiger.h"
#include "SearchTestTueBaDZ.h"
#include "SearchTestParlament.h"
#include "SearchTestGUM.h"
#include "CorpusStorageTest.h"

int main(int argc, char **argv)
{

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();

  fac.setConfiguration(humble::logging::DefaultConfiguration::createFromString(
    "logger.level(*)=warn\n"
  ));
//  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date] %m\n"));
  fac.registerAppender(new humble::logging::FileAppender("testexecution_annis4.log"));

  ::testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}
