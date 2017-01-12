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

#include <annis/annosearch/annotationsearch.h>
#include <annis/graphstorage/graphstorage.h>
#include <functional>

namespace annis
{
class NodeByEdgeAnnoSearch : public EstimatedSearch
{

  using ItType = BTreeMultiAnnoStorage<Edge>::InverseAnnoMap_t::const_iterator;
  using Range = std::pair<ItType, ItType>;

public:
  NodeByEdgeAnnoSearch(std::vector<std::shared_ptr<const ReadableGraphStorage>> gs, std::set<Annotation> validEdgeAnnos,
                       std::function<std::list<Annotation> (nodeid_t)> nodeAnnoMatchGenerator,
                       bool maximalOneNodeAnno,
                       std::int64_t wrappedNodeCountEstimate,
                       std::string debugDescription="");

  virtual bool next(Match& m) override;
  virtual void reset() override;

  std::function<std::list<Annotation> (nodeid_t)> getNodeAnnoMatchGenerator()
  {
    return nodeAnnoMatchGenerator;
  }

  virtual std::int64_t guessMaxCount() const override {return wrappedNodeCountEstimate;}

  virtual std::string debugString() const override {return debugDescription;}

  virtual ~NodeByEdgeAnnoSearch();
private:
  std::function<std::list<Annotation> (nodeid_t)> nodeAnnoMatchGenerator;
public:
  const bool maximalOneNodeAnno;
private:
  const std::int64_t wrappedNodeCountEstimate;
  const std::string debugDescription;


  std::list<Match> currentMatchBuffer;

  std::list<Range> searchRanges;
  std::list<Range>::const_iterator currentRange;
  ItType it;

  std::unordered_set<nodeid_t> visited;


private:
  bool nextMatchBuffer();
};

}

