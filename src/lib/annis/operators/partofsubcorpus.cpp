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

#include "partofsubcorpus.h"
#include "annis/operators/abstractedgeoperator.h"  // for AbstractEdgeOperator
#include "annis/types.h"                           // for ComponentType, Com...

#include <annis/graphstorageholder.h>
#include <annis/graphstorage/graphstorage.h>

namespace annis { class StringStorage; }

using namespace annis;

PartOfSubCorpus::PartOfSubCorpus(GraphStorageHolder &gsh, const StringStorage& strings)
  : AbstractEdgeOperator(ComponentType::PART_OF_SUBCORPUS,
                         gsh, strings, annis_ns, "", 1, std::numeric_limits<unsigned int>::max())
{
  gs = gsh.getGraphStorage(ComponentType::PART_OF_SUBCORPUS, annis_ns, "");
}

double PartOfSubCorpus::selectivity()
{
  double graphStorageSelectivity = 0.0;
  const auto& stat = gs->getStatistics();
  if(stat.valid)
  {
    // We assume that normally the LHS of the join is a document and we
    // always search for all connected actual annotation nodes.
    // Thus we don't use the average fan-out (which includes the empty output edge lists of the annotation nodes
    // and only use the max fan-out which gives a clearer picture how many nodes are included in a document at
    // maxium.
    std::uint32_t reachable = stat.maxFanOut;
    graphStorageSelectivity = ((double) reachable ) / ((double) stat.nodes);

  }
  else
  {
     // assume a default selecivity for this graph storage operator
     graphStorageSelectivity = 0.1;
  }

  return graphStorageSelectivity;
}

PartOfSubCorpus::~PartOfSubCorpus()
{

}

