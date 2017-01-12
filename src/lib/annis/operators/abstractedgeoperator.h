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

#include <annis/db.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/operators/operator.h>
#include <vector>
#include <annis/annosearch/nodebyedgeannosearch.h>

namespace annis
{

class AbstractEdgeOperator : public Operator
{

public:
  AbstractEdgeOperator(ComponentType componentType,
      GraphStorageHolder& gsh, const StringStorage& strings, std::string ns, std::string name,
      unsigned int minDistance = 1, unsigned int maxDistance = 1);

  AbstractEdgeOperator(
      ComponentType componentType,
      GraphStorageHolder& gsh, const StringStorage& strings, std::string ns, std::string name,
      const Annotation& edgeAnno = Init::initAnnotation());

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs) override;
  virtual bool filter(const Match& lhs, const Match& rhs) override;

  virtual bool valid() const override {return !gs.empty();}
  
  virtual std::string operatorString() = 0;

  virtual std::string description() override;

  virtual double selectivity() override;

  virtual double edgeAnnoSelectivity() override;

  virtual std::int64_t guessMaxCountEdgeAnnos();
  
  virtual std::shared_ptr<NodeByEdgeAnnoSearch> createAnnoSearch(
      std::function<std::list<Annotation> (nodeid_t)> nodeAnnoMatchGenerator,
      bool maximalOneNodeAnno,
      int64_t wrappedNodeCountEstimate, std::string debugDescription) const;

  virtual ~AbstractEdgeOperator();
private:
  ComponentType componentType;
  GraphStorageHolder& gsh;
  const StringStorage& strings;
  std::string ns;
  std::string name;
  unsigned int minDistance;
  unsigned int maxDistance;
  Annotation anyAnno;
  const Annotation edgeAnno;

  std::vector<std::shared_ptr<const ReadableGraphStorage>> gs;

  void initGraphStorage();
  bool checkEdgeAnnotation(std::shared_ptr<const ReadableGraphStorage> gs, nodeid_t source, nodeid_t target);
};

} // end namespace annis
