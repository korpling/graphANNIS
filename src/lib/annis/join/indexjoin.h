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

#pragma once

#include <annis/types.h>
#include <annis/iterators.h>
#include <annis/operators/operator.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/db.h>
#include <annis/util/comparefunctions.h>

#include <unordered_set>

namespace annis
{

/**
 * A join that takes the left argument as a seed, finds all connected nodes
 * (probably using and index of the graph storage) and checks the condition for each node.
 * This join is not parallized.
 */
class IndexJoin : public Iterator
{
public:
  IndexJoin(const DB& db, std::shared_ptr<Operator> op,
           std::shared_ptr<Iterator> lhs,
            size_t lhsIdx,
           std::function< std::list<Annotation> (nodeid_t) > matchGeneratorFunc,
           bool maximalOneRHSAnno);
  virtual ~IndexJoin();

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;
private:
  const DB& db;
  std::shared_ptr<Operator> op;

  std::shared_ptr<Iterator> left;
  const size_t lhsIdx;
  const std::function<std::list<Annotation> (nodeid_t)> matchGeneratorFunc;

  std::unique_ptr<AnnoIt> matchesByOperator;
  std::vector<Match> currentLHSMatch;
  bool currentLHSMatchValid;
  std::list<Annotation> rhsCandidates;

  Match currentRHSMatch;

  const bool operatorIsReflexive;
  const bool maximalOneRHSAnno;

private:
  bool nextLeftMatch();
  bool nextRightAnnotation();

};



} // end namespace annis

