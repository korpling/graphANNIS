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
#include "annis/types.h"  // for Component

annis::QueryConfig::QueryConfig()
  : optimize(true),
    optimize_operand_order(true),
    optimize_unbound_regex(true),
    optimize_nodeby_edgeanno(true),
    optimize_join_order(true),
    all_permutations_threshold(6),
    forceFallback(false),
    avoidNestedBySwitch(true),
    numOfBackgroundTasks(0),
    enableTaskIndexJoin(false),
    enableThreadIndexJoin(true),
    enableSIMDIndexJoin(false),
    threadPool(nullptr)

{

}
