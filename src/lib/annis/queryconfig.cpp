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

#include "queryconfig.h"

#include <thread>

annis::QueryConfig::QueryConfig()
  : optimize(true), forceFallback(false), avoidNestedBySwitch(true) ,
    numOfBackgroundTasks(0), enableTaskIndexJoin(false), enableThreadIndexJoin(false), threadPool(nullptr)

{
//  size_t numOfCPUs = std::thread::hardware_concurrency();
//  if(numOfCPUs > 0)
//  {
//    numOfBackgroundTasks = numOfCPUs-1;
//    threadPool = std::make_shared<ThreadPool>(numOfBackgroundTasks);
//  }
}
