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

#include <gtest/gtest.h>

#include "LoadTest.h"
#include "SearchTestPcc2.h"
#include "SearchTestRidges.h"
#include "SearchTestTiger.h"
#include "SearchTestParlament.h"
#include "SearchTestGUM.h"
#include "CorpusStorageManagerTest.h"
#include "DFSTest.h"

int main(int argc, char **argv)
{

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();

  fac.setConfiguration(humble::logging::DefaultConfiguration::createFromString(
    "logger.level(*)=info\n"
  ));
//  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date] %m\n"));
  fac.registerAppender(new humble::logging::FileAppender("testexecution_annis4.log"));

  ::testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}
